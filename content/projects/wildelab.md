---
{
    "title": "Wildelab: A WireGuard-Meshed Homelab",
    "date": "2026-05-29",
    "draft": false,
    "description": "A self-hosted rack reconciled by Ansible from cron, talking to itself over WireGuard, serving local LLMs behind a wildcard cert."
}
---

# Wildelab

I wanted to be able to remotely develop on a static device. Not a thin
client, not a cloud VM that disappears, not a laptop on battery. A
real machine at home that I can SSH into from anywhere and find my
tools, my models, and all the services I lean on already running and
warm.

The rack is six machines: a Raspberry Pi gateway, a Framework Desktop
running llama.cpp on Vulkan, a Jetson Orin Nano running llama.cpp on
CUDA, four Pi 5 gameservers, and a $5/mo Vultr VPS sitting outside the
NAT. Everything is reachable from anywhere I happen to be over a
WireGuard mesh, and everything converges on its own from a single git
repository.

This post walks through three pieces of that setup: the VPN + DNS
fabric, the Ansible bootstrap-then-cron flow, and the LLM serving
stack. The rest (Beszel monitoring, SearXNG, Open WebUI, an Incus VM
pool) gets a passing nod where it touches these.

## VPN + DNS: one wildcard cert, no public exposure

The whole rack is private; the only thing that talks to the internet is
the VPS. WireGuard glues my laptop, phone, and any friends' devices
to the VPS, and the VPS bridges them to the gateway over a second
tunnel. Visually:

```
laptop / phone / friends           (10.9.9.0/24)
       │
       │ WireGuard
       ▼
VPS (vpn.wildelab.com)             public IP, 10.9.9.1
       │
       │ WireGuard
       ▼
Pi gateway (10.42.0.1 / 10.9.9.2)
       │
       │ ethernet
       ▼
framework (10.42.0.10)
jetson    (10.42.0.20)
pi5-0..3  (10.42.0.50-53)
```

The gateway has two faces: `10.42.0.1` on the rack side and `10.9.9.2`
on the WireGuard mesh. Roaming peers reach the rack subnet via the
VPS's `FORWARD` + `MASQUERADE`, so a packet from my phone to
`framework.wildelab.com` goes phone to VPS to gateway to framework with
no extra config on my phone.

### Split-DNS via dnsmasq

Names are the interesting part. I want `llm.wildelab.com` to resolve
when I'm on the mesh but not leak anywhere public. So the gateway runs
dnsmasq with a host file that knows every rack device, and the VPS runs
a second dnsmasq instance that forwards anything under `wildelab.com`
to the gateway:

```conf
# On the VPS (which is the laptop's DNS server while WG is up).
server=/wildelab.com/10.9.9.2

# Apex 'wildelab.com' resolves to the gateway's rack IP, reachable from
# any WG peer thanks to the VPS forwarding rules above.
# (this lives in the gateway's dnsmasq.conf)
host-record=wildelab.com,10.42.0.1
```

The only public DNS records I own are `vpn.wildelab.com` and
`vps.wildelab.com` (the VPS itself) and an `NS` delegation for
`acme.wildelab.com`. Everything else is gateway-internal.

### Wildcard cert without exposing port 80

I wanted real Let's Encrypt certs for every internal service: `llm.`,
`llama.`, `llama2.`, `searx.`, `status.`. HTTP-01 would mean exposing
the gateway to the public internet, which defeats the point. So:
DNS-01 via [acme-dns](https://github.com/joohoi/acme-dns) running on
the VPS.

The flow:

1. Certbot on the gateway asks for `*.wildelab.com`.
2. Let's Encrypt says "prove you control `wildelab.com` by setting a
   TXT at `_acme-challenge.wildelab.com`."
3. Certbot calls a hook script that POSTs the challenge to acme-dns on
   the VPS.
4. `_acme-challenge.wildelab.com` is a `CNAME` to a random subdomain of
   `acme.wildelab.com`, which acme-dns is authoritative for.
5. Let's Encrypt resolves the CNAME, queries the VPS, sees the
   challenge, hands me the cert.

I only had to set one CNAME at the registrar once. Renewals are
automatic: certbot's timer runs, hits acme-dns, gets a new cert, and
the gateway's nginx auto-reloads.

## Ansible: bootstrap once, reconcile forever

The whole rack is described by one git repository. Originally I ran
`ansible-playbook` from my laptop whenever I changed something, which
meant whenever I forgot to run it, the rack drifted. Now there are two
paths:

- **Bootstrap** (`playbooks/setup/*.yml`): the from-scratch path. Heavy
  apt installs, WireGuard key generation, cert issuance, kernel cmdline
  edits, source clones. Run by hand, with manual gates between stages
  (e.g. adding the acme-dns CNAME at the registrar).
- **Scheduled** (`playbooks/scheduled/*.yml`): cron-driven reconciliation
  from the gateway itself. Two playbooks, both idempotent: a daily
  `reconcile.yml` that re-applies config templates and SSH-key sync, and
  a weekly `apt-update.yml` that does `apt update && apt upgrade --safe`
  host-by-host.

The two paths share roles where it makes sense and split when it
doesn't. `reconcile.yml` imports the `base` and `gateway` roles
directly, since they're both safe to re-run forever. The cron command
tacks on `--skip-tags setup`, which filters out the few tasks inside
`gateway` that *aren't* safe on a schedule (cert issuance, kernel
cmdline edits, the initial WireGuard keygen). Roles that are entirely
bootstrap-only (`framework`, `jetson`, `vms`) only live under
`playbooks/setup/` and never run from cron at all.

### The runner pattern

To run from cron I needed a service account, so a `gateway_runner` role
creates `wildelab-cron` on the Pi with NOPASSWD sudo and an Ed25519
key. The `base` role (which runs against every rack host) mirrors the
same user onto each box with the gateway's pubkey pre-authorized. End
result: the gateway can SSH as `wildelab-cron@<any rack host>` and
sudo, without any password or human key.

The gateway pulls itself from GitHub before each run. To avoid stashing
the deploy key, I reuse `wildelab-cron`'s SSH key as a read-only Deploy
Key on the repo. One key does two jobs:

```cron
# /etc/cron.d/wildelab-reconcile
0 4 * * * wildelab-cron cd /opt/wildelab-ansible && \
  git pull --ff-only && \
  ansible-playbook -i inventory/hosts.ini -e ansible_user=wildelab-cron \
    --skip-tags setup playbooks/scheduled/reconcile.yml \
    >> /var/log/wildelab-ansible/reconcile.log 2>&1
```

Pushing to `main` is the deploy mechanism. By the next morning the
fleet has converged. There's no agent, no manifest server, no
controller. Just `git pull && ansible-playbook` on a cron timer.

The vault password lives on the gateway as `/opt/wildelab-ansible/.vault_pass`
(mode 0600, gitignored, separate from the repo). Decryption works the
same as on my laptop. The secrets it unlocks are already deployed to
the rack anyway, so the gateway carrying the password adds little
marginal exposure.

### SSH key sync is the canary

One of the older but quietly important roles is `base`: it slurps
`/home/mason/.ssh/authorized_keys` from the gateway and pushes it to
every rack host. The gateway is the canonical store; everywhere else
mirrors. If I want to add a new device to the mesh I add its pubkey on
the gateway and the next reconcile pushes it everywhere. If I add it
on the wrong host, it's gone within 24 hours, which has been a useful
forcing function for keeping the canonical store actually canonical.

## LLMs on two architectures

The fun part. I wanted to run local LLMs without one being "the prod
model". Different backends are good at different sizes, and I wanted
to actually use both.

### Framework Desktop: Vulkan llama.cpp, big models, slow swaps

The Framework Desktop has an AMD Ryzen AI Max+ 395 (Strix Halo) with a
Radeon 8060S iGPU sharing 128 GB of unified LPDDR5x with the CPU.
Vulkan is the universal backend for llama.cpp on AMD, so the box runs a
hand-built `llama-server` with `-DGGML_VULKAN=ON`. The systemd unit
loads the active model from inventory:

```yaml
# inventory/host_vars/framework.yml
llama_active_model: qwen3.6-35b-q4

llama_models:
  - name: qwen3.6-35b-q4
    repo: unsloth/Qwen3.6-35B-A3B-GGUF
    file: Qwen3.6-35B-A3B-UD-Q4_K_M.gguf
    args: '-c 131072 -fa on -ngl 999 --reasoning-budget 2048 ...'
  - name: gemma-4-31b-q4
    ...
  - name: deepseek-v4
    ...
```

Switching models is `llama_active_model: <new>` + `ansible-playbook
framework.yml --skip-tags setup`. All models are pre-downloaded by the
setup path (via the HF CLI installed through `uv`), so a swap is just a
systemd restart and a model load, usually under a minute.

The framework serves at `llama.wildelab.com` via the gateway's nginx
proxy, with the wildcard cert from the VPN section.

### Jetson Orin Nano: CUDA + llama-swap, small fast models

The Jetson is the opposite end: 8GB of unified memory, CUDA capability
8.7 (Ampere). Big models won't fit, but small models run fast. So the
Jetson runs `llama-swap` in front of a CUDA-built `llama-server`:

- `llama-swap` listens on port 8080 and hot-swaps models on demand.
- When a request comes in for model A it spawns `llama-server -m A.gguf`,
  routes the request through, and idles A out after 5 minutes of no use.
- Switching to model B is just another request: A unloads, B loads.

Inventory:

```yaml
# inventory/host_vars/jetson.yml
llama_models:
  - name: qwen3.5-4b
    repo: unsloth/Qwen3.5-4B-GGUF
    file: Qwen3.5-4B-UD-Q6_K_XL.gguf
    args: "-ngl 99 -fa on -c 16384 --jinja"
  - name: gemma-4-e4b
    repo: unsloth/gemma-4-E4B-it-GGUF
    file: gemma-4-E4B-it-UD-Q3_K_XL.gguf
    args: "-ngl 99 -fa on -c 16384 --jinja"
```

The Jetson serves at `llama2.wildelab.com`. Open WebUI on the framework
treats both endpoints as upstream OpenAI-compatible servers, so the
model picker shows everything across both machines. Heavy reasoning
goes to the framework; quick chats while the framework is busy go to
the Jetson.

### Building llama.cpp on the cron path

llama.cpp moves fast and benefits from frequent rebuilds. The clone +
build is *not* tagged `setup`, so the scheduled reconcile pulls and
rebuilds every morning. CMake's incremental build means most days it's
a few seconds; the days a major upstream change lands, the build takes
longer but the rest of the reconcile carries on.

## Patterns worth stealing

If you build something similar, the patterns that turned out to matter
most:

- **Split-DNS over a WG mesh** beats any reverse-proxy-on-public-IP
  setup I've used. Less attack surface, real certs, real names.
- **Two playbook directories** for bootstrap vs reconcile is what kept
  the cron path safe. The shared roles handle both modes; the
  bootstrap-only ones never live in `scheduled/` at all.
- **GitHub deploy key for the runner** is what makes the cron loop a
  real GitOps flow instead of a manual copy.

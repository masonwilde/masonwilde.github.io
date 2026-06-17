---
{
    "title": "Wildelab: A WireGuard-Meshed Homelab",
    "date": "2026-05-29",
    "draft": false,
    "description": "A self-hosted rack reconciled by Ansible from cron, talking to itself over WireGuard, serving local LLMs behind a wildcard cert."
}
---

# Wildelab

Six machines in a rack at home, reachable from anywhere over WireGuard, converging nightly from a single Ansible repository. A Raspberry Pi gateway, a Framework Desktop (Vulkan LLMs), a Jetson Orin Nano (CUDA LLMs), four Pi 5 gameservers, and a $5/mo Vultr VPS outside the NAT.

## Network

### Topology

```
laptop / phone / friends           (10.9.9.0/24)
       |
       | WireGuard
       v
VPS (vpn.wildelab.com)             public IP, 10.9.9.1
       |
       | WireGuard
       v
Pi gateway (10.42.0.1 / 10.9.9.2)
       |
       | ethernet
       v
framework (10.42.0.10)
jetson    (10.42.0.20)
pi5-0..3  (10.42.0.50-53)
```

The gateway bridges the rack subnet (`10.42.0.0/24`) and the WireGuard mesh (`10.9.9.0/24`). Roaming peers reach rack hosts via the VPS's `FORWARD` + `MASQUERADE` rules.

### Split DNS

The gateway runs dnsmasq with a host file for every rack device. The VPS runs a second dnsmasq that forwards `wildelab.com` queries to the gateway:

```conf
# VPS dnsmasq - forward wildelab.com to gateway
server=/wildelab.com/10.9.9.2

# Gateway dnsmasq - apex record
host-record=wildelab.com,10.42.0.1
```

Only `vpn.wildelab.com`, `vps.wildelab.com`, and an `NS` delegation for `acme.wildelab.com` exist in public DNS. All other names resolve only on the mesh.

### Wildcard TLS

DNS-01 via [acme-dns](https://github.com/joohoi/acme-dns) on the VPS, avoiding any public HTTP exposure.

1. Certbot on the gateway requests `*.wildelab.com`.
2. A hook script POSTs the ACME challenge to acme-dns on the VPS.
3. `_acme-challenge.wildelab.com` CNAMEs to a subdomain acme-dns is authoritative for.
4. Let's Encrypt validates, issues the cert.

One CNAME at the registrar, set once. Renewals are automatic via certbot timer.

## Ansible

### Bootstrap vs Scheduled

Two playbook directories, one repository:

- **`playbooks/setup/`** - From-scratch provisioning. Heavy apt installs, WireGuard keygen, cert issuance, kernel cmdline edits. Run manually with gates between stages.
- **`playbooks/scheduled/`** - Cron-driven reconciliation from the gateway. Daily `reconcile.yml` re-applies config templates and SSH key sync. Weekly `apt-update.yml` upgrades host-by-host.

Shared roles work in both modes. `reconcile.yml` imports `base` and `gateway` with `--skip-tags setup` to filter out one-time tasks. Bootstrap-only roles (`framework`, `jetson`, `vms`) never run from cron.

### Cron Runner

A `wildelab-cron` service account on the gateway with NOPASSWD sudo and an Ed25519 key. The `base` role mirrors this user onto every rack host with the gateway's pubkey pre-authorized. The same key doubles as a read-only GitHub Deploy Key.

```cron
# /etc/cron.d/wildelab-reconcile
0 4 * * * wildelab-cron cd /opt/wildelab-ansible && \
  git pull --ff-only && \
  ansible-playbook -i inventory/hosts.ini -e ansible_user=wildelab-cron \
    --skip-tags setup playbooks/scheduled/reconcile.yml \
    >> /var/log/wildelab-ansible/reconcile.log 2>&1
```

Push to `main` is the deploy mechanism. The fleet converges by the next morning.

Vault password lives at `/opt/wildelab-ansible/.vault_pass` (mode 0600, gitignored). The secrets it unlocks are already deployed to the rack, so the gateway carrying it adds minimal exposure.

### SSH Key Sync

The gateway is the canonical store for `authorized_keys`. The `base` role pushes it to every rack host on each reconcile. Keys added on the wrong host are overwritten within 24 hours.

## LLM Serving

### Framework Desktop (Vulkan)

AMD Ryzen AI Max+ 395 with Radeon 8060S iGPU, 128 GB unified LPDDR5x. Runs `llama-server` built with `-DGGML_VULKAN=ON`. Active model is set in inventory:

```yaml
# inventory/host_vars/framework.yml
llama_active_model: qwen3-coder-next-q4

llama_models:
  - name: qwen3-coder-next-q4
    repo: unsloth/Qwen3-Coder-Next-GGUF
    file: Qwen3-Coder-Next-UD-Q4_K_M.gguf
    args: '-c 131072 -fa on -ngl 999 --reasoning-budget 2048 ...'
  - name: qwen3.6-35b-q4
    repo: unsloth/Qwen3.6-35B-A3B-GGUF
    file: Qwen3.6-35B-A3B-UD-Q4_K_M.gguf
    args: '-c 131072 -fa on -ngl 999 ...'
  - name: qwen3.6-35b-bf16  # sharded
    ...
  - name: gemma-4-31b-q4
    ...
  - name: gemma-4-31b-bf16   # sharded
    ...
  - name: gemma-4-e4b        # quick iteration tests
    ...
  - name: deepseek-v4        # ~87GB mixed-precision
    ...
```

Seven models in the catalog, Q4 and BF16 variants for direct quality/speed comparison. Model swap: change `llama_active_model`, run the playbook. Models are pre-downloaded via HF CLI, so swaps are a systemd restart. Serves at `llama.wildelab.com` via nginx with the wildcard cert.

### Jetson Orin Nano (CUDA)

8 GB unified memory, CUDA 8.7 (Ampere). Runs `llama-swap` in front of a CUDA-built `llama-server`:

- Hot-swaps models on demand per request.
- Idles models out after 5 minutes of inactivity.

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

Serves at `llama2.wildelab.com`. Open WebUI on the framework aggregates both endpoints. Heavy reasoning goes to the framework; quick tasks go to the Jetson.

### Builds

llama.cpp clone + build is not tagged `setup`, so the scheduled reconcile rebuilds nightly. CMake incremental builds keep most runs under a few seconds.

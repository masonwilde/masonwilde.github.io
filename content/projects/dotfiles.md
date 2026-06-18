---
{
    "title": "Dotfiles",
    "date": "2026-06-17",
    "description": "GNU Stow-managed dotfiles for Zsh, Neovim, Tmux, and a cross-platform bootstrap script."
}
---

My system configuration, managed with [GNU Stow](https://www.gnu.org/software/stow/) and bootstrappable from a bare machine via a single script.

[GitHub](https://github.com/masonwilde/.dotfiles)

## What It Manages

- **Shell** - Zsh (Antigen, Powerlevel10k) with Bash fallback. Shared config via `.sharedrc` sourced by both. Vi mode enabled.
- **Editor** - Neovim with Lua config, mason.nvim for LSP (rust-analyzer, pyright, clangd, gopls, tsserver), nvim-cmp, fzf-lua, conform.nvim, OneDark theme. Helix and Vim configs as fallbacks.
- **Terminal** - Kitty and Alacritty.
- **Multiplexer** - Tmux with `Ctrl+Space` prefix, TPM, vim-tmux-navigator.
- **WM** - Waybar config for Hyprland.
- **Git** - Global gitignore (excludes `.claude`, `.cursor`, `.copilot` dirs), gitleaks pre-commit hook.

## Structure

Each top-level directory mirrors `$HOME`:

```
dotfiles/
  nvim/.config/nvim/
  zsh/.zshrc
  tmux/.tmux.conf
  git/.gitconfig
  ...
```

`stow -R -t $HOME <package>` symlinks each one into place. The setup script backs up conflicts before overwriting.

## Bootstrap

`from-dust.sh` takes a fresh machine to a working environment:

1. **`get_packages.sh`** - Detects OS (macOS/Arch/Ubuntu), installs packages via the native package manager.
2. **`install_externals.sh`** - Installs TPM and other tools that don't come from package managers.
3. **`setup.sh`** - Stows all configs with dry-run conflict detection.

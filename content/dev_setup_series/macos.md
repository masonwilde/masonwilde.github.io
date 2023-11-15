+++
title = 'Dev Setup Series: MacOS 14'
date = 2023-11-09T08:09:09-07:00
draft = true
description = 'My setup steps for a new MacOS 14 device.'
+++

# MacOS 14 Developer Setup

* [Robin Wieruch's Dev Guide](https://www.robinwieruch.de/mac-setup-web-development/)
* [Sourabhbajaj's Guide](https://sourabhbajaj.com/mac-setup/)
* [CJR's Dev.to Guide](https://dev.to/w3cj/setting-up-a-mac-for-development-3g4c)

## Before you start
Read before you execute. I tried to vet all the scripts and tools run in here, but you should always read .sh or other executable files before running them, **especially** if they require `sudo`.

## TL;DR (Things I think are non-negotiable)
* OS Settings (From [Robin Wieruch](https://www.robinwieruch.de/mac-setup-web-development/) and [Sourabhbajaj's](https://sourabhbajaj.com/mac-setup/) Guides)
    ```bash
    # take screenshots as jpg (usually smaller size) and not png
    defaults write com.apple.screencapture type jpg

    # do not open previous previewed files (e.g. PDFs) when opening a new one
    defaults write com.apple.Preview ApplePersistenceIgnoreState YES

    # show Library folder
    chflags nohidden ~/Library

    # show hidden files
    defaults write com.apple.finder AppleShowAllFiles YES

    # show path bar
    defaults write com.apple.finder ShowPathbar -bool true

    # show status bar
    defaults write com.apple.finder ShowStatusBar -bool true

    killall Finder;
    
    # remove workspace auto-switching
    defaults write com.apple.dock workspaces-auto-swoosh -bool NO
    
    killall Dock # Restart the Dock process

    # custom screenshot location
    # CMD+SHIFT+4 for region or CMD+SHIFT+3 for fullscreen
    mkdir -p ~/Pictures/screenshots
    defaults write com.apple.screencapture location ~/Pictures/screenshots
    
    killall SystemUIServer
    ```
* System Settings
    * Notifications > **ALL OFF**
    * General > About > Name > Enter a `$DEVICE_NAME` for your device
        * Open terminal.app
            ```bash
            export DEVICE_NAME='yourdevicename'
            sudo scutil --set HostName $DEVICE_NAME
            sudo scutil --set LocalHostName $DEVICE_NAME
            sudo scutil --set ComputerName $DEVICE_NAME
            ```
    * Security and Privacy
        * Turn on FileVault
    * Storage
        * Remove unwanted bloat (e.g. GarageBand)
    * Siri & Spotlight
        * Set Spotlight search categories to your liking (I prefer only Applications and System Settings enabled)
    * Desktop & Dock
        * Desktop
            * Click wallpaper to reveal desktop > Only in Stage Manager
        * Hot Corner Shortcuts > **ALL OFF**
* Finder Settings
    * Sidebar
        * Enable home directory
        * (optional) Go into finder, drag user>Library to favorites.
    * Advanced
        * Enable `Show all filename extensions`
* [Homebrew](https://brew.sh/) (Packages and Apps)
    * Install Homebrew (in Terminal)
        ```bash
        /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
        ```
    * Core Packages (In Terminal)
        ```bash
        brew install wget \
        vim \
        exa \
        git \
        stow \
        fzf \
        tree \
        ack
        ```
    * Core Apps (In Terminal)
        ```bash
        brew install --cask iterm2 \
        rectangle \
        vlc \
        imageoptim \
        maccy \
        ```
* iTerm (Your new primary terminal experience)
    * Shell settings
        * switch to Bash `chsh -s /bin/bash`
        * switch to Zsh `chsh -s /bin/zsh`
    * Settings > Profiles > General > Working Directory
        * Enable Advanced Configuration
        * Edit
            * Set these how you'd like. I prefer things to open with my previous session directory by default.
    * Other Setup
        * I'll leave this alone as pretty much everything is subjective, but I recommend using dotfiles and stow at the very least ([Guide on Venthur.de](https://venthur.de/2021-12-19-managing-dotfiles-with-stow.html)).
        * If you stick with Zsh, I highly recommend Zsh+Antigen+Oh-My-Zsh
            * [COMING SOON: My Zsh Setup](../dev_setup_series/zsh.md)
* Git
    * Configuration
        ```bash
        git config --global user.name "Your Name"
        git config --global user.email "you@your-domain.com"
        git config --global init.defaultBranch main
        ```
    * Global .gitignore
        ```bash
        curl https://raw.githubusercontent.com/github/gitignore/master/Global/macOS.gitignore -o ~/.gitignore
        git config --global core.excludesfile ~/.gitignore
        ```
    * Using Github
        * (optional) Github CLI (I don't use it because I fear change)
            ```bash
            brew install gh
            ```
        * Auth
            ```bash
            mkdir ~/.ssh
            ssh-keygen -t ed25519 -C "github"
            cat <<EOT >> ~/.ssh/config
            Host *
                AddKeysToAgent yes
                UseKeychain yes
                IdentityFile ~/.ssh/id_ed25519
            EOT
            ssh-add --apple-use-keychain ~/.ssh/id_ed25519
            ```
            * Manual
                ```bash
                cat ~/.ssh/github.pub | pbcopy
                ```
                Add at https://github.com/settings/keys
            * Github CLI
                ```bash
                gh auth login
                gh ssh-key add ~/.ssh/id_ed25519.pub -t github
                ```

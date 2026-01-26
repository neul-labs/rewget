# Installation

rewget is distributed as a single binary per platform. After installation, you can optionally configure your system to use rewget as the default wget.

## Quick Install

### Linux / macOS

```bash
curl -fsSL https://rewget.dev/install.sh | sh
```

### Windows

```powershell
irm https://rewget.dev/install.ps1 | iex
```

## Manual Installation

### Download

| Platform | Architecture | Download |
|----------|--------------|----------|
| Linux | x86_64 | [rewget-linux-x86_64.tar.gz](https://github.com/user/rewget/releases/latest) |
| Linux | arm64 | [rewget-linux-arm64.tar.gz](https://github.com/user/rewget/releases/latest) |
| macOS | x86_64 (Intel) | [rewget-darwin-x86_64.tar.gz](https://github.com/user/rewget/releases/latest) |
| macOS | arm64 (Apple Silicon) | [rewget-darwin-arm64.tar.gz](https://github.com/user/rewget/releases/latest) |
| Windows | x86_64 | [rewget-windows-x86_64.zip](https://github.com/user/rewget/releases/latest) |

### Extract and Install

**Linux / macOS:**
```bash
tar -xzf rewget-*.tar.gz
sudo mv rewget /usr/local/bin/
sudo mv rewgetd /usr/local/bin/
```

**Windows:**
```powershell
Expand-Archive rewget-windows-x86_64.zip -DestinationPath C:\Program Files\rewget
# Add to PATH via System Properties > Environment Variables
```

## Replacing wget with rewget

rewget is designed to be a drop-in replacement for wget. You can configure your system to use rewget whenever you type `wget`.

### Option 1: Shell Alias (Recommended)

The simplest approach. Add an alias to your shell configuration.

**Bash (~/.bashrc):**
```bash
alias wget='rewget'
```

**Zsh (~/.zshrc):**
```zsh
alias wget='rewget'
```

**Fish (~/.config/fish/config.fish):**
```fish
alias wget='rewget'
```

**PowerShell ($PROFILE):**
```powershell
Set-Alias -Name wget -Value rewget
```

After adding, reload your shell:
```bash
source ~/.bashrc  # or ~/.zshrc
```

### Option 2: Symlink (System-wide)

Create a symlink so all users and scripts use rewget.

**Linux / macOS:**
```bash
# Backup original wget (if installed)
sudo mv /usr/bin/wget /usr/bin/wget.orig

# Create symlink
sudo ln -s /usr/local/bin/rewget /usr/bin/wget
```

**To restore original wget:**
```bash
sudo rm /usr/bin/wget
sudo mv /usr/bin/wget.orig /usr/bin/wget
```

### Option 3: PATH Priority

Place rewget earlier in your PATH than the system wget.

**Linux / macOS:**
```bash
# Add to ~/.bashrc or ~/.zshrc
export PATH="$HOME/.local/bin:$PATH"

# Then symlink rewget
ln -s /usr/local/bin/rewget ~/.local/bin/wget
```

### Option 4: Wrapper Script

For more control, create a wrapper script.

**Linux / macOS (/usr/local/bin/wget):**
```bash
#!/bin/bash
exec rewget "$@"
```

```bash
sudo chmod +x /usr/local/bin/wget
```

## Platform-Specific Notes

### Linux

**Debian/Ubuntu:**
```bash
# If wget is installed via apt, the symlink approach may conflict
# Prefer the alias method or PATH priority

# Check current wget
which wget
# /usr/bin/wget

# rewget installs to
# /usr/local/bin/rewget
```

**Arch Linux:**
```bash
# AUR package available (once published)
yay -S rewget
```

### macOS

**Homebrew users:**
```bash
# Homebrew wget is at /opt/homebrew/bin/wget (Apple Silicon)
# or /usr/local/bin/wget (Intel)

# Option A: Alias (doesn't affect Homebrew)
alias wget='rewget'

# Option B: Unlink Homebrew wget
brew unlink wget
ln -s /usr/local/bin/rewget /usr/local/bin/wget
```

**Without Homebrew:**
```bash
# macOS doesn't include wget by default
# Just install rewget and alias it
alias wget='rewget'
```

### Windows

**PowerShell:**
```powershell
# Add to $PROFILE
function wget { rewget.exe @args }

# Or use Set-Alias
Set-Alias -Name wget -Value rewget.exe
```

**CMD:**
```batch
:: Create wget.bat in a PATH directory
@echo off
rewget.exe %*
```

**Git Bash / MSYS2:**
```bash
# Same as Linux
alias wget='rewget'
```

**WSL:**
```bash
# WSL uses Linux instructions
# Install the Linux binary, not Windows
alias wget='rewget'
```

## Verifying Installation

After installation and aliasing:

```bash
# Check version
rewget --version

# Verify alias works
wget --version
# Should show rewget version, not GNU wget

# Test fallback
rewget --rewget-debug https://example.com/
```

## Uninstallation

### Remove Binary

**Linux / macOS:**
```bash
sudo rm /usr/local/bin/rewget
sudo rm /usr/local/bin/rewgetd
rm -rf ~/.config/rewget
rm -rf ~/.local/share/rewget
rm -rf ~/.cache/rewget
```

**Windows:**
```powershell
Remove-Item "C:\Program Files\rewget" -Recurse
Remove-Item "$env:APPDATA\rewget" -Recurse
```

### Remove Alias

Remove the alias line from your shell configuration file.

### Restore Original wget

If you used the symlink approach:
```bash
sudo rm /usr/bin/wget
sudo mv /usr/bin/wget.orig /usr/bin/wget
```

## Configuration

rewget uses these configuration locations:

| Platform | Config | Data |
|----------|--------|------|
| Linux | `~/.config/rewget/` | `~/.local/share/rewget/` |
| macOS | `~/Library/Application Support/rewget/` | Same |
| Windows | `%APPDATA%\rewget\` | Same |

### Config File

`config.toml`:
```toml
[fallback]
enabled = true
codes = [403, 429, 503]
body_detection = true

[daemon]
idle_timeout = 300  # seconds
browser_pool_size = 2

[profiles]
default = "chrome"
auto_update = true
```

## Troubleshooting

### "rewget: command not found"

Ensure `/usr/local/bin` is in your PATH:
```bash
echo $PATH | tr ':' '\n' | grep local
```

### Alias not working in scripts

Shell aliases don't work in non-interactive scripts. Use:
- The symlink approach, or
- Call `rewget` directly in scripts

### Homebrew conflicts

If Homebrew wget shadows rewget:
```bash
# Check which wget is being used
which wget
type wget

# Ensure alias is set in ~/.zshrc (not ~/.zprofile)
# And reload: source ~/.zshrc
```

### Windows PATH issues

Ensure rewget directory is in PATH:
```powershell
$env:PATH -split ';' | Select-String rewget
```

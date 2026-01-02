# Installation

rwget is distributed as a single binary per platform. After installation, you can optionally configure your system to use rwget as the default wget.

## Quick Install

### Linux / macOS

```bash
curl -fsSL https://rwget.dev/install.sh | sh
```

### Windows

```powershell
irm https://rwget.dev/install.ps1 | iex
```

## Manual Installation

### Download

| Platform | Architecture | Download |
|----------|--------------|----------|
| Linux | x86_64 | [rwget-linux-x86_64.tar.gz](https://github.com/user/rwget/releases/latest) |
| Linux | arm64 | [rwget-linux-arm64.tar.gz](https://github.com/user/rwget/releases/latest) |
| macOS | x86_64 (Intel) | [rwget-darwin-x86_64.tar.gz](https://github.com/user/rwget/releases/latest) |
| macOS | arm64 (Apple Silicon) | [rwget-darwin-arm64.tar.gz](https://github.com/user/rwget/releases/latest) |
| Windows | x86_64 | [rwget-windows-x86_64.zip](https://github.com/user/rwget/releases/latest) |

### Extract and Install

**Linux / macOS:**
```bash
tar -xzf rwget-*.tar.gz
sudo mv rwget /usr/local/bin/
sudo mv rwgetd /usr/local/bin/
```

**Windows:**
```powershell
Expand-Archive rwget-windows-x86_64.zip -DestinationPath C:\Program Files\rwget
# Add to PATH via System Properties > Environment Variables
```

## Replacing wget with rwget

rwget is designed to be a drop-in replacement for wget. You can configure your system to use rwget whenever you type `wget`.

### Option 1: Shell Alias (Recommended)

The simplest approach. Add an alias to your shell configuration.

**Bash (~/.bashrc):**
```bash
alias wget='rwget'
```

**Zsh (~/.zshrc):**
```zsh
alias wget='rwget'
```

**Fish (~/.config/fish/config.fish):**
```fish
alias wget='rwget'
```

**PowerShell ($PROFILE):**
```powershell
Set-Alias -Name wget -Value rwget
```

After adding, reload your shell:
```bash
source ~/.bashrc  # or ~/.zshrc
```

### Option 2: Symlink (System-wide)

Create a symlink so all users and scripts use rwget.

**Linux / macOS:**
```bash
# Backup original wget (if installed)
sudo mv /usr/bin/wget /usr/bin/wget.orig

# Create symlink
sudo ln -s /usr/local/bin/rwget /usr/bin/wget
```

**To restore original wget:**
```bash
sudo rm /usr/bin/wget
sudo mv /usr/bin/wget.orig /usr/bin/wget
```

### Option 3: PATH Priority

Place rwget earlier in your PATH than the system wget.

**Linux / macOS:**
```bash
# Add to ~/.bashrc or ~/.zshrc
export PATH="$HOME/.local/bin:$PATH"

# Then symlink rwget
ln -s /usr/local/bin/rwget ~/.local/bin/wget
```

### Option 4: Wrapper Script

For more control, create a wrapper script.

**Linux / macOS (/usr/local/bin/wget):**
```bash
#!/bin/bash
exec rwget "$@"
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

# rwget installs to
# /usr/local/bin/rwget
```

**Arch Linux:**
```bash
# AUR package available (once published)
yay -S rwget
```

### macOS

**Homebrew users:**
```bash
# Homebrew wget is at /opt/homebrew/bin/wget (Apple Silicon)
# or /usr/local/bin/wget (Intel)

# Option A: Alias (doesn't affect Homebrew)
alias wget='rwget'

# Option B: Unlink Homebrew wget
brew unlink wget
ln -s /usr/local/bin/rwget /usr/local/bin/wget
```

**Without Homebrew:**
```bash
# macOS doesn't include wget by default
# Just install rwget and alias it
alias wget='rwget'
```

### Windows

**PowerShell:**
```powershell
# Add to $PROFILE
function wget { rwget.exe @args }

# Or use Set-Alias
Set-Alias -Name wget -Value rwget.exe
```

**CMD:**
```batch
:: Create wget.bat in a PATH directory
@echo off
rwget.exe %*
```

**Git Bash / MSYS2:**
```bash
# Same as Linux
alias wget='rwget'
```

**WSL:**
```bash
# WSL uses Linux instructions
# Install the Linux binary, not Windows
alias wget='rwget'
```

## Verifying Installation

After installation and aliasing:

```bash
# Check version
rwget --version

# Verify alias works
wget --version
# Should show rwget version, not GNU wget

# Test fallback
rwget --rwget-debug https://example.com/
```

## Uninstallation

### Remove Binary

**Linux / macOS:**
```bash
sudo rm /usr/local/bin/rwget
sudo rm /usr/local/bin/rwgetd
rm -rf ~/.config/rwget
rm -rf ~/.local/share/rwget
rm -rf ~/.cache/rwget
```

**Windows:**
```powershell
Remove-Item "C:\Program Files\rwget" -Recurse
Remove-Item "$env:APPDATA\rwget" -Recurse
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

rwget uses these configuration locations:

| Platform | Config | Data |
|----------|--------|------|
| Linux | `~/.config/rwget/` | `~/.local/share/rwget/` |
| macOS | `~/Library/Application Support/rwget/` | Same |
| Windows | `%APPDATA%\rwget\` | Same |

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

### "rwget: command not found"

Ensure `/usr/local/bin` is in your PATH:
```bash
echo $PATH | tr ':' '\n' | grep local
```

### Alias not working in scripts

Shell aliases don't work in non-interactive scripts. Use:
- The symlink approach, or
- Call `rwget` directly in scripts

### Homebrew conflicts

If Homebrew wget shadows rwget:
```bash
# Check which wget is being used
which wget
type wget

# Ensure alias is set in ~/.zshrc (not ~/.zprofile)
# And reload: source ~/.zshrc
```

### Windows PATH issues

Ensure rwget directory is in PATH:
```powershell
$env:PATH -split ';' | Select-String rwget
```

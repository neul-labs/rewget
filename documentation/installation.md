# Installation

rwget supports Linux, macOS, and Windows. Choose your preferred installation method below.

## Package Managers

### Homebrew (macOS / Linux)

The easiest way to install rwget on macOS or Linux:

```bash
brew install dipankardas011/tap/rwget
```

This installs both `rwget` and `rwgetd` binaries, along with shell completions and man pages.

### Cargo (All Platforms)

If you have Rust installed:

```bash
cargo install rwget
```

## Install Script

### Linux / macOS

```bash
curl -fsSL https://rwget.dev/install.sh | sh
```

Or with custom install directory:

```bash
RWGET_INSTALL_DIR=/usr/local/bin curl -fsSL https://rwget.dev/install.sh | sh
```

### Windows (PowerShell)

```powershell
irm https://rwget.dev/install.ps1 | iex
```

## Manual Installation

### Download Pre-built Binaries

Download the appropriate archive for your platform from the [releases page](https://github.com/dipankardas011/rwget/releases):

| Platform | Architecture | Download |
|----------|--------------|----------|
| Linux | x86_64 | `rwget-x86_64-unknown-linux-gnu.tar.gz` |
| Linux | ARM64 | `rwget-aarch64-unknown-linux-gnu.tar.gz` |
| macOS | x86_64 (Intel) | `rwget-x86_64-apple-darwin.tar.gz` |
| macOS | ARM64 (Apple Silicon) | `rwget-aarch64-apple-darwin.tar.gz` |
| Windows | x86_64 | `rwget-x86_64-pc-windows-gnu.zip` |

Extract and install:

=== "Linux / macOS"

    ```bash
    tar -xzf rwget-*.tar.gz
    sudo install -m 755 rwget /usr/local/bin/
    sudo install -m 755 rwgetd /usr/local/bin/
    ```

=== "Windows"

    Extract the zip file and add the directory to your PATH.

### Build from Source

Requirements:

- Rust 1.75 or later
- C compiler (for native dependencies)
- CMake (for some dependencies)

```bash
# Clone the repository
git clone https://github.com/dipankardas011/rwget
cd rwget

# Build release binaries
cargo build --release

# Install to ~/.cargo/bin (or copy manually)
cargo install --path crates/rwget
```

## Post-Installation Setup

### Verify Installation

```bash
rwget --rwget-version
# rwget 1.0.0

rwget --rwget-help
```

### Shell Completions

Enable tab completion for rwget commands:

=== "Bash"

    Add to `~/.bashrc`:

    ```bash
    eval "$(rwget --rwget-completions=bash)"
    ```

=== "Zsh"

    Add to `~/.zshrc`:

    ```bash
    eval "$(rwget --rwget-completions=zsh)"
    ```

=== "Fish"

    Add to `~/.config/fish/config.fish`:

    ```fish
    rwget --rwget-completions=fish | source
    ```

=== "PowerShell"

    Add to your PowerShell profile:

    ```powershell
    rwget --rwget-completions=powershell | Out-String | Invoke-Expression
    ```

### Replace wget (Optional)

To use rwget as your default wget, add an alias:

=== "Bash / Zsh"

    ```bash
    alias wget='rwget'
    ```

=== "Fish"

    ```fish
    alias wget='rwget'
    ```

=== "PowerShell"

    ```powershell
    Set-Alias -Name wget -Value rwget
    ```

### Pre-download Chromium (Optional)

Stage 3 (JavaScript preflight) requires Chromium. It downloads automatically on first use, but you can pre-download it:

```bash
rwget --rwget-download-chromium
```

This downloads Chrome for Testing (~150MB) to `~/.local/share/rwget/chromium/`.

## Dependencies

rwget requires `wget` or `wget2` to be installed on your system:

=== "Debian / Ubuntu"

    ```bash
    sudo apt install wget
    ```

=== "Fedora / RHEL"

    ```bash
    sudo dnf install wget
    ```

=== "macOS"

    ```bash
    brew install wget
    ```

=== "Windows"

    wget is typically available via Git Bash, WSL, or can be downloaded from [GNU wget](https://www.gnu.org/software/wget/).

## Uninstall

=== "Homebrew"

    ```bash
    brew uninstall rwget
    ```

=== "Manual"

    ```bash
    rm /usr/local/bin/rwget /usr/local/bin/rwgetd
    rm -rf ~/.local/share/rwget  # Remove Chromium and profiles
    rm -rf ~/.cache/rwget        # Remove cache
    ```

## Next Steps

- [Quick Start](quickstart.md) - Get started with rwget
- [Usage Guide](usage.md) - Learn about all features
- [Configuration](configuration.md) - Customize rwget behavior

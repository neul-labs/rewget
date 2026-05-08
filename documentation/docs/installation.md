# Installation

rewget supports Linux and macOS. Choose your preferred installation method below.

## Package Managers

### Homebrew (macOS / Linux)

The easiest way to install rewget on macOS or Linux:

```bash
brew install neul-labs/tap/rewget
```

This installs both `rewget` and `rewgetd` binaries, along with shell completions and man pages.

### npm (All Platforms)

If you have Node.js installed:

```bash
npm install -g rewget
```

### PyPI (All Platforms)

If you have Python installed:

```bash
pip install rewget
```

### Cargo (All Platforms)

If you have Rust installed:

```bash
cargo install rewget
```

## Install Script

### Linux / macOS

```bash
curl -fsSL https://rewget.dev/install.sh | sh
```

Or with custom install directory:

```bash
RWGET_INSTALL_DIR=/usr/local/bin curl -fsSL https://rewget.dev/install.sh | sh
```

## Manual Installation

### Download Pre-built Binaries

Download the appropriate archive for your platform from the [releases page](https://github.com/neul-labs/rewget/releases):

| Platform | Architecture | Download |
|----------|--------------|----------|
| Linux | x86_64 | `rewget-x86_64-unknown-linux-gnu.tar.gz` |
| Linux | ARM64 | `rewget-aarch64-unknown-linux-gnu.tar.gz` |
| macOS | x86_64 (Intel) | `rewget-x86_64-apple-darwin.tar.gz` |
| macOS | ARM64 (Apple Silicon) | `rewget-aarch64-apple-darwin.tar.gz` |

Extract and install:

```bash
tar -xzf rewget-*.tar.gz
sudo install -m 755 rewget /usr/local/bin/
sudo install -m 755 rewgetd /usr/local/bin/
```

### Build from Source

Requirements:

- Rust 1.75 or later
- C compiler (for native dependencies)
- CMake (for some dependencies)

```bash
# Clone the repository
git clone https://github.com/neul-labs/rewget
cd rewget

# Build release binaries
cargo build --release

# Install to ~/.cargo/bin (or copy manually)
cargo install --path crates/rewget
```

## Post-Installation Setup

### Verify Installation

```bash
rewget --rewget-version
# rewget 1.0.1

rewget --rewget-help
```

### Shell Completions

Enable tab completion for rewget commands:

=== "Bash"

    Add to `~/.bashrc`:

    ```bash
    eval "$(rewget --rewget-completions=bash)"
    ```

=== "Zsh"

    Add to `~/.zshrc`:

    ```bash
    eval "$(rewget --rewget-completions=zsh)"
    ```

=== "Fish"

    Add to `~/.config/fish/config.fish`:

    ```fish
    rewget --rewget-completions=fish | source
    ```


### Replace wget (Optional)

To use rewget as your default wget, add an alias:

=== "Bash / Zsh"

    ```bash
    alias wget='rewget'
    ```

=== "Fish"

    ```fish
    alias wget='rewget'
    ```


### Pre-download Chromium (Optional)

Stage 3 (JavaScript preflight) requires Chromium. It downloads automatically on first use, but you can pre-download it:

```bash
rewget --rewget-download-chromium
```

This downloads Chrome for Testing (~150MB) to `~/.local/share/rewget/chromium/`.

## Dependencies

rewget requires `wget` or `wget2` to be installed on your system:

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

## Uninstall

=== "Homebrew"

    ```bash
    brew uninstall rewget
    ```

=== "Manual"

    ```bash
    rm /usr/local/bin/rewget /usr/local/bin/rewgetd
    rm -rf ~/.local/share/rewget  # Remove Chromium and profiles
    rm -rf ~/.cache/rewget        # Remove cache
    ```

## Next Steps

- [Quick Start](quickstart.md) - Get started with rewget
- [Usage Guide](usage.md) - Learn about all features
- [Configuration](configuration.md) - Customize rewget behavior

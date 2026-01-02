# Contributing

Thank you for your interest in contributing to rwget!

## Getting Started

### Prerequisites

- Rust 1.75 or later
- Git
- wget (for testing)

### Clone and Build

```bash
git clone https://github.com/dipankardas011/rwget
cd rwget
cargo build
```

### Run Tests

```bash
cargo test
```

### Run with Debug Output

```bash
cargo run -- --rwget-debug https://example.com/
```

## Project Structure

```
rwget/
├── crates/
│   ├── rwget/           # CLI binary
│   ├── rwgetd/          # Daemon binary
│   └── rwget-core/      # Shared library
├── documentation/       # MkDocs documentation
├── Formula/            # Homebrew formula
└── scripts/            # Build scripts
```

## How to Contribute

### Reporting Bugs

1. Check [existing issues](https://github.com/dipankardas011/rwget/issues) first
2. Create a new issue with:
   - rwget version (`rwget --rwget-version`)
   - Operating system and version
   - Steps to reproduce
   - Debug output (`--rwget-debug`)
   - Expected vs actual behavior

### Suggesting Features

1. Open a [new issue](https://github.com/dipankardas011/rwget/issues/new)
2. Describe the feature and use case
3. Explain why it would be useful

### Submitting Code

1. Fork the repository
2. Create a feature branch:
   ```bash
   git checkout -b feature/my-feature
   ```
3. Make your changes
4. Run tests:
   ```bash
   cargo test
   cargo clippy
   cargo fmt --check
   ```
5. Commit with a clear message:
   ```bash
   git commit -m "Add feature: description"
   ```
6. Push and create a Pull Request

## Code Style

### Rust Style

- Follow the [Rust Style Guide](https://doc.rust-lang.org/style-guide/)
- Use `cargo fmt` before committing
- Run `cargo clippy` and address warnings
- Add tests for new functionality

### Commit Messages

Use clear, descriptive commit messages:

```
Add Stage 2 timeout configuration

- Add --rwget-timeout-stage2 flag
- Default to 15 seconds
- Update documentation
```

### Documentation

- Update docs when adding features
- Include doc comments for public APIs
- Keep README.md in sync with changes

## Development Workflow

### Adding a New CLI Flag

1. Add to `crates/rwget/src/args.rs`:
   ```rust
   "my-flag" => {
       let v = value.ok_or_else(|| anyhow!("--rwget-my-flag requires a value"))?;
       config.my_setting = v.parse()?;
   }
   ```

2. Add to `crates/rwget-core/src/config.rs`:
   ```rust
   pub struct Config {
       pub my_setting: String,
   }
   ```

3. Add to `crates/rwget/src/cli.rs` for completions:
   ```rust
   .arg(
       Arg::new("my-flag")
           .long("rwget-my-flag")
           .value_name("VALUE")
           .help("Description of my flag")
   )
   ```

4. Update documentation

### Adding a Detection Pattern

Edit `crates/rwget-core/src/detection.rs`:

```rust
pub const BLOCK_PATTERNS: &[&str] = &[
    "existing-pattern",
    "your-new-pattern",
];
```

### Adding a Browser Profile

1. Create profile JSON following the format in `profiles.md`
2. Add to `crates/rwget-core/src/profile.rs`:
   ```rust
   fn default_profiles() -> Vec<Profile> {
       vec![
           // existing profiles...
           Profile { name: "my_browser", ... },
       ]
   }
   ```

### Testing Against Real Sites

For development, you can test against sites known to have bot protection:

```bash
# Test impersonation
cargo run -- --rwget-debug https://nowsecure.nl/

# Test different profiles
cargo run -- --rwget-profile=firefox_136 --rwget-debug https://example.com/
```

## Areas for Contribution

### High Priority

- [ ] Browser profile capture automation
- [ ] Connection pooling in Stage 2
- [ ] Performance benchmarks
- [ ] Integration tests for fallback scenarios

### Medium Priority

- [ ] Additional detection patterns
- [ ] Windows installer (winget/chocolatey)
- [ ] Configuration file support
- [ ] Proxy support improvements

### Documentation

- [ ] More examples in user guide
- [ ] Video tutorials
- [ ] Translation to other languages

### Testing

- [ ] Golden test suite for wget compatibility
- [ ] Fuzzing for argument parser
- [ ] Performance regression tests

## Release Process

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Create git tag: `git tag v1.0.1`
4. Push: `git push origin v1.0.1`
5. GitHub Actions builds and publishes release

## Code of Conduct

- Be respectful and inclusive
- Focus on constructive feedback
- Help newcomers feel welcome
- Report unacceptable behavior to maintainers

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

## Getting Help

- **Questions**: Open a [discussion](https://github.com/dipankardas011/rwget/discussions)
- **Bugs**: Open an [issue](https://github.com/dipankardas011/rwget/issues)
- **Security**: Email security@rwget.dev

Thank you for contributing to rwget!

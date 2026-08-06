# scat

A small dependency-free bat-like file viewer written in Rust.

## Features

- **Syntax highlighting** for 12+ languages
- **Zero dependencies** — everything is hand-written
- **Pager support** via `$PAGER` (defaults to `less -R`)
- **Line numbers** with configurable visibility

## Installation

```bash
cargo install scat
```

## Usage

```bash
# View a file with syntax highlighting
scat src/main.rs

# View with line numbers (default)
scat --line-numbers src/main.rs

# Force a specific language
scat --language rust src/main.txt

# Disable color output
scat --color never src/main.rs
```

## Supported Languages

| Language   | Extensions           |
|------------|----------------------|
| Rust       | `.rs`                |
| JavaScript | `.js`, `.mjs`, `.cjs`|
| TypeScript | `.ts`                |
| Go         | `.go`                |
| Python     | `.py`                |
| JSON       | `.json`              |
| Markdown   | `.md`                |

## Example

```rust
fn main() {
    println!("Hello, world!");
}
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Submit a pull request

> **Note:** This project intentionally avoids external dependencies.

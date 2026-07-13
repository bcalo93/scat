# mybat

`mybat` is a small `bat`-like terminal file viewer written in Rust.

The current goal is to stay dependency-free while keeping the code modular
enough to replace the hand-written highlighter later.

## Usage

Run it locally before installing:

```bash
cargo run -- src/main.rs
cargo run -- --no-line-numbers src/main.rs
cargo run -- --color src/main.rs
cargo run -- --pager src/main.rs
cargo run -- --language tsx -
cat Cargo.toml | cargo run --
```

Install it globally:

```bash
cargo install --path .
```

Then use:

```bash
mybat src/main.rs
mybat --no-line-numbers src/main.rs
cat file.json | mybat
cat component.txt | mybat --language tsx -
```

## Options

```text
-l, --language <lang>   Force language highlighting
-n, --line-numbers      Show line numbers (default)
    --no-line-numbers   Hide line numbers
    --color             Force ANSI colors
    --plain             Disable colors and highlighting
    --no-color          Disable colors and highlighting
    --pager             Send output to $PAGER or less -R
-h, --help              Show help
```

By default, color is enabled only when stdout is a terminal. This avoids writing
ANSI escapes when redirecting output to a file or piping to another command.
`--color` forces ANSI colors, and `NO_COLOR=1` disables ANSI output.

`--pager` sends rendered output to `$PAGER`. If `$PAGER` is not set, `mybat`
uses `less -R`. The pager command is split on whitespace and is not executed
through a shell.

## Supported Languages

Detection is extension-based for files and intentionally simple:

- JSON
- JavaScript
- TypeScript
- JSX
- TSX
- Go
- Rust
- Swift
- Kotlin
- Java

For stdin, `mybat` infers JSON when the content starts with `{` or `[`.
Use `--language <lang>` for anything else.

## Architecture

- `args`: manual CLI parsing with standard library types.
- `input`: file and stdin reading with explicit user-facing errors.
- `language`: language detection by extension, name, or lightweight content inference.
- `highlight`: ANSI syntax highlighting behind the `SyntaxHighlighter` trait.
- `render`: line numbering and output assembly.
- `render_to_writer`: streaming output writer used by the CLI.
- `ansi`: raw ANSI escape helpers.

The default highlighter is `SimpleHighlighter`. It is heuristic and stateful
enough to handle multiline block comments, but it is not a parser.

## Design Tradeoffs

What this gains by avoiding dependencies:

- Fast builds.
- Small maintenance surface.
- Easy installation with only Cargo and the Rust toolchain.
- Code that is straightforward to inspect and change.

What this gives up for now:

- No semantic syntax parsing.
- No themes.
- Limited JSX/TSX understanding.
- No terminal width or wrapping logic.

## Roadmap

- Add a streaming input path for large files.
- Add snapshot-style tests for rendered ANSI output.
- Add a `SyntaxHighlighter` implementation backed by `syntect` or `tree-sitter`
  if higher-fidelity highlighting becomes more important than zero dependencies.

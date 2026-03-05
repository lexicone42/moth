# moth

Render markdown on the CLI, with pizzazz.

A terminal markdown renderer written in Rust, inspired by [glow](https://github.com/charmbracelet/glow).

## Features

- Syntax-highlighted code blocks with box-drawing borders
- Colored headings (h1–h6), bold, italic, strikethrough
- Inline code with background highlighting
- Box-drawn tables with header separators
- Blockquotes with vertical bar
- Ordered and unordered lists with nesting
- Links, images, horizontal rules
- Word wrapping to terminal width or custom width
- Dark and light themes
- Pager support (`less -r` or `$PAGER`)
- Stdin support for piping

## Install

```bash
cargo install --path .
```

## Usage

```bash
# Render a file
moth README.md

# Pipe from stdin
cat doc.md | moth -

# Custom width
moth -w 60 README.md

# Light theme
moth -s light README.md

# Through a pager
moth -p README.md
```

## Options

```
Usage: moth [OPTIONS] [FILE]

Arguments:
  [FILE]  Markdown file to render (use "-" for stdin)

Options:
  -w, --width <WIDTH>  Word wrap at specified width (0 = terminal width) [default: 0]
  -p, --pager          Pipe output through a pager
  -s, --style <STYLE>  Style to use (dark, light) [default: dark]
  -h, --help           Print help
  -V, --version        Print version
```

## Acknowledgments

Inspired by [glow](https://github.com/charmbracelet/glow) by [Charmbracelet](https://charm.sh).

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

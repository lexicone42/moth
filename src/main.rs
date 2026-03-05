mod render;
mod style;

use std::io::{self, IsTerminal, Read};
use std::process::{Command, Stdio};

use clap::Parser;

/// A terminal markdown renderer
#[derive(Parser)]
#[command(name = "moth", version, about = "Render markdown on the CLI, with pizzazz")]
struct Cli {
    /// Markdown files to render (use "-" for stdin)
    files: Vec<String>,

    /// Word wrap at specified width (0 = terminal width)
    #[arg(short, long, default_value_t = 0)]
    width: usize,

    /// Pipe output through a pager
    #[arg(short, long)]
    pager: bool,

    /// Style to use (dark, light)
    #[arg(short, long, default_value = "dark")]
    style: String,
}

fn main() {
    let cli = Cli::parse();

    let wrap_width = if cli.width == 0 {
        terminal_size::terminal_size()
            .map(|(w, _)| w.0 as usize)
            .unwrap_or(80)
    } else {
        cli.width
    };

    let theme = style::Theme::from_name(&cli.style);
    let mut all_rendered = String::new();

    if cli.files.is_empty() {
        if io::stdin().is_terminal() {
            eprintln!("moth: no input. Usage: moth <file.md ...> or pipe markdown to stdin");
            std::process::exit(1);
        }
        let mut buf = String::new();
        io::stdin()
            .read_to_string(&mut buf)
            .expect("failed to read stdin");
        all_rendered = render::render_markdown(&buf, wrap_width, &theme);
    } else {
        for (i, path) in cli.files.iter().enumerate() {
            let markdown = if path == "-" {
                let mut buf = String::new();
                io::stdin()
                    .read_to_string(&mut buf)
                    .expect("failed to read stdin");
                buf
            } else {
                std::fs::read_to_string(path).unwrap_or_else(|e| {
                    eprintln!("moth: {path}: {e}");
                    std::process::exit(1);
                })
            };

            if i > 0 {
                // Separator between files
                let sep = "─".repeat(wrap_width.min(80));
                all_rendered.push_str(&format!(
                    "\n  \x1b[38;5;8m{sep}\x1b[0m\n  \x1b[38;5;8m{path}\x1b[0m\n\n"
                ));
            }

            all_rendered.push_str(&render::render_markdown(&markdown, wrap_width, &theme));
        }
    }

    if cli.pager {
        pipe_to_pager(&all_rendered);
    } else {
        print!("{all_rendered}");
    }
}

fn pipe_to_pager(content: &str) {
    let pager = std::env::var("PAGER").unwrap_or_else(|_| "less".into());
    let mut child = Command::new(&pager)
        .arg("-r")
        .stdin(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| {
            eprintln!("moth: failed to start pager '{pager}': {e}");
            std::process::exit(1);
        });

    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        let _ = stdin.write_all(content.as_bytes());
    }

    let _ = child.wait();
}

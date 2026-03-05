mod render;
mod style;

use std::io::{self, IsTerminal, Read};
use std::process::{Command, Stdio};

use clap::{Parser, ValueEnum};

/// A terminal markdown renderer
#[derive(Parser)]
#[command(name = "moth", version, about = "Render markdown on the CLI, with pizzazz")]
struct Cli {
    /// Markdown files to render (use "-" for stdin)
    files: Vec<String>,

    /// Word wrap at specified width (0 = terminal width)
    #[arg(short, long, default_value_t = 0)]
    width: usize,

    /// When to use the pager [default: auto]
    #[arg(long, value_enum, default_value_t = PagerMode::Auto)]
    paging: PagerMode,

    /// Style to use (dark, light)
    #[arg(short, long, default_value = "dark")]
    style: String,

    /// Show line numbers
    #[arg(short = 'n', long)]
    line_numbers: bool,
}

#[derive(Clone, ValueEnum)]
enum PagerMode {
    /// Page when output exceeds terminal height and stdout is a terminal
    Auto,
    /// Always page output
    Always,
    /// Never page output
    Never,
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
                let sep = "─".repeat(wrap_width.min(80));
                all_rendered.push_str(&format!(
                    "\n  \x1b[38;5;8m{sep}\x1b[0m\n  \x1b[38;5;8m{path}\x1b[0m\n\n"
                ));
            }

            all_rendered.push_str(&render::render_markdown(&markdown, wrap_width, &theme));
        }
    }

    if cli.line_numbers {
        all_rendered = add_line_numbers(&all_rendered);
    }

    let should_page = match cli.paging {
        PagerMode::Always => true,
        PagerMode::Never => false,
        PagerMode::Auto => {
            io::stdout().is_terminal() && exceeds_terminal_height(&all_rendered)
        }
    };

    if should_page {
        pipe_to_pager(&all_rendered);
    } else {
        print!("{all_rendered}");
    }
}

fn exceeds_terminal_height(content: &str) -> bool {
    let term_height = terminal_size::terminal_size()
        .map(|(_, h)| h.0 as usize)
        .unwrap_or(24);
    let line_count = content.lines().count();
    line_count > term_height.saturating_sub(1)
}

fn add_line_numbers(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let width = lines.len().to_string().len().max(3);
    let mut result = String::new();
    for (i, line) in lines.iter().enumerate() {
        result.push_str(&format!(
            " \x1b[38;5;8m{:>width$}\x1b[0m │ {}\n",
            i + 1,
            line,
            width = width,
        ));
    }
    result
}

fn pipe_to_pager(content: &str) {
    let pager = std::env::var("PAGER").unwrap_or_else(|_| "less".into());

    // Build args: use -R for less to handle ANSI, plus -F to quit if fits on screen
    let (cmd, args) = if pager.contains("less") {
        (pager.as_str(), vec!["-RFX"])
    } else {
        (pager.as_str(), vec!["-r"])
    };

    let mut child = Command::new(cmd)
        .args(&args)
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

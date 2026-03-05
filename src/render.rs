use crossterm::style::{Attribute, Color, SetAttribute, SetBackgroundColor, SetForegroundColor};
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

use crate::style::Theme;

/// Render markdown text to a styled terminal string.
pub fn render_markdown(input: &str, width: usize, theme: &Theme) -> String {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TABLES);

    let parser = Parser::new_ext(input, opts);
    let mut renderer = Renderer::new(width, theme);
    renderer.render(parser);
    renderer.output
}

struct Renderer<'t> {
    output: String,
    width: usize,
    theme: &'t Theme,

    // State tracking
    in_heading: Option<usize>,
    heading_text: String,
    in_code_block: bool,
    code_lang: String,
    code_text: String,
    in_blockquote: bool,
    blockquote_text: String,
    in_list: bool,
    list_stack: Vec<ListKind>,
    in_table: bool,
    table_rows: Vec<Vec<String>>,
    table_cell: String,
    table_alignments: Vec<pulldown_cmark::Alignment>,
    in_link: bool,
    link_url: String,
    link_text: String,
    inline_styles: Vec<InlineStyle>,
    needs_newline: bool,
    in_paragraph: bool,
    paragraph_text: String,
    in_table_head: bool,
}

#[derive(Clone)]
enum ListKind {
    Unordered,
    Ordered(u64),
}

#[derive(Clone, Copy)]
enum InlineStyle {
    Bold,
    Italic,
    Strikethrough,
}

impl<'t> Renderer<'t> {
    fn new(width: usize, theme: &'t Theme) -> Self {
        Self {
            output: String::new(),
            width,
            theme,
            in_heading: None,
            heading_text: String::new(),
            in_code_block: false,
            code_lang: String::new(),
            code_text: String::new(),
            in_blockquote: false,
            blockquote_text: String::new(),
            in_list: false,
            list_stack: Vec::new(),
            in_table: false,
            table_rows: Vec::new(),
            table_cell: String::new(),
            table_alignments: Vec::new(),
            in_link: false,
            link_url: String::new(),
            link_text: String::new(),
            inline_styles: Vec::new(),
            needs_newline: false,
            in_paragraph: false,
            paragraph_text: String::new(),
            in_table_head: false,
        }
    }

    fn render(&mut self, parser: Parser) {
        for event in parser {
            match event {
                Event::Start(tag) => self.start_tag(tag),
                Event::End(tag) => self.end_tag(tag),
                Event::Text(text) => self.text(&text),
                Event::Code(code) => self.inline_code(&code),
                Event::SoftBreak => self.soft_break(),
                Event::HardBreak => self.hard_break(),
                Event::Rule => self.horizontal_rule(),
                _ => {}
            }
        }
    }

    fn start_tag(&mut self, tag: Tag) {
        match tag {
            Tag::Heading { level, .. } => {
                self.ensure_blank_line();
                let lvl = match level {
                    HeadingLevel::H1 => 0,
                    HeadingLevel::H2 => 1,
                    HeadingLevel::H3 => 2,
                    HeadingLevel::H4 => 3,
                    HeadingLevel::H5 => 4,
                    HeadingLevel::H6 => 5,
                };
                self.in_heading = Some(lvl);
                self.heading_text.clear();
            }
            Tag::Paragraph => {
                if self.in_blockquote || self.in_list {
                    // handled by parent
                } else {
                    self.ensure_blank_line();
                    self.in_paragraph = true;
                    self.paragraph_text.clear();
                }
            }
            Tag::BlockQuote(_) => {
                self.ensure_blank_line();
                self.in_blockquote = true;
                self.blockquote_text.clear();
            }
            Tag::CodeBlock(kind) => {
                self.ensure_blank_line();
                self.in_code_block = true;
                self.code_lang = match kind {
                    CodeBlockKind::Fenced(lang) => lang.to_string(),
                    CodeBlockKind::Indented => String::new(),
                };
                self.code_text.clear();
            }
            Tag::List(start) => {
                if self.list_stack.is_empty() {
                    self.ensure_blank_line();
                }
                self.in_list = true;
                match start {
                    Some(n) => self.list_stack.push(ListKind::Ordered(n)),
                    None => self.list_stack.push(ListKind::Unordered),
                }
            }
            Tag::Item => {}
            Tag::Table(alignments) => {
                self.ensure_blank_line();
                self.in_table = true;
                self.table_rows.clear();
                self.table_alignments = alignments;
            }
            Tag::TableHead => {
                self.in_table_head = true;
                self.table_rows.push(Vec::new());
            }
            Tag::TableRow => {
                self.table_rows.push(Vec::new());
            }
            Tag::TableCell => {
                self.table_cell.clear();
            }
            Tag::Emphasis => {
                self.inline_styles.push(InlineStyle::Italic);
            }
            Tag::Strong => {
                self.inline_styles.push(InlineStyle::Bold);
            }
            Tag::Strikethrough => {
                self.inline_styles.push(InlineStyle::Strikethrough);
            }
            Tag::Link { dest_url, .. } => {
                self.in_link = true;
                self.link_url = dest_url.to_string();
                self.link_text.clear();
            }
            Tag::Image { dest_url, .. } => {
                // Render as [🖼 alt](url)
                let styled = format!(
                    "{}🖼  {}{}",
                    SetForegroundColor(self.theme.image_text),
                    dest_url,
                    SetForegroundColor(Color::Reset),
                );
                self.push_text(&styled);
            }
            _ => {}
        }
    }

    fn end_tag(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Heading(_level) => {
                if let Some(lvl) = self.in_heading.take() {
                    let text = std::mem::take(&mut self.heading_text);
                    self.render_heading(lvl, &text);
                }
            }
            TagEnd::Paragraph => {
                if self.in_blockquote {
                    self.blockquote_text.push('\n');
                } else if self.in_list {
                    // handled by Item
                } else {
                    let text = std::mem::take(&mut self.paragraph_text);
                    self.render_paragraph(&text);
                    self.in_paragraph = false;
                }
            }
            TagEnd::BlockQuote(_) => {
                let text = std::mem::take(&mut self.blockquote_text);
                self.render_blockquote(&text);
                self.in_blockquote = false;
            }
            TagEnd::CodeBlock => {
                let lang = std::mem::take(&mut self.code_lang);
                let code = std::mem::take(&mut self.code_text);
                self.render_code_block(&lang, &code);
                self.in_code_block = false;
            }
            TagEnd::List(_) => {
                self.list_stack.pop();
                if self.list_stack.is_empty() {
                    self.in_list = false;
                    self.needs_newline = true;
                }
            }
            TagEnd::Item => {
                // item content was already pushed
            }
            TagEnd::Table => {
                let rows = std::mem::take(&mut self.table_rows);
                let aligns = std::mem::take(&mut self.table_alignments);
                self.render_table(&rows, &aligns);
                self.in_table = false;
            }
            TagEnd::TableHead => {
                self.in_table_head = false;
            }
            TagEnd::TableRow => {}
            TagEnd::TableCell => {
                let cell = std::mem::take(&mut self.table_cell);
                if let Some(row) = self.table_rows.last_mut() {
                    row.push(cell);
                }
            }
            TagEnd::Emphasis => {
                self.inline_styles.pop();
            }
            TagEnd::Strong => {
                self.inline_styles.pop();
            }
            TagEnd::Strikethrough => {
                self.inline_styles.pop();
            }
            TagEnd::Link => {
                self.in_link = false;
                let url = std::mem::take(&mut self.link_url);
                let text = std::mem::take(&mut self.link_text);
                let styled = format!(
                    "{}{}{}{}({}{}{}){}",
                    SetAttribute(Attribute::Underlined),
                    SetForegroundColor(self.theme.link_text),
                    text,
                    SetAttribute(Attribute::NoUnderline),
                    SetForegroundColor(self.theme.link_url),
                    url,
                    SetForegroundColor(Color::Reset),
                    SetForegroundColor(Color::Reset),
                );
                self.push_text(&styled);
            }
            _ => {}
        }
    }

    fn text(&mut self, text: &str) {
        if self.in_heading.is_some() {
            self.heading_text.push_str(text);
        } else if self.in_code_block {
            self.code_text.push_str(text);
        } else if self.in_blockquote {
            self.blockquote_text.push_str(text);
        } else if self.in_table {
            self.table_cell.push_str(text);
        } else if self.in_link {
            self.link_text.push_str(text);
        } else {
            let styled = self.apply_inline_styles(text);
            self.push_text(&styled);
        }
    }

    fn inline_code(&mut self, code: &str) {
        if self.in_heading.is_some() {
            self.heading_text.push('`');
            self.heading_text.push_str(code);
            self.heading_text.push('`');
            return;
        }
        if self.in_table {
            self.table_cell.push('`');
            self.table_cell.push_str(code);
            self.table_cell.push('`');
            return;
        }
        let styled = format!(
            "{}{} {} {}{}",
            SetForegroundColor(self.theme.code_inline),
            SetBackgroundColor(self.theme.code_inline_bg),
            code,
            SetBackgroundColor(Color::Reset),
            SetForegroundColor(Color::Reset),
        );
        self.push_text(&styled);
    }

    fn soft_break(&mut self) {
        if self.in_heading.is_some() {
            self.heading_text.push(' ');
        } else if self.in_blockquote {
            self.blockquote_text.push(' ');
        } else {
            self.push_text(" ");
        }
    }

    fn hard_break(&mut self) {
        if self.in_blockquote {
            self.blockquote_text.push('\n');
        } else {
            self.push_text("\n");
        }
    }

    fn horizontal_rule(&mut self) {
        self.ensure_blank_line();
        let line: String = "─".repeat(self.width.min(80));
        self.output.push_str(&format!(
            "  {}{}{}\n",
            SetForegroundColor(self.theme.hr),
            line,
            SetForegroundColor(Color::Reset),
        ));
        self.needs_newline = true;
    }

    // ── Rendering helpers ──────────────────────────────────────

    fn render_heading(&mut self, level: usize, text: &str) {
        let color = self.theme.heading[level.min(5)];
        let prefix = "#".repeat(level + 1);

        let (attr_on, attr_off) = if level == 0 {
            (
                format!("{}", SetAttribute(Attribute::Bold)),
                format!("{}", SetAttribute(Attribute::NoBold)),
            )
        } else {
            (String::new(), String::new())
        };

        self.output.push_str(&format!(
            "  {}{prefix}{} {attr_on}{}{text}{}{attr_off}\n",
            SetForegroundColor(self.theme.heading_prefix),
            SetForegroundColor(color),
            SetAttribute(Attribute::Bold),
            SetAttribute(Attribute::NoBold),
        ));
        self.needs_newline = true;
    }

    fn render_paragraph(&mut self, text: &str) {
        let wrapped = self.wrap_text(text, self.width.saturating_sub(4));
        for line in wrapped.lines() {
            self.output.push_str(&format!("  {line}\n"));
        }
        self.needs_newline = true;
    }

    fn render_blockquote(&mut self, text: &str) {
        let bar = format!(
            "{}│{}",
            SetForegroundColor(self.theme.blockquote_bar),
            SetForegroundColor(Color::Reset),
        );

        let clean_text = text.trim_end_matches('\n');
        let wrapped = self.wrap_text(clean_text, self.width.saturating_sub(8));

        for line in wrapped.lines() {
            self.output.push_str(&format!(
                "  {bar} {}{line}{}\n",
                SetForegroundColor(self.theme.blockquote_text),
                SetForegroundColor(Color::Reset),
            ));
        }
        self.needs_newline = true;
    }

    fn render_code_block(&mut self, lang: &str, code: &str) {
        let code = code.trim_end_matches('\n');
        let content_width = self.width.saturating_sub(6);

        // Try syntax highlighting
        let highlighted = self.highlight_code(lang, code);

        // Top border
        let border_line: String = "─".repeat(content_width + 2);
        let lang_label = if lang.is_empty() {
            String::new()
        } else {
            format!(
                " {}{}{}",
                SetForegroundColor(self.theme.code_block_border),
                lang,
                SetForegroundColor(Color::Reset),
            )
        };
        self.output.push_str(&format!(
            "  {}╭{border_line}╮{}{lang_label}\n",
            SetForegroundColor(self.theme.code_block_border),
            SetForegroundColor(Color::Reset),
        ));

        // Code lines
        for line in highlighted.lines() {
            let display_len = strip_ansi_len(line);
            let padding = if display_len < content_width {
                " ".repeat(content_width - display_len)
            } else {
                String::new()
            };
            self.output.push_str(&format!(
                "  {}│{} {}{line}{padding} {}{}│{}\n",
                SetForegroundColor(self.theme.code_block_border),
                SetBackgroundColor(self.theme.code_block_bg),
                SetForegroundColor(Color::Reset),
                SetBackgroundColor(Color::Reset),
                SetForegroundColor(self.theme.code_block_border),
                SetForegroundColor(Color::Reset),
            ));
        }

        // Bottom border
        self.output.push_str(&format!(
            "  {}╰{border_line}╯{}\n",
            SetForegroundColor(self.theme.code_block_border),
            SetForegroundColor(Color::Reset),
        ));
        self.needs_newline = true;
    }

    fn highlight_code(&self, lang: &str, code: &str) -> String {
        let ss = SyntaxSet::load_defaults_newlines();
        let ts = ThemeSet::load_defaults();
        let syntax = if lang.is_empty() {
            ss.find_syntax_plain_text()
        } else {
            ss.find_syntax_by_token(lang)
                .unwrap_or_else(|| ss.find_syntax_plain_text())
        };

        let theme_name = "base16-ocean.dark";
        let theme = &ts.themes[theme_name];
        let mut h = HighlightLines::new(syntax, theme);

        let mut result = String::new();
        for line in LinesWithEndings::from(code) {
            let ranges = h.highlight_line(line, &ss).unwrap_or_default();
            for (style, text) in ranges {
                let text = text.trim_end_matches('\n');
                if text.is_empty() {
                    continue;
                }
                result.push_str(&style_to_ansi(&style, text));
            }
            if line.ends_with('\n') && !result.ends_with('\n') {
                result.push('\n');
            }
        }
        result
    }

    fn render_table(&mut self, rows: &[Vec<String>], alignments: &[pulldown_cmark::Alignment]) {
        if rows.is_empty() {
            return;
        }

        // Calculate column widths
        let num_cols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
        let mut col_widths = vec![0usize; num_cols];
        for row in rows {
            for (i, cell) in row.iter().enumerate() {
                if i < num_cols {
                    col_widths[i] = col_widths[i].max(cell.len());
                }
            }
        }

        // Ensure minimum width
        for w in &mut col_widths {
            *w = (*w).max(3);
        }

        let border_color = self.theme.table_border;

        // Top border
        self.output.push_str(&format!(
            "  {}╭{}╮{}\n",
            SetForegroundColor(border_color),
            col_widths
                .iter()
                .map(|w| "─".repeat(w + 2))
                .collect::<Vec<_>>()
                .join("┬"),
            SetForegroundColor(Color::Reset),
        ));

        for (row_idx, row) in rows.iter().enumerate() {
            // Data row
            self.output.push_str(&format!(
                "  {}│{}",
                SetForegroundColor(border_color),
                SetForegroundColor(Color::Reset),
            ));
            for (col, cell) in row.iter().enumerate() {
                let width = col_widths.get(col).copied().unwrap_or(3);
                let align = alignments
                    .get(col)
                    .copied()
                    .unwrap_or(pulldown_cmark::Alignment::None);
                let padded = align_text(cell, width, align);

                let cell_style = if row_idx == 0 {
                    format!("{}{}", SetAttribute(Attribute::Bold), SetForegroundColor(self.theme.text))
                } else {
                    format!("{}", SetForegroundColor(self.theme.text))
                };
                let cell_reset = if row_idx == 0 {
                    format!("{}{}", SetAttribute(Attribute::NoBold), SetForegroundColor(Color::Reset))
                } else {
                    format!("{}", SetForegroundColor(Color::Reset))
                };

                self.output.push_str(&format!(
                    " {cell_style}{padded}{cell_reset} {}│{}",
                    SetForegroundColor(border_color),
                    SetForegroundColor(Color::Reset),
                ));
            }
            self.output.push('\n');

            // Separator after header
            if row_idx == 0 {
                self.output.push_str(&format!(
                    "  {}├{}┤{}\n",
                    SetForegroundColor(border_color),
                    col_widths
                        .iter()
                        .map(|w| "─".repeat(w + 2))
                        .collect::<Vec<_>>()
                        .join("┼"),
                    SetForegroundColor(Color::Reset),
                ));
            }
        }

        // Bottom border
        self.output.push_str(&format!(
            "  {}╰{}╯{}\n",
            SetForegroundColor(border_color),
            col_widths
                .iter()
                .map(|w| "─".repeat(w + 2))
                .collect::<Vec<_>>()
                .join("┴"),
            SetForegroundColor(Color::Reset),
        ));
        self.needs_newline = true;
    }

    fn apply_inline_styles(&self, text: &str) -> String {
        let mut result = String::new();
        for style in &self.inline_styles {
            match style {
                InlineStyle::Bold => {
                    result.push_str(&format!(
                        "{}{}",
                        SetAttribute(Attribute::Bold),
                        SetForegroundColor(self.theme.bold),
                    ));
                }
                InlineStyle::Italic => {
                    result.push_str(&format!(
                        "{}{}",
                        SetAttribute(Attribute::Italic),
                        SetForegroundColor(self.theme.italic),
                    ));
                }
                InlineStyle::Strikethrough => {
                    result.push_str(&format!(
                        "{}{}",
                        SetAttribute(Attribute::CrossedOut),
                        SetForegroundColor(self.theme.strikethrough),
                    ));
                }
            }
        }
        result.push_str(text);
        for style in self.inline_styles.iter().rev() {
            match style {
                InlineStyle::Bold => {
                    result.push_str(&format!(
                        "{}{}",
                        SetAttribute(Attribute::NoBold),
                        SetForegroundColor(Color::Reset),
                    ));
                }
                InlineStyle::Italic => {
                    result.push_str(&format!(
                        "{}{}",
                        SetAttribute(Attribute::NoItalic),
                        SetForegroundColor(Color::Reset),
                    ));
                }
                InlineStyle::Strikethrough => {
                    result.push_str(&format!(
                        "{}{}",
                        SetAttribute(Attribute::NotCrossedOut),
                        SetForegroundColor(Color::Reset),
                    ));
                }
            }
        }
        result
    }

    fn push_text(&mut self, text: &str) {
        if self.in_list {
            // For list items, we handle text directly
            if text.trim().is_empty() {
                return;
            }
            let depth = self.list_stack.len();
            let indent = "  ".repeat(depth);

            let bullet = match self.list_stack.last_mut() {
                Some(ListKind::Unordered) => {
                    let bullets = ["•", "◦", "▸", "▹"];
                    let b = bullets[(depth - 1).min(bullets.len() - 1)];
                    format!(
                        "{}{}{}",
                        SetForegroundColor(self.theme.list_bullet),
                        b,
                        SetForegroundColor(Color::Reset),
                    )
                }
                Some(ListKind::Ordered(n)) => {
                    let num = *n;
                    *n += 1;
                    format!(
                        "{}{num}.{}",
                        SetForegroundColor(self.theme.list_bullet),
                        SetForegroundColor(Color::Reset),
                    )
                }
                None => String::new(),
            };

            self.output
                .push_str(&format!("  {indent}{bullet} {text}\n"));
        } else if self.in_paragraph {
            self.paragraph_text.push_str(text);
        } else {
            self.output.push_str(text);
        }
    }

    fn ensure_blank_line(&mut self) {
        if self.needs_newline {
            self.output.push('\n');
            self.needs_newline = false;
        } else if !self.output.is_empty() && !self.output.ends_with("\n\n") {
            if !self.output.ends_with('\n') {
                self.output.push('\n');
            }
        }
    }

    fn wrap_text(&self, text: &str, width: usize) -> String {
        // Simple wrapping that respects ANSI escape codes
        // For now, use textwrap on the plain text portions
        textwrap::fill(text, width)
    }
}

fn style_to_ansi(style: &Style, text: &str) -> String {
    let fg = style.foreground;
    format!(
        "{}{}{}",
        SetForegroundColor(Color::Rgb {
            r: fg.r,
            g: fg.g,
            b: fg.b,
        }),
        text,
        SetForegroundColor(Color::Reset),
    )
}

fn strip_ansi_len(s: &str) -> usize {
    // Quick and dirty: count visible characters by skipping ANSI escape sequences
    let mut len = 0;
    let mut in_escape = false;
    for c in s.chars() {
        if in_escape {
            if c.is_ascii_alphabetic() {
                in_escape = false;
            }
        } else if c == '\x1b' {
            in_escape = true;
        } else {
            len += 1;
        }
    }
    len
}

fn align_text(text: &str, width: usize, align: pulldown_cmark::Alignment) -> String {
    let text_len = text.len();
    if text_len >= width {
        return text[..width].to_string();
    }
    let padding = width - text_len;
    match align {
        pulldown_cmark::Alignment::Right => format!("{}{}", " ".repeat(padding), text),
        pulldown_cmark::Alignment::Center => {
            let left = padding / 2;
            let right = padding - left;
            format!("{}{}{}", " ".repeat(left), text, " ".repeat(right))
        }
        _ => format!("{}{}", text, " ".repeat(padding)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render_plain(md: &str) -> String {
        let theme = crate::style::Theme::dark();
        render_markdown(md, 80, &theme)
    }

    fn strip_ansi(s: &str) -> String {
        let mut result = String::new();
        let mut in_escape = false;
        for c in s.chars() {
            if in_escape {
                if c.is_ascii_alphabetic() {
                    in_escape = false;
                }
            } else if c == '\x1b' {
                in_escape = true;
            } else {
                result.push(c);
            }
        }
        result
    }

    #[test]
    fn test_heading_rendering() {
        let output = strip_ansi(&render_plain("# Hello World"));
        assert!(output.contains("# Hello World"));
    }

    #[test]
    fn test_h2_rendering() {
        let output = strip_ansi(&render_plain("## Subheading"));
        assert!(output.contains("## Subheading"));
    }

    #[test]
    fn test_paragraph() {
        let output = strip_ansi(&render_plain("This is a paragraph."));
        assert!(output.contains("This is a paragraph."));
    }

    #[test]
    fn test_code_block() {
        let md = "```rust\nfn main() {}\n```";
        let output = strip_ansi(&render_plain(md));
        assert!(output.contains("fn main() {}"));
        assert!(output.contains("╭")); // box drawing
        assert!(output.contains("╰")); // box drawing
    }

    #[test]
    fn test_blockquote() {
        let output = strip_ansi(&render_plain("> A wise quote"));
        assert!(output.contains("│"));
        assert!(output.contains("A wise quote"));
    }

    #[test]
    fn test_unordered_list() {
        let md = "- one\n- two\n- three";
        let output = strip_ansi(&render_plain(md));
        assert!(output.contains("•"));
        assert!(output.contains("one"));
        assert!(output.contains("two"));
        assert!(output.contains("three"));
    }

    #[test]
    fn test_ordered_list() {
        let md = "1. first\n2. second";
        let output = strip_ansi(&render_plain(md));
        assert!(output.contains("1."));
        assert!(output.contains("first"));
    }

    #[test]
    fn test_horizontal_rule() {
        let output = strip_ansi(&render_plain("---"));
        assert!(output.contains("─"));
    }

    #[test]
    fn test_table() {
        let md = "| A | B |\n|---|---|\n| 1 | 2 |";
        let output = strip_ansi(&render_plain(md));
        assert!(output.contains("╭"));
        assert!(output.contains("A"));
        assert!(output.contains("1"));
    }

    #[test]
    fn test_strip_ansi_len() {
        assert_eq!(strip_ansi_len("hello"), 5);
        assert_eq!(strip_ansi_len("\x1b[31mred\x1b[0m"), 3);
    }

    #[test]
    fn test_inline_code() {
        let output = render_plain("Use `cargo build` to compile.");
        assert!(output.contains("cargo build"));
    }

    #[test]
    fn test_glow_readme() {
        let readme = include_str!("../tests/fixtures/glow_readme.md");
        // Should not panic
        let output = render_plain(readme);
        let plain = strip_ansi(&output);
        assert!(plain.contains("Glow"));
        assert!(plain.contains("markdown"));
    }
}

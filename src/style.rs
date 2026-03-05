use crossterm::style::Color;

/// Terminal colors for each markdown element
pub struct Theme {
    pub heading: [Color; 6],
    pub heading_prefix: Color,
    pub text: Color,
    pub bold: Color,
    pub italic: Color,
    pub code_inline: Color,
    pub code_inline_bg: Color,
    pub code_block_bg: Color,
    pub code_block_border: Color,
    pub link_text: Color,
    pub link_url: Color,
    pub blockquote_bar: Color,
    pub blockquote_text: Color,
    pub list_bullet: Color,
    pub hr: Color,
    pub table_border: Color,
    pub image_text: Color,
    pub strikethrough: Color,
}

impl Theme {
    pub fn from_name(name: &str) -> Self {
        match name {
            "light" => Self::light(),
            _ => Self::dark(),
        }
    }

    pub fn dark() -> Self {
        Self {
            heading: [
                Color::Rgb { r: 189, g: 147, b: 249 }, // h1 - purple
                Color::Rgb { r: 139, g: 233, b: 253 }, // h2 - cyan
                Color::Rgb { r: 80, g: 250, b: 123 },  // h3 - green
                Color::Rgb { r: 255, g: 184, b: 108 }, // h4 - orange
                Color::Rgb { r: 255, g: 121, b: 198 }, // h5 - pink
                Color::Rgb { r: 241, g: 250, b: 140 }, // h6 - yellow
            ],
            heading_prefix: Color::DarkGrey,
            text: Color::Rgb { r: 248, g: 248, b: 242 },
            bold: Color::Rgb { r: 255, g: 255, b: 255 },
            italic: Color::Rgb { r: 139, g: 233, b: 253 },
            code_inline: Color::Rgb { r: 255, g: 184, b: 108 },
            code_inline_bg: Color::Rgb { r: 68, g: 71, b: 90 },
            code_block_bg: Color::Rgb { r: 40, g: 42, b: 54 },
            code_block_border: Color::Rgb { r: 98, g: 114, b: 164 },
            link_text: Color::Rgb { r: 139, g: 233, b: 253 },
            link_url: Color::DarkGrey,
            blockquote_bar: Color::Rgb { r: 98, g: 114, b: 164 },
            blockquote_text: Color::Rgb { r: 188, g: 188, b: 188 },
            list_bullet: Color::Rgb { r: 189, g: 147, b: 249 },
            hr: Color::Rgb { r: 98, g: 114, b: 164 },
            table_border: Color::Rgb { r: 98, g: 114, b: 164 },
            image_text: Color::Rgb { r: 80, g: 250, b: 123 },
            strikethrough: Color::DarkGrey,
        }
    }

    pub fn light() -> Self {
        Self {
            heading: [
                Color::Rgb { r: 124, g: 58, b: 237 },  // h1 - purple
                Color::Rgb { r: 6, g: 148, b: 162 },    // h2 - teal
                Color::Rgb { r: 22, g: 163, b: 74 },    // h3 - green
                Color::Rgb { r: 234, g: 88, b: 12 },    // h4 - orange
                Color::Rgb { r: 219, g: 39, b: 119 },   // h5 - pink
                Color::Rgb { r: 161, g: 98, b: 7 },     // h6 - amber
            ],
            heading_prefix: Color::Grey,
            text: Color::Rgb { r: 30, g: 30, b: 30 },
            bold: Color::Black,
            italic: Color::Rgb { r: 6, g: 148, b: 162 },
            code_inline: Color::Rgb { r: 194, g: 65, b: 12 },
            code_inline_bg: Color::Rgb { r: 241, g: 245, b: 249 },
            code_block_bg: Color::Rgb { r: 248, g: 250, b: 252 },
            code_block_border: Color::Rgb { r: 203, g: 213, b: 225 },
            link_text: Color::Rgb { r: 37, g: 99, b: 235 },
            link_url: Color::Grey,
            blockquote_bar: Color::Rgb { r: 203, g: 213, b: 225 },
            blockquote_text: Color::Rgb { r: 100, g: 116, b: 139 },
            list_bullet: Color::Rgb { r: 124, g: 58, b: 237 },
            hr: Color::Rgb { r: 203, g: 213, b: 225 },
            table_border: Color::Rgb { r: 203, g: 213, b: 225 },
            image_text: Color::Rgb { r: 22, g: 163, b: 74 },
            strikethrough: Color::Grey,
        }
    }
}

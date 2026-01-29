use crossterm::style::Color;

pub fn parse_color(s: &str) -> Option<Color> {
    let s = s.trim().to_lowercase();
    match s.as_str() {
        "none" => None,
        s if s.starts_with('#') => parse_hex(s),
        _ => parse_named(&s),
    }
}

pub fn parse_hex(s: &str) -> Option<Color> {
    let hex = s.strip_prefix('#')?;
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(Color::Rgb { r, g, b })
}

fn parse_named(s: &str) -> Option<Color> {
    match s {
        "black" => Some(Color::Black),
        "red" | "dark_red" => Some(Color::DarkRed),
        "green" | "dark_green" => Some(Color::DarkGreen),
        "yellow" | "dark_yellow" => Some(Color::DarkYellow),
        "blue" | "dark_blue" => Some(Color::DarkBlue),
        "magenta" | "dark_magenta" => Some(Color::DarkMagenta),
        "cyan" | "dark_cyan" => Some(Color::DarkCyan),
        "white" => Some(Color::White),
        "grey" | "gray" => Some(Color::Grey),
        "bright_red" => Some(Color::Red),
        "bright_green" => Some(Color::Green),
        "bright_yellow" => Some(Color::Yellow),
        "bright_blue" => Some(Color::Blue),
        "bright_magenta" => Some(Color::Magenta),
        "bright_cyan" => Some(Color::Cyan),
        "bright_white" => Some(Color::White),
        "dark_grey" | "dark_gray" => Some(Color::DarkGrey),
        _ => None,
    }
}

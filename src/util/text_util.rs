#![allow(dead_code)]

const RESET_ANSI: &str = "\x1B[0m";

use std::fmt::Display;

const BOLD_CODE_ANSI: &str  = "\x1B[1m";
const BOLD_RESET_ANSI: &str = "\x1B[22m";

pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite
}

impl Color {
    pub fn background_code(&self) -> i32 {
        match self {
            Color::Black         => 40,
            Color::Red           => 41,
            Color::Green         => 42,
            Color::Yellow        => 43,
            Color::Blue          => 44,
            Color::Magenta       => 45,
            Color::Cyan          => 46,
            Color::White         => 47,
            Color::BrightBlack   => 100,
            Color::BrightRed     => 101,
            Color::BrightGreen   => 102,
            Color::BrightYellow  => 103,
            Color::BrightBlue    => 104,
            Color::BrightMagenta => 105,
            Color::BrightCyan    => 106,
            Color::BrightWhite   => 107,
        }
    }

    pub fn foreground_code(&self) -> i32 {
        match self {
            Color::Black         => 30,
            Color::Red           => 31,
            Color::Green         => 32,
            Color::Yellow        => 33,
            Color::Blue          => 34,
            Color::Magenta       => 35,
            Color::Cyan          => 36,
            Color::White         => 37,
            Color::BrightBlack   => 90,
            Color::BrightRed     => 91,
            Color::BrightGreen   => 92,
            Color::BrightYellow  => 93,
            Color::BrightBlue    => 94,
            Color::BrightMagenta => 95,
            Color::BrightCyan    => 96,
            Color::BrightWhite   => 97,
        }
    }
}

pub fn bold<T: AsRef<str> + Display>(text: T) -> String {
    format!("{BOLD_CODE_ANSI}{}{BOLD_RESET_ANSI}", text)
}

pub fn colored_background<T: AsRef<str> + Display>(
      text: T
    , color: Color
    ) -> String 
{
    colored(text, Some(color), None)
}

pub fn colored_foreground<T: AsRef<str> + Display>(
    text: T
  , color: Color
  ) -> String 
{
    colored(text, None, Some(color))
}

pub fn colored<T: AsRef<str> + Display>(
      text: T
    , background_color: Option<Color>
    , foreground_color: Option<Color>
    ) -> String 
{
    match (foreground_color, background_color) {
        (None           , None           ) => text.to_string(),
        (None           , Some(bg)) => format!("\x1B[{}m{text}{RESET_ANSI}", bg.background_code()),
        (Some(fg), None           ) => format!("\x1B[{}m{text}{RESET_ANSI}", fg.foreground_code()),
        (Some(fg), Some(bg)) => format!("\x1B[{};{}m{text}{RESET_ANSI}", fg.foreground_code(), bg.background_code()),
    }
}
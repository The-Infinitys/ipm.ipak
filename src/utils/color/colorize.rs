use super::RGB;
pub trait Colorize {
    fn red(&self) -> String;
    fn yellow(&self) -> String;
    fn green(&self) -> String;
    fn cyan(&self) -> String;
    fn blue(&self) -> String;
    fn magenta(&self) -> String;
    fn rgb(&self, rgb: RGB) -> String;
}
impl Colorize for String {
    fn red(&self) -> String {
        format!("\x1b[31m{}\x1b[0m", self)
    }
    fn yellow(&self) -> String {
        format!("\x1b[33m{}\x1b[0m", self)
    }
    fn green(&self) -> String {
        format!("\x1b[32m{}\x1b[0m", self)
    }
    fn cyan(&self) -> String {
        format!("\x1b[36m{}\x1b[0m", self)
    }
    fn blue(&self) -> String {
        format!("\x1b[34m{}\x1b[0m", self)
    }
    fn magenta(&self) -> String {
        format!("\x1b[35m{}\x1b[0m", self)
    }
    fn rgb(&self, rgb: RGB) -> String {
        format!(
            "\x1b[38;2;{};{};{}m{}\x1b[0m",
            rgb.red, rgb.green, rgb.blue, self
        )
    }
}

impl Colorize for &str {
    fn red(&self) -> String {
        format!("\x1b[31m{}\x1b[0m", self)
    }
    fn yellow(&self) -> String {
        format!("\x1b[33m{}\x1b[0m", self)
    }
    fn green(&self) -> String {
        format!("\x1b[32m{}\x1b[0m", self)
    }
    fn cyan(&self) -> String {
        format!("\x1b[36m{}\x1b[0m", self)
    }
    fn blue(&self) -> String {
        format!("\x1b[34m{}\x1b[0m", self)
    }
    fn magenta(&self) -> String {
        format!("\x1b[35m{}\x1b[0m", self)
    }
    fn rgb(&self, rgb: RGB) -> String {
        format!(
            "\x1b[38;2;{};{};{}m{}\x1b[0m",
            rgb.red, rgb.green, rgb.blue, self
        )
    }
}

pub trait ColorizeBg {
    fn red_bg(&self) -> String;
    fn yellow_bg(&self) -> String;
    fn green_bg(&self) -> String;
    fn cyan_bg(&self) -> String;
    fn blue_bg(&self) -> String;
    fn magenta_bg(&self) -> String;
    fn rgb_bg(&self, rgb: RGB) -> String;
}
impl ColorizeBg for String {
    fn red_bg(&self) -> String {
        format!("\x1b[41m{}\x1b[0m", self)
    }
    fn yellow_bg(&self) -> String {
        format!("\x1b[43m{}\x1b[0m", self)
    }
    fn green_bg(&self) -> String {
        format!("\x1b[42m{}\x1b[0m", self)
    }
    fn cyan_bg(&self) -> String {
        format!("\x1b[46m{}\x1b[0m", self)
    }
    fn blue_bg(&self) -> String {
        format!("\x1b[44m{}\x1b[0m", self)
    }
    fn magenta_bg(&self) -> String {
        format!("\x1b[45m{}\x1b[0m", self)
    }
    fn rgb_bg(&self, rgb: RGB) -> String {
        format!(
            "\x1b[48;2;{};{};{}m{}\x1b[0m",
            rgb.red, rgb.green, rgb.blue, self
        )
    }
}
impl ColorizeBg for &str {
    fn red_bg(&self) -> String {
        format!("\x1b[41m{}\x1b[0m", self)
    }
    fn yellow_bg(&self) -> String {
        format!("\x1b[43m{}\x1b[0m", self)
    }
    fn green_bg(&self) -> String {
        format!("\x1b[42m{}\x1b[0m", self)
    }
    fn cyan_bg(&self) -> String {
        format!("\x1b[46m{}\x1b[0m", self)
    }
    fn blue_bg(&self) -> String {
        format!("\x1b[44m{}\x1b[0m", self)
    }
    fn magenta_bg(&self) -> String {
        format!("\x1b[45m{}\x1b[0m", self)
    }
    fn rgb_bg(&self, rgb: RGB) -> String {
        format!(
            "\x1b[48;2;{};{};{}m{}\x1b[0m",
            rgb.red, rgb.green, rgb.blue, self
        )
    }
}
pub trait StyleModifier {
    fn bold(&self) -> String;
    fn italic(&self) -> String;
    fn underline(&self) -> String;
}

impl StyleModifier for String {
    fn bold(&self) -> String {
        format!("\x1b[1m{}\x1b[0m", self)
    }
    fn italic(&self) -> String {
        format!("\x1b[3m{}\x1b[0m", self)
    }
    fn underline(&self) -> String {
        format!("\x1b[4m{}\x1b[0m", self)
    }
}

impl StyleModifier for &str {
    fn bold(&self) -> String {
        format!("\x1b[1m{}\x1b[0m", self)
    }
    fn italic(&self) -> String {
        format!("\x1b[3m{}\x1b[0m", self)
    }
    fn underline(&self) -> String {
        format!("\x1b[4m{}\x1b[0m", self)
    }
}

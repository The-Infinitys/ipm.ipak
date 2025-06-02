use std::fmt;
use std::str::FromStr;
pub mod colorize;
#[derive(Debug)]
pub struct RGB {
    red: u8,
    green: u8,
    blue: u8,
}

impl FromStr for RGB {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with('#') || s.len() != 7 {
            return Err(format!("無効なカラーコードです: {}", s));
        }

        let r = u8::from_str_radix(&s[1..3], 16)
            .map_err(|_| format!("無効な赤色値です: {}", &s[1..3]))?;
        let g = u8::from_str_radix(&s[3..5], 16)
            .map_err(|_| format!("無効な緑色値です: {}", &s[3..5]))?;
        let b = u8::from_str_radix(&s[5..7], 16)
            .map_err(|_| format!("無効な青色値です: {}", &s[5..7]))?;

        Ok(RGB { red: r, green: g, blue: b })
    }
}

impl fmt::Display for RGB {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#{:02X}{:02X}{:02X}", self.red, self.green, self.blue)
    }
}

impl RGB {
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        RGB { red, green, blue }
    }

    pub fn to_ansi_color_code(&self) -> String {
        format!("\x1b[38;2;{};{};{}m", self.red, self.green, self.blue)
    }

    pub fn reset() -> &'static str {
        "\x1b[0m"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    #[test]
    fn test() {
        let red = RGB::from_str("#FF0000").unwrap();
        assert_eq!(red.to_string(), "#FF0000");
    }
}

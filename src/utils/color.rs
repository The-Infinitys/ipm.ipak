//! このモジュールは、RGBカラー値の表現と操作を提供します。
//! 16進数文字列との相互変換機能を含みます。

use std::fmt;
use std::str::FromStr;
pub mod colorize;

/// RGBカラー値を表現する構造体です。
#[derive(Debug)]
pub struct RGB {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl FromStr for RGB {
    type Err = String;

    /// 16進数文字列（例: "#RRGGBB"）からRGB構造体をパースします。
    ///
    /// # Arguments
    /// * `s` - 16進数カラーコード文字列
    ///
    /// # Returns
    /// `Ok(RGB)`: パースが成功した場合
    /// `Err(String)`: パースに失敗した場合
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
    /// RGB値を16進数文字列（例: "#RRGGBB"）としてフォーマットします。
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#{:02X}{:02X}{:02X}", self.red, self.green, self.blue)
    }
}

impl RGB {
    /// 新しいRGB構造体を作成します。
    ///
    /// # Arguments
    /// * `red` - 赤色の値 (0-255)
    /// * `green` - 緑色の値 (0-255)
    /// * `blue` - 青色の値 (0-255)
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        RGB { red, green, blue }
    }
}

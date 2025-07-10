//! このモジュールは、セマンティックバージョニングを扱うための構造体と機能を提供します。
//! `Version`構造体はバージョン番号を解析し、比較するための機能を提供します。
//! `VersionRange`構造体は、特定のバージョン範囲を定義し、バージョンがその範囲内にあるかをチェックする機能を提供します。

use std::{fmt, fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

/// バージョン番号を表す構造体です。
///
/// バージョン文字列（例: "1.2.3-alpha.4"）を解析し、数値部分とセパレータ部分に分割して保持します。
/// これにより、セマンティックバージョニングのルールに基づいた比較が可能になります。
#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct Version {
    /// 元のバージョン文字列。
    pub string: String,
    /// バージョン文字列の数値部分。
    pub nums: Vec<u32>,
    /// バージョン文字列のセパレータ部分（例: "-", "."）。
    pub separators: Vec<String>,
}

impl Default for Version {
    /// デフォルトのバージョンとして"1.0.0"を返します。
    fn default() -> Self {
        Version::from_str("1.0.0").unwrap()
    }
}

impl Serialize for Version {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Version::from_str(&s).map_err(serde::de::Error::custom)
    }
}

/// バージョン文字列を数値とセパレータに分解します。
///
/// # Arguments
/// * `version_str` - 解析するバージョン文字列。
///
/// # Returns
/// バージョン文字列から抽出された数値のベクターとセパレータのベクターのタプル。
fn serialize_version_str(version_str: &str) -> (Vec<u32>, Vec<String>) {
    let mut numbers = Vec::new();
    let mut separators = Vec::new();
    let mut current_segment = String::new();
    let mut is_digit_segment = true;

    for c in version_str.chars() {
        if c.is_ascii_digit() {
            if !is_digit_segment {
                separators.push(std::mem::take(&mut current_segment));
                is_digit_segment = true;
            }
            current_segment.push(c);
        } else {
            if is_digit_segment {
                if let Ok(num) = current_segment.parse::<u32>() {
                    numbers.push(num);
                }
                std::mem::take(&mut current_segment);
                is_digit_segment = false;
            }
            current_segment.push(c);
        }
    }

    if is_digit_segment {
        if let Ok(num) = current_segment.parse::<u32>() {
            numbers.push(num);
        }
    } else {
        separators.push(current_segment);
    }

    (numbers, separators)
}

impl FromStr for Version {
    type Err = String;

    /// 文字列から`Version`構造体をパースします。
    ///
    /// # Arguments
    /// * `s` - パースするバージョン文字列。
    ///
    /// # Returns
    /// `Ok(Version)`: パースが成功した場合。
    /// `Err(String)`: パースに失敗した場合。
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (nums, separators) = serialize_version_str(s);
        if nums.is_empty() {
            return Err(
                "There is no values for Version struct.".to_string()
            );
        }
        Ok(Version { string: s.to_string(), nums, separators })
    }
}

impl Version {
    /// 指定された範囲データにバージョンを挿入します。
    ///
    /// この関数は、バージョン範囲のロジックを処理し、新しいバージョンが既存の範囲とどのように相互作用するかを決定します。
    ///
    /// # Arguments
    /// * `range_data_opt` - 既存の`RangeData`の`Option`。
    /// * `insert_type` - 挿入するバージョンのタイプ（例: 厳密に以前、以前または等しい）。
    ///
    /// # Returns
    /// 更新された`RangeData`の`Option`。競合がある場合は`None`を返します。
    fn insert_to_range_data(
        &self,
        range_data_opt: Option<RangeData>,
        insert_type: VersionRangeInsertType,
    ) -> Option<RangeData> {
        let mut range_data = range_data_opt?;

        match insert_type {
            VersionRangeInsertType::StrictlyEarlier => {
                if range_data
                    .exactly_equal
                    .as_ref()
                    .is_some_and(|v| v >= self)
                    || range_data
                        .later_or_equal
                        .as_ref()
                        .is_some_and(|v| v >= self)
                    || range_data
                        .strictly_later
                        .as_ref()
                        .is_some_and(|v| v >= self)
                {
                    return None;
                }

                if let Some(ref mut current_earlier_or_equal) =
                    range_data.earlier_or_equal
                {
                    if *current_earlier_or_equal >= *self {
                        *current_earlier_or_equal = self.clone();
                    }
                } else if let Some(ref mut current_strictly_earlier) =
                    range_data.strictly_earlier
                {
                    if *current_strictly_earlier > *self {
                        *current_strictly_earlier = self.clone();
                    }
                } else {
                    range_data.strictly_earlier = Some(self.clone());
                }
            }
            VersionRangeInsertType::EarlierOrEqual => {
                if range_data
                    .exactly_equal
                    .as_ref()
                    .is_some_and(|v| v > self)
                    || range_data
                        .strictly_later
                        .as_ref()
                        .is_some_and(|v| v > self)
                {
                    return None;
                }

                if let Some(ref mut current_earlier_or_equal) =
                    range_data.earlier_or_equal
                {
                    if *current_earlier_or_equal > *self {
                        *current_earlier_or_equal = self.clone();
                    }
                } else {
                    range_data.earlier_or_equal = Some(self.clone());
                }

                if let Some(later_ver) = &range_data.later_or_equal {
                    if let Some(earlier_ver) = &range_data.earlier_or_equal
                    {
                        if later_ver == earlier_ver {
                            range_data.exactly_equal =
                                Some(later_ver.clone());
                            range_data.earlier_or_equal = None;
                            range_data.later_or_equal = None;
                        }
                    }
                }
            }
            VersionRangeInsertType::ExactlyEqual => {
                if range_data
                    .exactly_equal
                    .as_ref()
                    .is_some_and(|v| v != self)
                {
                    return None;
                }

                if range_data
                    .strictly_earlier
                    .as_ref()
                    .is_some_and(|v| v <= self)
                    || range_data
                        .earlier_or_equal
                        .as_ref()
                        .is_some_and(|v| v < self)
                    || range_data
                        .strictly_later
                        .as_ref()
                        .is_some_and(|v| v >= self)
                    || range_data
                        .later_or_equal
                        .as_ref()
                        .is_some_and(|v| v > self)
                {
                    return None;
                }

                range_data.exactly_equal = Some(self.clone());

                range_data.strictly_earlier = None;
                range_data.earlier_or_equal = None;
                range_data.strictly_later = None;
                range_data.later_or_equal = None;
            }
            VersionRangeInsertType::LaterOrEqual => {
                if range_data
                    .exactly_equal
                    .as_ref()
                    .is_some_and(|v| v < self)
                    || range_data
                        .strictly_earlier
                        .as_ref()
                        .is_some_and(|v| v < self)
                {
                    return None;
                }

                if let Some(ref mut current_later_or_equal) =
                    range_data.later_or_equal
                {
                    if *current_later_or_equal < *self {
                        *current_later_or_equal = self.clone();
                    }
                } else {
                    range_data.later_or_equal = Some(self.clone());
                }

                if let Some(earlier_ver) = &range_data.earlier_or_equal {
                    if let Some(later_ver) = &range_data.later_or_equal {
                        if later_ver == earlier_ver {
                            range_data.exactly_equal =
                                Some(later_ver.clone());
                            range_data.earlier_or_equal = None;
                            range_data.later_or_equal = None;
                        }
                    }
                }
            }
            VersionRangeInsertType::StrictlyLater => {
                if range_data
                    .exactly_equal
                    .as_ref()
                    .is_some_and(|v| v <= self)
                    || range_data
                        .earlier_or_equal
                        .as_ref()
                        .is_some_and(|v| v <= self)
                    || range_data
                        .strictly_earlier
                        .as_ref()
                        .is_some_and(|v| v <= self)
                {
                    return None;
                }

                if let Some(ref mut current_later_or_equal) =
                    range_data.later_or_equal
                {
                    if *current_later_or_equal <= *self {
                        range_data.later_or_equal = None;
                        range_data.strictly_later = Some(self.clone());
                    }
                } else if let Some(ref mut current_strictly_later) =
                    range_data.strictly_later
                {
                    if *current_strictly_later < *self {
                        *current_strictly_later = self.clone();
                    }
                } else {
                    range_data.strictly_later = Some(self.clone());
                }
            }
        }
        Some(range_data)
    }
}

impl fmt::Display for Version {
    /// `Version`を元の文字列形式でフォーマットします。
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.string)
    }
}

/// バージョン範囲の挿入タイプを定義する列挙型です。
#[derive(Clone, Copy, Debug)]
enum VersionRangeInsertType {
    /// 厳密に以前のバージョン。
    StrictlyEarlier,
    /// 以前または等しいバージョン。
    EarlierOrEqual,
    /// 厳密に等しいバージョン。
    ExactlyEqual,
    /// 以降または等しいバージョン。
    LaterOrEqual,
    /// 厳密に以降のバージョン。
    StrictlyLater,
}

impl PartialOrd for Version {
    /// 2つの`Version`インスタンスを比較します。
    ///
    /// 数値部分を左から順に比較し、異なる部分が見つかった時点で比較結果を返します。
    /// 全ての数値部分が同じ場合、より長いバージョンが「大きい」と判断されます。
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let min_len = self.nums.len().min(other.nums.len());
        for i in 0..min_len {
            match self.nums[i].cmp(&other.nums[i]) {
                std::cmp::Ordering::Equal => {
                    continue;
                }
                ord => return Some(ord),
            }
        }

        Some(self.nums.len().cmp(&other.nums.len()))
    }
}

/// バージョン範囲を表す構造体です。
///
/// 複数の条件（例: "> 1.0, < 2.0"）を組み合わせてバージョン範囲を定義できます。
#[derive(Clone, Debug, Default)]
pub struct VersionRange {
    /// バージョン範囲の内部データ。
    _range_data: Option<RangeData>,
}

impl Serialize for VersionRange {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for VersionRange {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        VersionRange::from_str(&s).map_err(serde::de::Error::custom)
    }
}

/// バージョン範囲の具体的なデータを保持する構造体です。
#[derive(Clone, Debug, Serialize, Deserialize)]
struct RangeData {
    /// 厳密に以前のバージョン。
    strictly_earlier: Option<Version>,
    /// 以前または等しいバージョン。
    earlier_or_equal: Option<Version>,
    /// 厳密に等しいバージョン。
    exactly_equal: Option<Version>,
    /// 以降または等しいバージョン。
    later_or_equal: Option<Version>,
    /// 厳密に以降のバージョン。
    strictly_later: Option<Version>,
}

impl FromStr for VersionRange {
    type Err = String;

    /// 文字列から`VersionRange`構造体をパースします。
    ///
    /// カンマ区切りの複数の条件をサポートします（例: "> 1.0, < 2.0"）。
    ///
    /// # Arguments
    /// * `s` - パースするバージョン範囲文字列。
    ///
    /// # Returns
    /// `Ok(VersionRange)`: パースが成功した場合。
    /// `Err(String)`: パースに失敗した場合。
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed_s = s.trim();
        if trimmed_s == "*" {
            return Ok(VersionRange { _range_data: None });
        }

        let mut range_data = Some(RangeData {
            strictly_earlier: None,
            earlier_or_equal: None,
            exactly_equal: None,
            later_or_equal: None,
            strictly_later: None,
        });

        for part in trimmed_s.split(',').map(str::trim) {
            let parts_vec: Vec<&str> = part.split_whitespace().collect();
            let (version_str, insert_type) = match parts_vec.as_slice() {
                [v_str] => (v_str, VersionRangeInsertType::ExactlyEqual),
                [symbol, v_str] => {
                    let insert_type = match *symbol {
                        ">>" | ">" => {
                            VersionRangeInsertType::StrictlyLater
                        }
                        ">=" => VersionRangeInsertType::LaterOrEqual,
                        "=" | "==" => VersionRangeInsertType::ExactlyEqual,
                        "<=" => VersionRangeInsertType::EarlierOrEqual,
                        "<<" | "<" => {
                            VersionRangeInsertType::StrictlyEarlier
                        }
                        _ => {
                            return Err(format!(
                                "Invalid relation symbol: {}",
                                symbol
                            ));
                        }
                    };
                    (v_str, insert_type)
                }
                _ => {
                    return Err(format!("Invalid range format: {}", part));
                }
            };

            let version = Version::from_str(version_str)?;
            range_data =
                version.insert_to_range_data(range_data, insert_type);

            if range_data.is_none() {
                return Err(format!("Conflicting version range: {}", s));
            }
        }

        Ok(VersionRange { _range_data: range_data })
    }
}

impl VersionRange {
    /// 指定されたバージョンがこの範囲内にあるかをチェックします。
    ///
    /// # Arguments
    /// * `version` - チェックする`Version`インスタンス。
    ///
    /// # Returns
    /// `true`: バージョンが範囲内にある場合。
    /// `false`: バージョンが範囲外にある場合。
    pub fn compare(&self, version: &Version) -> bool {
        match self._range_data.as_ref() {
            None => true,
            Some(range_data) => {
                if let Some(v) = &range_data.strictly_earlier {
                    if version >= v {
                        return false;
                    }
                }
                if let Some(v) = &range_data.earlier_or_equal {
                    if version > v {
                        return false;
                    }
                }
                if let Some(v) = &range_data.exactly_equal {
                    if version != v {
                        return false;
                    }
                }
                if let Some(v) = &range_data.later_or_equal {
                    if version < v {
                        return false;
                    }
                }
                if let Some(v) = &range_data.strictly_later {
                    if version <= v {
                        return false;
                    }
                }
                true
            }
        }
    }
}

impl Display for VersionRange {
    /// `VersionRange`を文字列形式でフォーマットします。
    ///
    /// 例: "> 1.0, <= 2.0"
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self._range_data.as_ref() {
            None => write!(f, "*"),
            Some(range_data) => {
                write!(f, "{}", range_data)
            }
        }
    }
}

impl Display for RangeData {
    /// `RangeData`を文字列形式でフォーマットします。
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut parts = Vec::new();
        if let Some(v) = &self.strictly_earlier {
            parts.push(format!("< {}", v.string));
        }
        if let Some(v) = &self.earlier_or_equal {
            parts.push(format!("<= {}", v.string));
        }
        if let Some(v) = &self.exactly_equal {
            parts.push(format!("= {}", v.string));
        }
        if let Some(v) = &self.later_or_equal {
            parts.push(format!(">= {}", v.string));
        }
        if let Some(v) = &self.strictly_later {
            parts.push(format!("> {}", v.string));
        }
        if parts.is_empty() {
            write!(f, "*")
        } else {
            write!(f, "{}", parts.join(", "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let version1 = Version::from_str("1.2.3").unwrap();
        let version2 = Version::from_str("1.2.2-build-4").unwrap();
        let version3 = Version::from_str("2.123.12").unwrap();
        println!("version2 == version1: {}", version1 == version2);
        println!("version2 >= version1: {}", version1 >= version2);
        println!("version3 < version1: {}", version3 < version1);
        let range1 =
            VersionRange::from_str("< 2.0, > 1.1.3-build-1").unwrap();
        println!("Range1: {}", &range1);
        println!("In Range1, version1: {}", range1.compare(&version1));
        let range_all = VersionRange::from_str("*").unwrap();
        println!("RangeAll: {}", &range_all);
        println!(
            "In RangeAll, version1: {}",
            range_all.compare(&version1)
        );
        let range_exact = VersionRange::from_str("== 1.2.3").unwrap();
        println!("RangeExact: {}", &range_exact);
        println!(
            "In RangeExact, version1: {}",
            range_exact.compare(&version1)
        );
        let range_debug = VersionRange::from_str("<= 2.0, = 2.0").unwrap();
        assert_eq!(range_debug.to_string(), "= 2.0");
        let conflict_range = VersionRange::from_str(">= 2.0, < 1.0");
        println!("Conflict Range: {:?}", conflict_range);
    }
}

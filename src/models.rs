extern crate mysql;
use mysql::chrono::{Local, DateTime, TimeZone};

#[derive(Debug, Clone)]
pub struct Ratio {
    pub sum: f32,
    pub ratios: Vec<f32>,
    pub sellers: u32,
    pub average: f32,
    pub sin_average: f32,
    pub pull_id: u32,
}
impl Default for Ratio {
    fn default() -> Ratio {
        return Ratio {
            sum: -0.0,
            ratios: Vec::new(),
            sellers: 0,
            average: -1.0,
            sin_average: -1.0,
            pull_id: 0,
        };
    }
}
impl Ratio {
    pub fn new() -> Self {
        Default::default()
    }
}
#[derive(Clone, Debug, Default)]
pub struct CurrencyPair {
    pub to: Currency,
    pub from: Currency,
}
impl CurrencyPair {
    pub fn new() -> CurrencyPair {
        return CurrencyPair {
            to: Currency::new(),
            from: Currency::new(),
        };
    }
    pub fn new_from_currencies(to: Currency, from: Currency) -> CurrencyPair {
        return CurrencyPair { to: to, from: from };
    }
}
#[derive(Clone)]
pub struct CurrencyPairBunched {
    pub to: Currency,
    pub from: Vec<Currency>,
}
impl fmt::Debug for CurrencyPairBunched {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut from = Vec::new();
        for iter in &self.from {
            from.push(iter.id);
        }
        write!(f, "Bunched => From: {:?}, to: {}", from, self.to.id)
    }
}
#[derive(Clone, Debug)]
pub struct BunchRatio {
    id: u32,
    ratios: Vec<f32>,
}
impl BunchRatio {
    pub fn get_id(&self) -> u32 {
        return self.id;
    }
    pub fn get_len(&self) -> usize {
        return self.ratios.len();
    }
    pub fn get_ratios(&self) -> Vec<f32> {
        return self.ratios.clone();
    }
    pub fn add_ratio(&mut self, ratio: f32) {
        self.ratios.push(ratio);
    }
    pub fn new(id: u32, ratios: Vec<f32>) -> BunchRatio {
        return BunchRatio {
            id: id,
            ratios: ratios,
        };
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Currency {
    id: u32,
    pub name: String,
    pub css: String,
    pub active: bool,
    //pub ca: mysql::chrono::NaiveDate,
    //pub ua: mysql::chrono::NaiveDate,
}
impl Currency {
    pub fn get_id(&self) -> u32 {
        return self.id;
    }
    pub fn new() -> Currency {
        return Currency {
            id: 0,
            name: "".to_string(),
            css: "".to_string(),
            active: true,
            //ca: mysql::chrono::NaiveDate::from_yo(1900, 1),
            //ua: mysql::chrono::NaiveDate::from_yo(1900, 1),
        };
    }
    pub fn new_from_data(
        id: u32,
        name: std::string::String,
        css: std::string::String,
        active: bool,
        _ca: mysql::chrono::NaiveDateTime,
        _ua: mysql::chrono::NaiveDateTime,
    ) -> Currency {
        return Currency {
            id: id,
            name: name,
            css: css,
            active: active,
            //ca: ca,
            //ua: ua,
        };
    }
}
use std::fmt;
impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({})", self.id)
    }
}
pub struct Pull {
    pub id: u32,
    pub league_id: u32,
    pub expires: mysql::chrono::NaiveDateTime,
    pub called_at: mysql::chrono::NaiveDateTime,
}

#[derive(Clone, Debug, Default)]
pub struct RatioPair {
    pub pair: CurrencyPair,
    pub ratio: Ratio,
}

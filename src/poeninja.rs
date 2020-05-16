extern crate serde;
extern crate serde_json;

use serde_json::Value;
use std::collections::BTreeMap as Map;

#[derive(Deserialize, Debug, Default)]
pub struct CurrencyDetail {
    pub id: u32,
    icon: String,
    pub name: String,
    pub poeTradeId: u32,
}

fn default_resource() -> u32 {
    0
}
#[derive(Deserialize, Debug, Clone)]
pub struct pay {
    pub id: u32,
    pub league_id: u32,
    pub pay_currency_id: u32,
    pub get_currency_id: u32,
    sample_time_utc: String,
    pub count: u32,
    pub value: f32,
    data_point_count: u32,
    includes_secondary: bool,
}
#[derive(Deserialize, Debug)]
pub struct receive {
    pub id: u32,
    pub league_id: u32,
    pub pay_currency_id: u32,
    pub get_currency_id: u32,
    sample_time_utc: String,
    pub count: u32,
    pub value: f32,
    data_point_count: u32,
    includes_secondary: bool,
}
#[derive(Deserialize, Debug)]
pub struct paySparkLine {
    data: Option<Vec<f32>>,
    totalChange: f32,
}

#[derive(Deserialize, Debug)]
pub struct receiveSparkLine {
    #[serde(deserialize_with = "default_if_empty")]
    data: Option<Vec<f32>>,
    totalChange: f32,
}
#[derive(Deserialize, Debug)]
pub struct lowConfidencePaySparkLine {
    #[serde(deserialize_with = "default_if_empty")]
    data: Option<Vec<f32>>,
    totalChange: f32,
}
#[derive(Deserialize, Debug)]
pub struct lowConfidenceReceiveSparkLine {
    #[serde(deserialize_with = "default_if_empty")]
    data: Option<Vec<f32>>,
    totalChange: f32,
}
#[derive(Deserialize, Debug, Clone)]
pub struct CurrencyInfo {
    pub currencyTypeName: String,
    pub pay: Option<pay>,
    //#[serde(flatten)]
    //pub pay: Map<String, Value>,
    #[serde(flatten)]
    pub receive: Map<String, Value>,
    #[serde(flatten)]
    paysparkline: Map<String, Value>,
    #[serde(flatten)]
    receivesparkline: Map<String, Value>,
    //#[serde(flatten)]
    pub chaosEquivalent: f32,
    //pub chaosEquivalent: Map<String, Value>,
    #[serde(flatten)]
    lowConfidencePaySparkLine: Map<String, Value>,
    #[serde(flatten)]
    lowConfidenceReceiveSparkLine: Map<String, Value>,
    /*paySparkLine: paySparkLine,
    receiveSparkLine: receiveSparkLine,
    chaosEquivalent: f32,
    lowConfidencePaySparkLine: lowConfidencePaySparkLine,
    lowConfidenceReceiveSparkLine: lowConfidenceReceiveSparkLine,*/
}
impl CurrencyInfo {
    pub fn getCurrencyTypeName(self) -> std::string::String {
        return self.currencyTypeName.to_string().clone();
    }
}

#[derive(Deserialize, Debug)]
pub struct Poeninja {
    pub lines: Vec<CurrencyInfo>,
    pub currencyDetails: Vec<CurrencyDetail>,
}
impl Default for Poeninja {
    fn default() -> Poeninja {
        Poeninja {
            lines: Vec::new(),
            currencyDetails: Vec::new(),
        }
    }
}
fn default_if_empty<'de, D, T>(de: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de> + Default,
{
    use serde::Deserialize;
    Option::<T>::deserialize(de).map(|x| x.unwrap_or_else(|| T::default()))
}
#[derive(Clone, Debug)]
pub struct CurrencyValues {
    pub id: u32,
    pub value: f32,
    pub name: String,
    pub to: u32,
}

fn scrape_poeninja_start() {
    let url: String =
        "http://poe.ninja/api/Data/GetCurrencyOverview?league=Hardcore%20Incursion".to_string();
    let mut retval: Vec<pj::CurrencyValues> = Vec::new();
    match scrape_poeninja(&url, &mut retval) {
        Ok(result) => println!("{:?}", result),
        Err(e) => println!("{:?}", e),
    }

    retval.push(pj::CurrencyValues {
        id: 4,
        name: "Chaos Orb".to_string(),
        value: 1.0,
        to: 4,
    });
    for iter in &retval {
        println!("{:?}", iter);
    }

    let mut cp: Vec<db::models::CurrencyPair> = Vec::new();
    let mut rp: Vec<db::models::RatioPair> = Vec::new();
    for i in 0..retval.clone().len() {
        for j in 0..retval.len() {
            if &retval[i].id == &retval[j].id {
                continue;
            }
            let c1: db::models::Currency = db::models::Currency {
                id: retval[i].id.clone(),
                name: retval[i].name.clone(),
                css: "".to_string(),
                active: true,
                ca: mysql::chrono::NaiveDate::from_yo(2015, 73),
                ua: mysql::chrono::NaiveDate::from_yo(2015, 73),
            };
            let c2: db::models::Currency = db::models::Currency {
                id: retval[j].id.clone(),
                name: retval[j].name.clone(),
                css: "".to_string(),
                active: true,
                ca: mysql::chrono::NaiveDate::from_yo(2015, 73),
                ua: mysql::chrono::NaiveDate::from_yo(2015, 73),
            };
            let new = db::models::CurrencyPair { to: c1, from: c2 };
            cp.push(new);
        }
    }
    for iter in cp {
        let mut ratio = db::models::Ratio {
            sum: 0.0,
            ratios: Vec::new(),
            sellers: 0,
            average: 0.0,
            sin_average: 0.0,
        };
        ratio.ratios = Vec::new();
        let mut torat = 0.0;
        let mut fromrat = 0.0;
        for curv in retval.clone() {
            if iter.to.id == curv.id {
                torat = curv.value.clone();
            }
            if iter.from.id == curv.id {
                fromrat = curv.value.clone();
            }
        }
        for _i in 0..12 {
            let rat = fromrat / torat;
            ratio.ratios.push(rat);
            ratio.sum = ratio.sum + rat;
        }
        ratio.sellers = ratio.ratios.len() as u32;
        ratio.average = ratio.sum / ratio.ratios.len() as f32;
        //println!("body = {:?}", resp);

        ratio.sin_average = calc_sin_avg(&ratio);
        rp.push(db::models::RatioPair {
            pair: db::models::CurrencyPair {
                from: iter.from.clone(),
                to: iter.to.clone(),
            },
            ratio: ratio.clone(),
        });
    }
    for i in &rp {
        println!("{} vs {} -> Sellers: {} Sum: {} average: {} sin average: {} -> Cheapest: {} and most expensive: {}", i.pair.to.name, i.pair.from.name, i.ratio.ratios.len(), i.ratio.sum, i.ratio.average, i.ratio.sin_average, i.ratio.ratios[0], i.ratio.ratios[i.ratio.ratios.len()-1]);
    }
}
fn scrape_poeninja(
    url: &String,
    retval: &mut Vec<pj::CurrencyValues>,
) -> Result<(), reqwest::Error> {
    let body = reqwest::get(url)?.text()?;

    let json: pj::Poeninja = match serde_json::from_str(&body) {
        Result::Ok(v) => {
            //print!("{:?}", v);
            v
        }
        Result::Err(err) => {
            println!("\nFailed to pars\n{:?}\n", &err);
            pj::Poeninja::default()
        }
    };
    /*for iter in &json.lines {
        let cname = &iter.currencyTypeName.clone();
        println!("{} -> {} -> {}", &cname, &iter.chaosEquivalent.clone());

    }*/
    for i in 0..json.lines.len() - 1 {
        let mut id: u32 = 0;
        let mut toid: u32 = 0;
        let mut value: f32 = 0.0;
        for j in &json.currencyDetails {
            if json.lines[i].currencyTypeName == j.name {
                id = j.poeTradeId.clone();
                //break;
            }
            match &json.lines[i].clone().pay {
                None => break,
                Some(v) => {
                    if v.pay_currency_id == j.poeTradeId {
                        toid = j.poeTradeId.clone();
                        value = v.value;
                    }
                }
            }
        }
        let mut cv = pj::CurrencyValues {
            id: id,
            value: value,
            name: json.lines[i].currencyTypeName.clone(),
            to: toid,
        };
        retval.push(cv.clone());
        //println!("{} -> {} -> {}", cv.id, cv.name, cv.chaosValue );
    }
    Ok(())
}

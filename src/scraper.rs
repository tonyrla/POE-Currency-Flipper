use db;
use models as dbm;
use rayon::prelude::*;
use std;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

extern crate itertools;
extern crate regex;
extern crate reqwest;
pub extern crate serde;
pub extern crate serde_json;

fn get_currencylist(pools: &mysql::Pool) -> Vec<dbm::Currency> {
    trace!("Getting currency list!");
    return db::get_currencylist(pools);
}
fn calc_sin_avg(ratio: &dbm::Ratio) -> f32 {
    let offset = 0.7;
    let period = 1.0 * std::f32::consts::PI;
    let delta_cutoff = 0.20;
    let lower_delta_cutoff = 1.0 - delta_cutoff;
    let upper_delta_cutoff = 1.0 / (1.0 - delta_cutoff);
    let ratios = &ratio.ratios;
    let len = ratios.len() as f32;
    let mut start = (len / 7.0).floor() as i32;
    let mut sum: f32 = 0.0;
    let mut factor: f32 = 0.0;
    let mut dir = -1;
    let mut i = start;

    for _vittu_mika_kieli in 0..2 {
        while &i <= &(ratio.sellers as i32) && start > 0 && i > 0 {
            let delta = ratios[i as usize] / ratios[(i - dir) as usize];
            if (delta > 0.0 && delta < lower_delta_cutoff) || delta > upper_delta_cutoff {
                break;
            }
            let rad = (i + 1) as f32 / ratio.sellers as f32;
            let sin = f32::sin(rad.powf(offset) * period);
            sum += ratios[i as usize] * sin;
            factor += sin;

            i += dir;
            start -= 1;
        }
        i = start;
        dir = 1;
    }
    trace!("Calculated sine average {}", sum/factor);
    return sum / factor;
}
fn create_currency_pairs(pools: &mysql::Pool) -> Vec<dbm::CurrencyPair> {
    let currency = get_currencylist(pools);
    let mut retval: Vec<dbm::CurrencyPair> = Vec::new();

    trace!("Amount of currencies {}", currency.len());

    for a in &currency {
        for b in &currency {
            if a.get_id() != b.get_id() {
                retval.push(dbm::CurrencyPair::new_from_currencies(a.clone(), b.clone()))
            }
        }
    }
    info!("Amount of currency pairs created {}", retval.len());
    return retval;
}

fn scrape_bunched(
    league: &str,
    pair: &dbm::CurrencyPairBunched,
    re: &regex::Regex,
) -> Result<Vec<dbm::BunchRatio>, reqwest::Error> {
    let mut list: Vec<dbm::BunchRatio> = Vec::new();

    for chunk in pair.from.chunks(5) {
        let mut url: String = "http://currency.poe.trade/search?league=".to_string();
        url.push_str(league);
        url.push_str("&online=x&want=");
        url.push_str(&pair.to.get_id().to_string());

        url.push_str("&have=");
        for piece in chunk {
            url.push_str(&piece.get_id().to_string());
            url.push_str("-");
        }
        url.pop();
        debug!("Scraping : {}", url );

        let mut ratio = dbm::Ratio::new();

        ratio.ratios = Vec::new();

        let body = reqwest::get(&url)?.text()?;

        if re.captures_len() > 0 {
            for cap in re.captures_iter(&body) {
                /*
                    Group 1.	438052-438061	`HauntnBoo` username
                    Group 2.	438082-438083	`4` Sell currency
                    Group 3.	438101-438105	`60.0` sell Value
                    Group 4.	438125-438126	`6` Buy Currency
                    Group 5.	438143-438146	`1.0` Buy-Value
                    Group 6.	438148-438190	`data-ign="IlearnedToDodge" data-stock="60"`
                */
                let mut found = false;
                let mut foundIter = 0;
                let ratio = &cap[3].parse::<f32>().unwrap() / &cap[5].parse::<f32>().unwrap();
                let from = &cap[4].parse::<u32>().unwrap();
                if list.len() <= 0 {
                    let mut rats: Vec<f32> = Vec::new();
                    rats.push(ratio.clone());
                    list.push(dbm::BunchRatio::new(from.clone(), rats.clone()));
                } else {
                    for i in 0..list.len() {
                        if &list[i].get_id() == from {
                            found = true;
                            foundIter = i;
                            break;
                        }
                    }
                    if found {
                        list[foundIter].add_ratio(ratio.clone());
                    } else {
                        let mut rats: Vec<f32> = Vec::new();
                        rats.push(ratio.clone());
                        list.push(dbm::BunchRatio::new(from.clone(), rats.clone()));
                    }
                }
            }
        }
    }

    trace!("Scraping done : {:?}", pair);

    Ok(list)
}

pub fn scrape_all_bunched_parallel(league: &str) {
    let pools = db::create_connection_pool();
    let pull = db::get_pull(&pools);
    let now = chrono::Local::now().naive_local();
    let duration = pull.expires.signed_duration_since(now).num_minutes();

    if duration > 0 {
        warn!("{} minutes until next allowed scrape", duration);
        return;
    }

    use regex::Regex;
    let re = Regex::new("<div class=\"displayoffer \" data-username=\"(.*?)\" data-sellcurrency=\"(.*?)\" data-sellvalue=\"(.*?)\" data-buycurrency=\"(.*?)\" data-buyvalue=\"(.*?)\" (.*?)>").unwrap();

    let currency_pairs: Vec<dbm::CurrencyPair> = create_currency_pairs(&pools);

    let cur_pair_bunched = Arc::new(Mutex::new(Vec::new()));
    let mut fromList = Arc::new(Mutex::new(Vec::new()));

    for iter in get_currencylist(&pools) {
        let to = iter.clone();
        for from in get_currencylist(&pools) {
            if to.get_id() == from.get_id() {
                continue;
            }
            fromList.lock().unwrap().push(from.clone());
        }
        cur_pair_bunched
            .lock()
            .unwrap()
            .push(dbm::CurrencyPairBunched {
                from: fromList.lock().unwrap().clone(),
                to: to.clone(),
            });
        fromList = Arc::new(Mutex::new(Vec::new()));
    }

    let ratiopair = Arc::new(Mutex::new(Vec::new()));
    let cpb = cur_pair_bunched.lock().unwrap().clone();
    cpb.par_iter()
        .map(|current| {
            let mtcpb = &current.clone();
            match scrape_bunched(league, &mtcpb, &re) {
                Ok(r) => {
                    for iter in r {
                        let mut ratio = dbm::Ratio::new();
                        ratio.ratios = iter.get_ratios();
                        ratio.sum = ratio.ratios.iter().sum();
                        ratio.sellers = ratio.ratios.len() as u32;
                        ratio.average = ratio.sum / iter.get_len() as f32;
                        ratio.sin_average = calc_sin_avg(&ratio).clone();

                        ratiopair.lock().unwrap().push(dbm::RatioPair {
                            pair: dbm::CurrencyPair {
                                from: db::get_currency(iter.get_id(), &pools),
                                to: current.to.clone(),
                            },
                            ratio: ratio.clone(),
                        });
                    }
                }
                Err(e) => error!("{:?}", e),
            }

            thread::sleep(Duration::from_secs((&currency_pairs.len() / 500) as u64));
        })
        .collect::<Vec<_>>();

    db::create_ratio(&ratiopair.lock().unwrap(), &db::get_pull(&pools), &pools);
}

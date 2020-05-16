#![feature(type_ascription, extern_prelude)]
#![feature(vec_remove_item)]
#![allow(non_snake_case)]
#![allow(dead_code)]
extern crate chrono;
extern crate itertools;
#[macro_use]
extern crate mysql;
extern crate rayon;
extern crate regex;
extern crate reqwest;

extern crate serde;
extern crate serde_json;

extern crate time;

#[macro_use] extern crate log;


use recursivesearch as rs;

pub mod db;
pub mod models;
pub mod recursivesearch;
pub mod scraper;

fn scrapeit(args: String) {
    scraper::scrape_all_bunched_parallel(&args);
}
fn calculate_trades(
    current_amount: u32,
    investing_id: u32,
    exiting_id: u32,
    invest_amount: u32,
    min_sellers: u32,
    max_depth: u32,
) {
    let mut ruleset = rs::RuleSet {
        current_amount: current_amount,
        investing_id: investing_id,
        exiting_id: exiting_id,
        invest_amount: invest_amount,
        min_sellers: min_sellers,
        max_depth: max_depth,
        pool: db::create_connection_pool(),
    };

    let result = recursivesearch::Calculate(&mut ruleset);
}

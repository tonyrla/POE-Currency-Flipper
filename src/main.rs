#![feature(type_ascription, extern_prelude)]
#![feature(vec_remove_item)]
#![allow(non_snake_case)]
#![allow(dead_code)]

extern crate PoeFlippingLib;
extern crate time;

use pfl::db;
use pfl::recursivesearch as rs;
use PoeFlippingLib as pfl;

#[macro_use] extern crate log;
extern crate simplelog;
use simplelog::*;
use std::fs::File;

use time::PreciseTime;
fn timed_function(f: ()) {
    let start = PreciseTime::now();
    f;
    let end = PreciseTime::now();
    let mut value = start.to(end).to_string();
    value.pop();
    value.pop();
    println!("Run time {} s.", value);
}

fn init_logger(){
    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Debug, Config::default()).unwrap(),
            WriteLogger::new(LevelFilter::Info, Config::default(), File::create("debug.log").unwrap()),
        ]
    ).unwrap();
}

fn main() {

    //scraper::scrape_all("Hardcore+Incursion");
    use std::env;
    let args: Vec<String> = env::args().collect();
    println!("{:?} len {}", args, args.len());
    init_logger();


    debug!("Starting scraper");
    timed_function(pfl::scraper::scrape_all_bunched_parallel("Hardcore+Legion"));


    let mut ruleset = rs::RuleSet {
        current_amount: 0,
        investing_id: 4,
        exiting_id: 4,
        invest_amount: 25,
        min_sellers: 2,
        max_depth: 7,
        pool: db::create_connection_pool(),
    };

     debug!("Starting calculations");
    let results = rs::Calculate(&mut ruleset);
     debug!("Printing {} results", results.len());
    let mut i = 0;
    for r in results {
        if i >= 10 {
            break;
        }
        rs::print_result(r, &mut ruleset);
        i+=1;
    }

}

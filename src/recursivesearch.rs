use db;
use models as dbm;
use rayon::prelude::*;

pub struct RuleSet {
    pub investing_id: u32,
    pub exiting_id: u32,
    pub invest_amount: u32,
    pub current_amount: u32,
    pub min_sellers: u32,
    pub max_depth: u32,
    pub pool: mysql::Pool,
}

#[derive(Clone, Debug, Default)]
pub struct Result {
    chain: Vec<Node>,
    finalCount: f32,
    profit_pct: f32,
    finalRatio: f32,
    invest_amount: u32,
}
#[derive(Clone, Debug, Default)]
pub struct Node {
    currency: u32,
    prev: Option<Box<Node>>,
    ratio: f32,
    sellers: u32,
    sell: bool,
    finalCount: f32,
    finalRatio: f32,
    cleanRatio: String,
}
fn SaveSearch(pull: dbm::Pull, args: &RuleSet) {
    info!(
        "Saving {} {} to {} min: {} amount {}",
        pull.id, args.investing_id, args.exiting_id, args.min_sellers, args.invest_amount
    );
}
fn BestResult(results: &Vec<Result>) -> Result {
    let mut best = Result::default();
    for r in results {
        if r.profit_pct > best.profit_pct {
            best = r.clone();
        }
    }

    return best;
}

pub fn Calculate(args: &mut RuleSet) -> Vec<Result> {
    let pool = db::create_connection_pool();
    let ratios = db::get_ratios(&pool);
    let currencies = db::get_currencylist(&pool);
    let mut bestResult: Result = Result::default();
    let mut results: Vec<Result> = Vec::new();
    let mut analyzed = 0;
    let mut result: Result;

    let pullid = db::get_pull(&pool).id;
    debug!("{} ratios, {} currencies from pullid {}", ratios.len(), currencies.len(), pullid );
    for r in &ratios{
        debug!("    (#{}) {} -> {} = {}", r.ratio.pull_id, r.pair.from.name, r.pair.to.name, r.ratio.sin_average);
    } 

    for i in 1..args.invest_amount + 1 {
        analyzed = 0;
        //results = Vec::new();
        args.current_amount = i;
        let mut tree = Node {
            currency: args.investing_id,
            prev: None,
            ratio: 0.0,
            finalCount: i as f32,
            finalRatio: 1.0,
            cleanRatio: "".to_string(),
            sellers: 0,
            sell: true,
        };

        GenerateCombinations(
            &currencies,
            &mut tree,
            &mut 0,
            args,
            &mut analyzed,
            &mut results,
            &ratios,
        );
        result = BestResult(&results);
        if result.profit_pct > bestResult.profit_pct {
            bestResult = result;
        }
    }

    println!("{} results", results.len() );

    SaveSearch(db::get_pull(&args.pool), args);

    results.sort_by(|a, b| b.profit_pct.partial_cmp(&a.profit_pct).unwrap());
    return results;
}

pub fn print_result(r: Result, args: &mut RuleSet) {
    println!(
        "{} {} to {} {} @ {}% profit",
        r.invest_amount, //args.invest_amount,
        db::get_currency(args.investing_id, &args.pool).name,
        r.finalCount,
        db::get_currency(args.exiting_id, &args.pool).name,
        r.profit_pct,
    );
    for c in r.chain {
        let t = if c.sell { "Myy" } else { "Sijoita" };
        let action = match c.prev {
            Some(v) => *v,
            None => continue,
        };

        println!(
            "    {} {} {} vs {} {}",
            t,
            action.finalCount,
            db::get_currency(action.currency, &args.pool).name,
            c.finalCount,
            db::get_currency(c.currency, &args.pool).name
        );
    }
}
fn GenerateCombinations(
    lastCurrencies: &Vec<dbm::Currency>,
    prev: &mut Node,
    depth: &mut u32,
    args: &mut RuleSet,
    analyzed: &mut u32,
    results: &mut Vec<Result>,
    ratios: &Vec<dbm::RatioPair>,
) {
    *depth += 1;
    if depth > &mut args.max_depth.clone() || prev.finalCount <= 0.0 {
        debug!("[{}] Returning with depth: {} prev.finalCount: {}", prev.currency, depth, prev.finalCount);
        return;
    }

    let mut currencies = lastCurrencies.clone();
    let mut index = 999;
    for i in 0..currencies.len() {
        if currencies[i].get_id() == prev.currency {
            index = i;
            break;
        }
    }
    if index < 999 {
        currencies.remove(index);
    }
    /*
    currencies.par_iter_mut().for_each(|c| {
        if let Some(next) = MakeTreeNode(c.get_id(), prev, analyzed, ratios) {
            GenerateCombinations(
                &currencies,
                &mut next.clone(),
                depth,
                args,
                analyzed,
                results,
                ratios,
            );
        }
    });*/

    for c in currencies.clone() {
        if let Some(next) = MakeTreeNode(c.get_id(), prev, analyzed, ratios) {
            GenerateCombinations(
                &currencies,
                &mut next.clone(),
                depth,
                args,
                analyzed,
                results,
                ratios,
            );
        }
    }
    
    if prev.currency == args.exiting_id {
        debug!("Currency ID matches target ID, returning. {} = {}", prev.currency, args.exiting_id);
        return;
    }
    let next = MakeTreeNode(args.exiting_id, prev, analyzed, ratios);
    match next {
        Some(v) => {
            let mut n = v;
            GenerateCombinations(&currencies, &mut n, depth, args, analyzed, results, ratios);

            AddResult(n, results, &args.current_amount);
        }
        None => {}
    }
}
fn MakeTreeNode(
    currency: u32,
    prev: &mut Node,
    analyzed: &mut u32,
    ratios: &Vec<dbm::RatioPair>,
) -> Option<Node> {
    *analyzed += 1;
    let mut sellRatioSet = true;
    let mut buyRatioSet = false; //Sijoita

    let sellRatio = match get_ratio(currency, prev.currency, ratios) {
        Some(v) => v,
        None => {
            debug!("[{}->{}]No sellratio", currency, prev.currency);
            sellRatioSet = false;
            dbm::Ratio::default()
        }
    };
    let buyRatio = match get_ratio(prev.currency, currency, ratios) {
        Some(v) => v,
        None => {
            debug!("[{}->{}]No buyratio", prev.currency, currency);
            buyRatioSet = false;
            dbm::Ratio::default()
        }
    };

    let mut ratio: dbm::Ratio = dbm::Ratio::default();

    if !sellRatioSet && !buyRatioSet {
        debug!("[{}->{}]No sell or buy ratio, returning None",currency, prev.currency);
        return None;
    }
    if buyRatioSet && sellRatioSet {
        ratio = if buyRatio.sin_average >= (1.00 / sellRatio.sin_average) {
            buyRatio.clone()
        } else {
            sellRatio.clone()
        }
    } else {
        if buyRatioSet {
            ratio = buyRatio.clone();
        } else {
            ratio = sellRatio.clone();
        }
    }

    if ratio.sin_average <= 0.0 {
        debug!("[{}->{}] No ratio, returning None.", prev.currency, currency);
        return None;
    }

    let sell = ratio.sin_average == sellRatio.sin_average;
    let finalCount = if sell {
        prev.finalCount * (1.0 / ratio.sin_average)
    } else {
        prev.finalCount * ratio.sin_average
    };
    let finalRatio = if sell {
        prev.finalRatio * (1.0 / ratio.sin_average)
    } else {
        prev.finalRatio * ratio.sin_average
    };
    let tree = Node {
        currency: currency,
        prev: Some(Box::new(prev.clone())),
        ratio: ratio.sin_average,
        sell: sell,
        finalCount: finalCount.floor(),
        sellers: ratio.sellers,
        finalRatio: finalRatio,
        cleanRatio: finalRatio.to_string(),
    };

    return Some(tree);
}
fn AddResult(tree: Node, results: &mut Vec<Result>, invest_amount: &u32) {
    if tree.finalCount <= 0.0 {
        debug!("[{}] finalcount <= 0, returning.", tree.currency);
        return;
    }
    let profit_pct = (tree.finalCount.clone() / *invest_amount as f32 * 100.00) - 100.00;
    if profit_pct <= 0.0 {
        debug!("[{}] profit <= 0, finalcount = {}, returning.", tree.currency, tree.finalCount);
        return;
    }
    let mut curr = tree.clone();
    let mut result = Result {
        chain: Vec::new(),
        invest_amount: invest_amount.clone(),
        finalCount: curr.finalCount.clone(),
        profit_pct: profit_pct,
        finalRatio: curr.finalRatio,
    };

    while let Some(v) = curr.prev.clone() {
        if let Some(p) = v.prev.clone() {
            //    if let Some(f) = p.prev.clone() {
            if p.currency == curr.currency && p.finalCount == curr.finalCount {
                curr = *p;
                continue;
            }
            //    }
        }

        result.chain.insert(
            0,
            Node {
                currency: curr.currency.clone(),
                prev: Some(v.clone()),
                ratio: curr.ratio.clone(),
                finalCount: curr.finalCount,
                finalRatio: curr.finalRatio,
                cleanRatio: (curr.finalCount / v.finalCount.clone()).to_string(),
                sellers: curr.sellers,
                sell: curr.sell,
            },
        );

        curr = *v;
    }

    for i in 0..results.len() {
        if results[i].finalCount > result.finalCount {
            continue;
        }
        results.insert(i, result.clone());
        return;
    }
    warn!("Adding result: {:?}", result);
    results.push(result);
}
fn get_ratio(from: u32, to: u32, ratios: &Vec<dbm::RatioPair>) -> Option<dbm::Ratio> {
    for rp in ratios {
        if rp.pair.from.get_id() == from && rp.pair.to.get_id() == to {
            return Some(rp.ratio.clone());
        }
    }
    return None;
}

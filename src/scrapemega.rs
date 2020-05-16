//200 tulosta maksimi, ei toimi.
#[derive(Clone, Debug)]
pub struct BunchRatioMega{
    pub to: u32,
    pub ratios: Vec<f32>,
    pub from: u32
}
fn scrape_all_mega(league: &str) {
    use regex::Regex;
    let re = Regex::new("<div class=\"displayoffer \" data-username=\"(.*?)\" data-sellcurrency=\"(.*?)\" data-sellvalue=\"(.*?)\" data-buycurrency=\"(.*?)\" data-buyvalue=\"(.*?)\" (.*?)>").unwrap();
    let currencies = get_currencylist();
    let mut fromList: Vec<dbm::Currency> = Vec::new();
    let mut toList: Vec<dbm::Currency> = Vec::new();

    for iter in &currencies {
        fromList.push(iter.clone());
        toList.push(iter.clone());
    }
    match scrape_mega(league, toList, fromList, &re) {
        Ok(_r) => { println!("Tuloksia {:?}", _r.len());}
        Err(_e) => {}
    }
}

fn scrape_mega(
    league: &str,
    toList: Vec<dbm::Currency>,
    fromList: Vec<dbm::Currency>,
    re: &regex::Regex
) -> Result<Vec<dbm::RatioPair>, reqwest::Error> {

    let mut url: String = "http://currency.poe.trade/search?league=".to_string();


    url.push_str(league);
    url.push_str("&online=x&want=");
    for iter in &toList {
        url.push_str(&iter.id.to_string());
        url.push_str("-");
    }
    url.pop();

    url.push_str("&have=");
    for iter in &fromList {
        url.push_str(&iter.id.to_string());
        url.push_str("-");
    }
    url.pop();

    let mut ratio = dbm::Ratio {
        sum: 0.0,
        ratios: Vec::new(),
        sellers: 0,
        average: 0.0,
        sin_average: 0.0,
    };

    println!("{}", url);
    ratio.ratios = Vec::new();
    let mut megalist: Vec<dbm::BunchRatioMega> = Vec::new();


    let body = reqwest::get(&url)?.text()?;

    /*
    Group 1.	438052-438061	`HauntnBoo` username
    Group 2.	438082-438083	`4` Sell currency
    Group 3.	438101-438105	`60.0` sell Value
    Group 4.	438125-438126	`6` Buy Currency
    Group 5.	438143-438146	`1.0` Buy-Value
    Group 6.	438148-438190	`data-ign="IlearnedToDodge" data-stock="60"`
    */
    if re.captures_len() > 0 {
        for cap in re.captures_iter(&body) {
            let to = &cap[2].parse::<u32>().unwrap();
            let ratio = &cap[3].parse::<f32>().unwrap()/&cap[5].parse::<f32>().unwrap();
            let from = &cap[4].parse::<u32>().unwrap();
            //println!("{} to {} with ratio of {}", from, to, ratio );
            let mut found = false;

            for iter in 0..megalist.len() {
                if &megalist[iter].to == to && &megalist[iter].from == from {
                    found = true;
                    &megalist[iter].ratios.push(ratio.clone());
                    //break;
                }
            }
            if found{
                continue;
            }
            let mut new = dbm::BunchRatioMega{
                to: to.clone(),
                ratios: Vec::new(),
                from: from.clone(),
            };
            new.ratios.push(ratio.clone());
            megalist.push(new.clone());
        }
    }

    let mut ratio_pairs: Vec<dbm::RatioPair> = Vec::new();
    let currency_list = db::get_currencylist();
    for iter in megalist{
        let mut to: dbm::Currency = dbm::Currency::new();
        let mut from: dbm::Currency = dbm::Currency::new();
        let mut totesti = &currency_list.iter().position(|r| r.id == iter.to).unwrap();
        let mut fromtesti = &currency_list.iter().position(|r| r.id == iter.from).unwrap();
        match currency_list.iter().nth(*totesti){
            None => {}
            Some(r) => {to = r.clone()}
        }
        match currency_list.iter().nth(*fromtesti){
            None => {}
            Some(r) => {from = r.clone()}
        }
        //println!("{} ja {}", to, from);

        let cpair: dbm::CurrencyPair = dbm::CurrencyPair{
            to: to,
            from: from,
        };
        let mut ratio = dbm::Ratio::new();
        ratio.ratios = Vec::new();
        ratio.sum = iter.ratios.iter().sum();
        ratio.ratios = iter.ratios.clone();
        ratio.sellers = ratio.ratios.len() as u32;
        ratio.average = ratio.sum / iter.ratios.len() as f32;
        ratio.sin_average = calc_sin_avg(&ratio).clone();


        //println!("{} vs {} -> Sellers: {} Sum: {} average: {} sin average: {} -> Cheapest: {} and most expensive: {}", &cpair.to, &cpair.from, &ratio.ratios.len(), &ratio.sum, &ratio.average, &ratio.sin_average, &ratio.ratios[0], &ratio.ratios[&ratio.ratios.len()-1]);

        ratio_pairs.push(dbm::RatioPair{
            pair: cpair,
            ratio: ratio,

        });

    }

     Ok(ratio_pairs)
}

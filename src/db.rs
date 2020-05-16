use models;
//#[macro_use]
use mysql as my;

pub fn create_connection_pool() -> mysql::Pool {
    let pool = my::Pool::new("mysql://online:n4kk1!@192.168.0.138/poe_currency").unwrap();

    return pool;
}

pub fn get_currency(id: u32, pools: &my::Pool) -> models::Currency {
    let mut pool = pools.get_conn().unwrap();
    let pull: Vec<models::Currency> =
        pool.prep_exec(
            "SELECT * FROM currencies WHERE id=:id",
            params!{
                    "id" => id,
            },
        ).map(|result| {
                result
                    .map(|x| x.unwrap())
                    .map(|row| {
                        let (cid, name, css, active, ca, ua) = my::from_row(row);
                        models::Currency::new_from_data(cid, name, css, active, ca, ua)
                    })
                    .collect()
            })
            .unwrap();

    return pull[0].clone();
}
pub fn get_currencylist(pools: &my::Pool) -> Vec<models::Currency> {
    let mut pool = pools.get_conn().unwrap();
    let currencies: Vec<models::Currency> = pool
        .prep_exec("SELECT * FROM currencies WHERE active = TRUE", ())
        .map(|result| {
            result
                .map(|x| x.unwrap())
                .map(|row| {
                    let (id, name, css, active, ca, ua) = my::from_row(row);
                    models::Currency::new_from_data(id, name, css, active, ca, ua)
                })
                .collect() // mappaa Currency vektoriin
        })
        .unwrap();

    return currencies;
}
pub fn get_ratios(pools: &my::Pool) -> Vec<models::RatioPair> {
    let mut pool = pools.get_conn().unwrap();

    let pullid = get_pull(pools).id;
    let pull: Vec<models::RatioPair> =
        pool.prep_exec(
            "SELECT * FROM currency_ratios WHERE pull_id=:pullid",
            params!{
                    "pullid" => pullid,
            },
        ).map(|result| {
                result
                    .map(|x| x.unwrap())
                    .map(|row| {
                        let (id, _pullid, fromid, toid, ratio, sellers, ca) = my::from_row(row);
                        let _id: u32 = id;
                        let _pully: i32 = _pullid;
                        let _ca: mysql::chrono::NaiveDate = ca;
                        models::RatioPair {
                            pair: models::CurrencyPair {
                                from: get_currency(fromid, pools),
                                to: get_currency(toid, pools),
                            },
                            ratio: models::Ratio {
                                sum: ratio,
                                ratios: Vec::new(),
                                sellers: sellers,
                                average: ratio,
                                sin_average: ratio,
                                pull_id: _pully as u32,
                            },
                        }
                    })
                    .collect()
            })
            .unwrap();

    return pull;
}
pub fn get_ratio_by_currency(
    to: models::Currency,
    from: models::Currency,
    pools: &my::Pool,
) -> models::RatioPair {
    let mut pool = pools.get_conn().unwrap();
    let pull: Vec<models::RatioPair> = pool.prep_exec(
        "SELECT pull_id,currency_from_id,currency_to_id,ratio,sellers FROM currency_ratios WHERE pull_id = (SELECT MAX(pull_id) FROM currency_ratios) AND currency_from_id=:fromid AND currency_to_id=:toid",
        params!{
            "fromid" => from.get_id(),
            "toid" => to.get_id(),
    },
    ).map(|result|{
        result
        .map(|x| x.unwrap())
        .map(|row| {
            let (pullid, fromid, toid, ratio, sellers) = my::from_row(row);
            let _pully: i32 = pullid;
            let _frommy: i32 = fromid;
            let _tommy: i32 = toid;
            models::RatioPair{
                pair: models::CurrencyPair{
                    from: from.clone(),
                    to: to.clone(),
                },
                ratio: models::Ratio {
                    sum: ratio,
                    ratios: Vec::new(),
                    sellers: sellers,
                    average: ratio,
                    sin_average: ratio,
                    pull_id: _pully as u32,
                }


            }
            })
            .collect()
        })
        .unwrap();

    return pull[0].clone();
}
pub fn get_ratio(from: u32, to: u32, pools: &my::Pool) -> models::RatioPair {
    let cur_to = get_currency(to, pools);
    let cur_from = get_currency(from, pools);
    let mut pool = pools.get_conn().unwrap();

    let pull: Vec<models::RatioPair> = pool.prep_exec(
        "SELECT pull_id,currency_from_id,currency_to_id,ratio,sellers FROM currency_ratios WHERE pull_id = (SELECT MAX(pull_id) FROM currency_ratios) AND currency_from_id=:fromid AND currency_to_id=:toid",
        params!{
            "fromid" => from,
            "toid" => to,
    },
    ).map(|result|{
        result
        .map(|x| x.unwrap())
        .map(|row| {
            let (pullid, fromid, toid, ratio, sellers) = my::from_row(row);
            let _pully: i32 = pullid;
            let _frommy: i32 = fromid;
            let _tommy: i32 = toid;
            models::RatioPair{
                pair: models::CurrencyPair{
                    from: cur_from.clone(),
                    to: cur_to.clone(),
                },
                ratio: models::Ratio {
                    sum: ratio,
                    ratios: Vec::new(),
                    sellers: sellers,
                    average: ratio,
                    sin_average: ratio,
                    pull_id: _pully as u32,
                }


            }
            })
            .collect()
        })
        .unwrap();

    return pull[0].clone();
}
pub fn get_pull(pools: &my::Pool) -> models::Pull {
    let mut pool = pools.get_conn().unwrap();
    trace!("Getting pull id");
    let pull: Vec<models::Pull> = pool
        .prep_exec("SELECT id,league_id,expires_at,_ca FROM pulls WHERE id = (SELECT MAX(id) FROM pulls) ", ())
        .map(|result| {
            result
                .map(|x| x.unwrap())
                .map(|row| {
                    let (id, lid, exp, ca) = my::from_row(row);
                    models::Pull {
                        id: id,
                        league_id: lid,
                        expires: exp,
                        called_at: ca,
                    }
                })
                .collect() // map to Currency
        })
        .unwrap();

    let retVal: models::Pull = models::Pull {
        id: pull[0].id,
        league_id: pull[0].league_id,
        expires: pull[0].expires,
        called_at: pull[0].called_at,
    };

    return retVal;
}
pub fn create_ratio(ratiopair: &Vec<models::RatioPair>, pull: &models::Pull, pools: &my::Pool) {
    let pullID = pull.id + 1;
    let now = chrono::Local::now();
    let mut pool = pools.get_conn().unwrap();
    let expires =now + chrono::Duration::minutes(15);
    info!(
        "Creating ratios for {} currency pairs with Pull Id:{}",
        ratiopair.len(),
        pullID
    );
    pool.prep_exec(
        r"INSERT INTO Pulls
                            (league_id, expires_at)
                            VALUES
                            (:league_id, :expires_at)",
        params!{
            "league_id" => 5,
            "expires_at" => expires.to_string(),
        },
    ).unwrap();

    println!("Creating ratios done.");

    for mut stmt in pool.prepare(r"INSERT INTO currency_ratios
                                       (pull_id, currency_from_id, currency_to_id, ratio, sellers)
                                   VALUES
                                       (:pull_id, :currency_from_id, :currency_to_id, :ratio, :sellers)").into_iter() {
        for p in ratiopair.iter() {
            stmt.execute(params!{
                "pull_id" => &pullID,
                "currency_from_id" => p.pair.from.get_id(),
                "currency_to_id" => p.pair.to.get_id(),
                "ratio" =>  p.ratio.sin_average,
                "sellers" => p.ratio.ratios.len(),
            }).unwrap();
        }
    }
}

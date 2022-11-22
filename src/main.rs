pub mod util;
use chrono::NaiveDate;
use util::*;
use std::vec;
use std::{str::FromStr, array};
use std::result::Result;
use std::error::Error;
use surreal_simple_client::{SurrealClient, rpc};
use serde_json::{json, Value};
use serde::Deserialize;
use chrono::{offset::Utc, NaiveDateTime};

#[tokio::main]
async fn main() {
    // get_volumes_by_chain_day("2022_17_11", "stride").await; GOOD
    // get_chain_aggregate_volumes("stride").await; GOOD
    // get_transaction("2fc679d2-bd95-4e45-9529-401debfa8c34".to_string()).await; GOOD
    // get_previous_x_transactions(None, None).await;
    // get_previous_transactions_by_chain(None, None, "stride").await;
    // get_previous_transactions_by_denom(None, None, "SCRT").await;
    // get_previous_transactions_by_denom_and_chain(None, None, "SCRT", "stride").await;
    // get_num_unique_wallets().await; GOOD
    // post_transaction(origin, destination, amount, denom, price, secret_address)
        // test_json();
}

async fn access_local_surrealdb() {
    let mut client = SurrealClient::new("ws://localhost:8000/rpc")
        .await
        .expect("RPC handshake error");

    client.signin("root", "root").await.expect("Signin error");
    client.use_namespace("kwahl", "sampleDBname").await.expect("Namespace error");

    // TODO: Parse entered date-time into just the date for volume indexing
    // let date = "2022-11-07".to_string();

    let response = client
        .send_query(
            "
            BEGIN TRANSACTION;
            CREATE transaction SET date = $date, origin = $origin, destination = $destination, amount = $amount, denom = $denom, transaction_num = $transaction_num;
            UPDATE chain:$origin SET transactions +=1;
            UPDATE $origin SET transactions +=1, amount +=$amount;
            UPDATE chain:$destination SET transactions +=1;
            UPDATE $destination SET transactions +=1, amount +=$amount;
            INSERT INTO volume (id, date, total_volume, $origin_origin, $destination_destination) 
                VALUES ($date, $date, $amount, $amount) 
                ON DUPLICATE KEY UPDATE 
                total_volume += $amount, 
                $origin_origin += $amount,
                $destination_destination += $amount;
            COMMIT TRANSACTION;
            "
            .to_owned(), 
            json!({
                "date": "2022_11_10",
                "origin": "origin:evmos",
                "destination": "destination:injective",
                "amount": 7000000.0,
                "denom": "SCRT",
                "transaction_num": 2,
            })
        ).await;

    match response {
        Ok(surrealResp) => {
            print!("DB Responded");
        },
        Err(rpcError) => {
            print!("RPC Error.");
        },
    };

}

// pub struct Array(pub Vec<Value>);

// impl IntoIterator for Array {
// 	type Item = Value;
// 	type IntoIter = std::vec::IntoIter<Self::Item>;
// 	fn into_iter(self) -> Self::IntoIter {
// 		self.0.into_iter()
// 	}
// }
// pub trait Take {
// 	fn needs_one_or_two(self) -> Result<(Value, Value), ()>;
// }

// impl Take for Array {
//     fn needs_one_or_two(self) -> Result<(Value, Value), ()> {
//         let mut x = self.into_iter();
//         match (x.next(), x.next()) {
//             (Some(a), Some(b)) => Ok((a, b)),
//             (Some(a), None) => Ok((a, Value::None)),
//             (_, _) => Ok((Value::None, Value::None)),
//         }
//     }
// }


fn test_json() {
    let query = "CREATE User SET username = $username";
    let params = json!({
        "username": "Ky:le"
    });
    let json = json!([query, params]);
    let result = json.to_string();
    println!("{}", result);
}
//     match result.needs_one_or_two() {
//         Ok((Value::Strand(s), o)) if o.is_none() => {
//             return match rpc.read().await.query(s).await {
//                 Ok(v) => res::success(id, v).send(out, chn).await,
//                 Err(e) => {
//                     res::failure(id, Failure::custom(e.to_string())).send(out, chn).await
//                 }
//             };
//         }
//         Ok((Value::Strand(s), Value::Object(o))) => {
//             return match rpc.read().await.query_with(s, o).await {
//                 Ok(v) => res::success(id, v).send(out, chn).await,
//                 Err(e) => {
//                     res::failure(id, Failure::custom(e.to_string())).send(out, chn).await
//                 }
//             };
//         }
//     }
//     println!("Tested");
//     println!("{}", result);
// }



async fn post_transaction(origin: &str, destination: &str, amount: f64, denom: &str, price: f64, secret_address: &str) {
    let mut client = get_client().await;

    let o = SupportedChains::from_str(origin).unwrap();
    let d = SupportedChains::from_str(destination).unwrap();
    let t = SupportedTokens::from_str(denom).unwrap();
    
    // Work can happen here with the identified chains and tokens, if need be.

    let sql_o_vol = sql_chain_token_vol_format(&o, &t);
    let sql_d_vol = sql_chain_token_vol_format(&d, &t);
    let sql_o = sql_chain_format(&o);
    let sql_d = sql_chain_format(&d);

    let date = Utc::now().date_naive().to_string(); // Retrieves current day without timezone information
    // TODO: Need logic for transaction num. Test surrealQL functions, probs
    let response = client
    .send_query(
    "
    BEGIN TRANSACTION;
    LET $uuid = rand::uuid();
    CREATE transaction SET datetime = $date, origin = $origin, destination = $destination, amount = $amount, denom = $denom, uuid = $uuid, price = $price;
    LET $t = (SELECT * FROM transaction WHERE transaction_uuid = $uuid);
    UPDATE $origin SET outgoing_transactions +=1, transactions = array::union(transactions, $t.id);
    UPDATE $destination SET incoming_transactions +=1, transactions = array::union(transactions, $t.id);
    UPDATE $origin_vol SET volume += { date: $date, incoming: 0, outgoing: 0 } WHERE volume[$].date != $date;
    UPDATE $origin_vol SET volume[$].outgoing += $amount, total_outgoing += $amount WHERE volume[$].date = $date;
    UPDATE $destination_vol SET volume += { date: $date, incoming: 0, outgoing: 0 } WHERE volume[$].date != $date;
    UPDATE $destination_vol SET volume[$].incoming += $amount, total_incoming += $amount WHERE volume[$].date = $date;
    INSERT INTO wallet (id, address) VALUES ($address, $address);
    COMMIT TRANSACTION;
    "
    .to_owned(),
    json!({
        "date": date,
        "origin": sql_o,
        "origin_vol": sql_o_vol,
        "destination": sql_d,
        "destination_vol": sql_d_vol,
        "amount": amount,
        "denom": denom.to_uppercase().as_str(),
        "price": price,
        "address": secret_address,
    })
    ).await;

    match response {
        Ok(surreal_resp) => {
            match surreal_resp.await {
                Ok(data) => {
                    if let Some(surreal_query_result) = data.get_nth_query_result(0){
                        let result = surreal_query_result.results();
                        match serde_json::from_value::<Transaction>(result[0].clone()){
                            Ok(transaction) => {
                                // TODO: Return a positive response? Something similar?
                            },
                            Err(err) => {
                                // Handle error
                            }
                        }
                    }
                },
                Err(recv_error) => {
                    // Handle recv error
                },
            }
        },
        Err(rpcError) => {
            // Handle RPC error
        },
    };
}

async fn get_transaction(tx_uuid: String) {
    let mut client = get_client().await;

    let response = client
    .find_one::<TxResponse>(
    "
    SELECT {amount: amount, datetime: datetime, denom: denom, origin: origin, destination: destination, transaction_uuid: transaction_uuid} AS `tx` FROM transaction WHERE transaction_uuid = $id
    "
    .to_owned(),
    json!({
        "id": tx_uuid
    })
    ).await;

    match response {
        Ok(resp) => {
            match resp {
                Some(tx_resp) => {
                    let tx = tx_resp.tx;
                    println!("{}", tx.amount)
                },
                None => todo!(),
            }
        },
        Err(rpcError) => {
            println!("{} RPC Error", rpcError)
        },
    };    

    // match response {
    //     Ok(surreal_resp) => {
    //         match surreal_resp.await {
    //             Ok(data) => {
    //                 if let Some(surreal_query_result) = data.get_nth_query_result(0){
    //                     let result = surreal_query_result.results();
    //                     match serde_json::from_value::<TxResponse>(result[0].clone()){
    //                         Ok(txResp) => {
    //                             println!("{}", txResp.tx.uuid)
    //                         },
    //                         Err(err) => {
    //                             println!("{}", err)
    //                         }
    //                     }
    //                 }
    //             },
    //             Err(recv_error) => {
    //                 // Handle recv error
    //             },
    //         }
    //     },
    //     Err(rpcError) => {
    //         println!("{}", rpcError)
    //     },
    // };

}

async fn get_volumes_by_chain_day(date: &str, chain: &str) {
    let mut client = get_client().await;

    let mut sql_chain = "".to_string();

    let c = valid_chain(chain);
    match c {
        Ok(valid) => {
            sql_chain = valid.surrealql_format(); // Will be formatted like "stride" rather than "chain:stride", which is important
        },
        Err(_) => todo!() // Flesh out error conditions still,
    }
    //         SELECT {vol: volume[WHERE date = $date], token: id} AS `daily_vol` FROM $chain;

    let response = client.find_many::<DailyVolumeVecResponse>(
        "
        SELECT {vol: volume[WHERE date = $date], token: id} AS `daily_vol` FROM type::table($chain);
        ".to_owned(),
        json!({
            "date": date.to_string(),
            "chain": sql_chain,
        })
    ).await;

    let mut output: Vec<FormattedDailyVolOutput> = vec![];

    // match response {
    //     Ok(surreal_resp) => {
    //         match surreal_resp.await {
    //             Ok(data) => {
    //                 if let Some(surreal_query_result) = data.get_nth_query_result(0){ // Has time, status, and result at this level
    //                     let result = surreal_query_result.results(); // Vec of DailiyVolumeVecResponse level
    //                     for value in result {
    //                         println!("{}", value.to_string());
    //                         match serde_json::from_value::<DailyVolumeVecResponse>(value.clone()){
    //                             Ok(vec_resp) => {
    //                                 if !vec_resp.daily_vol.vol.is_empty() {
    //                                     let o = FormattedDailyVolOutput {
    //                                         token: vec_resp.daily_vol.token,
    //                                         date: vec_resp.daily_vol.vol[0].date.clone(),
    //                                         incoming: vec_resp.daily_vol.vol[0].incoming,
    //                                         outgoing: vec_resp.daily_vol.vol[0].outgoing
    //                                     };
    //                                     output.push(o);
    //                                 }
    //                             },
    //                             Err(err) => {
    //                                 println!("{} err big time", err);
    //                                 continue;
    //                             }
    //                         }
    //                     }
    //                 }
    //             },
    //             Err(recv_error) => {
    //                 println!("{} recv err", recv_error)
    //             },
    //         }
    //     },
    //     Err(rpcError) => {
    //         println!("{} RPC Error", rpcError)
    //     },
    // };

    match response {
        Ok(resp) => {
            for vec_resp in resp {
                if !vec_resp.daily_vol.vol.is_empty() {
                    let o = FormattedDailyVolOutput {
                        token: vec_resp.daily_vol.token,
                        date: vec_resp.daily_vol.vol[0].date.clone(),
                        incoming: vec_resp.daily_vol.vol[0].incoming,
                        outgoing: vec_resp.daily_vol.vol[0].outgoing
                    };
                    output.push(o);
                }
            }
        },
        Err(rpcError) => {
            println!("{} RPC Error", rpcError)
        },
    };    

    // TODO: Return output
    println!("{}", output.pop().unwrap().date)
}

async fn get_chain_aggregate_volumes(chain: &str) {
    let mut client = get_client().await;

    let sql_chain;

    match valid_chain(chain){
        Ok(c) => {
            sql_chain = c.surrealql_format() // Will be "{chain_name}" formatted
        },
        Err(_) => todo!() // Exit execution,
    }

    let response = client.find_many::<AggregateVolVecResponse>(
        "
        SELECT {id: id, total_incoming: total_incoming, total_outgoing: total_outgoing} as `aggregate_vol` FROM type::table($chain);
        ".to_owned(),
        json!({
            "chain": sql_chain,
        })
    ).await;

    let mut output: Vec<AggregateVolume> = vec![];

    match response {
        Ok(resp) => {
            for vec_resp in resp {
                let o = AggregateVolume {
                    id: vec_resp.aggregate_vol.id,
                    total_incoming: vec_resp.aggregate_vol.total_incoming,
                    total_outgoing: vec_resp.aggregate_vol.total_outgoing
                };
                output.push(o);
            }
        },
        Err(rpcError) => {
            // Handle RPC error
            println!("{} RPC Error", rpcError)
        },
    };    

    // TODO: Return output
    println!("{}", output.pop().unwrap().id);
    println!("{}", output.pop().unwrap().total_incoming)    
}

async fn get_previous_x_transactions(limit: Option<u32>, start: Option<u32>) {
    let mut client = get_client().await;
    // MUST BE RUNNING surrealDB nightly, or a version greater than 1.0.0-beta.8+20220930.c246533
    let mut default_limit: u32 = 50;
    let mut default_start: u32 = 0;

    if let Some(l) = limit {
        default_limit = l;
    }

    if let Some(s) = start {
        default_start = s;
    }

    println!("Querying, also {} and {}", default_limit, default_start);
    let response = client.find_many::<Transaction>(
        "
        SELECT * FROM transaction ORDER BY datetime DESC LIMIT $limit START $start;
        ".to_owned(),
        json!({
            "limit": default_limit,
            "start": default_start
        })
    ).await;

    println!("Queried");
    let mut output: Vec<Transaction> = vec![];

    println!("Made it");
    match response {
        Ok(resp) => {
            for tx in resp {
                println!("{}", tx.amount);
                output.push(tx);
            }
        },
        Err(rpcError) => {
            // Handle RPC error
            println!("{} RPC Error", rpcError)
        },
    };    

    // TODO: Return output
    println!("{}", output.pop().unwrap().amount);
    println!("{}", output.pop().unwrap().datetime)  
}

async fn get_previous_transactions_by_chain(limit: Option<u32>, start: Option<u32>, chain: &str) {
    let mut client = get_client().await;
    // MUST BE RUNNING surrealDB nightly, or a version greater than 1.0.0-beta.8+20220930.c246533
    let mut default_limit: u32 = 50;
    let mut default_start: u32 = 0;

    if let Some(l) = limit {
        default_limit = l;
    }

    if let Some(s) = start {
        default_start = s;
    }

    // Chain formatting
    let sql_chain;

    match valid_chain(chain){
        Ok(c) => {
            sql_chain = sql_chain_format(&c);
        },
        Err(_) => todo!() // Exit execution,
    }

    println!("Querying, also {} and {}", default_limit, default_start);
    let response = client.find_many::<Transaction>(
        "
        SELECT * FROM transaction WHERE (origin = $chain OR destination = $chain) ORDER BY datetime DESC LIMIT $limit START $start;
        ".to_owned(),
        json!({
            "limit": default_limit,
            "start": default_start,
            "chain": chain
        })
    ).await;

    println!("Queried");
    let mut output: Vec<Transaction> = vec![];

    println!("Made it");
    match response {
        Ok(resp) => {
            for tx in resp {
                println!("{}", tx.amount);
                output.push(tx);
            }
        },
        Err(rpcError) => {
            // Handle RPC error
            println!("{} RPC Error", rpcError)
        },
    };    

    // TODO: Return output
    println!("{}", output.pop().unwrap().amount);
    println!("{}", output.pop().unwrap().datetime)  
}

async fn get_previous_transactions_by_denom(limit: Option<u32>, start: Option<u32>, denom: &str) {
    let mut client = get_client().await;
    // MUST BE RUNNING surrealDB nightly, or a version greater than 1.0.0-beta.8+20220930.c246533
    let mut default_limit: u32 = 50;
    let mut default_start: u32 = 0;

    if let Some(l) = limit {
        default_limit = l;
    }

    if let Some(s) = start {
        default_start = s;
    }

    let mut token: String = "".to_string();

    match valid_token(denom) {
        Ok(t) => {
            token = t.surrealql_format().to_uppercase();
        },
        Err(_) => todo!(),
    }

    println!("Querying, also {} and {}", default_limit, default_start);
    let response = client.find_many::<Transaction>(
        "
        SELECT * FROM transaction WHERE denom = $token ORDER BY datetime DESC LIMIT $limit START $start;
        ".to_owned(),
        json!({
            "limit": default_limit,
            "start": default_start,
            "token": token
        })
    ).await;

    println!("Queried");
    let mut output: Vec<Transaction> = vec![];

    println!("Made it");
    match response {
        Ok(resp) => {
            for tx in resp {
                println!("{}", tx.amount);
                output.push(tx);
            }
        },
        Err(rpcError) => {
            // Handle RPC error
            println!("{} RPC Error", rpcError)
        },
    };    

    // TODO: Return output
    println!("{}", output.pop().unwrap().amount);
    println!("{}", output.pop().unwrap().datetime)  
}

async fn get_previous_transactions_by_denom_and_chain(limit: Option<u32>, start: Option<u32>, denom: &str, chain: &str) {
    let mut client = get_client().await;
    // MUST BE RUNNING surrealDB nightly, or a version greater than 1.0.0-beta.8+20220930.c246533
    let mut default_limit: u32 = 50;
    let mut default_start: u32 = 0;

    if let Some(l) = limit {
        default_limit = l;
    }

    if let Some(s) = start {
        default_start = s;
    }

    let mut token: String = "".to_string();

    match valid_token(denom) {
        Ok(t) => {
            token = t.surrealql_format().to_uppercase();
        },
        Err(_) => todo!(),
    }

    // Chain formatting
    let sql_chain;

    match valid_chain(chain){
        Ok(c) => {
            sql_chain = sql_chain_format(&c);
        },
        Err(_) => todo!() // Exit execution,
    }

    println!("Querying, also {} and {}", default_limit, default_start);
    let response = client.find_many::<Transaction>(
        "
        SELECT * FROM transaction WHERE denom = $token AND (origin = $chain OR destination = $chain) ORDER BY datetime DESC LIMIT $limit START $start;
        ".to_owned(),
        json!({
            "limit": default_limit,
            "start": default_start,
            "token": token
        })
    ).await;

    println!("Queried");
    let mut output: Vec<Transaction> = vec![];

    println!("Made it");
    match response {
        Ok(resp) => {
            for tx in resp {
                println!("{}", tx.amount);
                output.push(tx);
            }
        },
        Err(rpcError) => {
            // Handle RPC error
            println!("{} RPC Error", rpcError)
        },
    };    

    // TODO: Return output
    println!("{}", output.pop().unwrap().amount);
    println!("{}", output.pop().unwrap().datetime)  
}

async fn get_num_unique_wallets() {
    let mut client = get_client().await;

    let response = client
    .find_one::<NumUniqueWallets>(
    "
    SELECT count() AS unique_wallets FROM wallets GROUP BY ALL;
    "
    .to_owned(),
    json!({})
    ).await;

    match response {
        Ok(resp) => {
            match resp {
                Some(wallets_resp) => {
                    let wallets = wallets_resp.unique_wallets;
                    println!("{}", wallets)
                },
                None => todo!(),
            }
        },
        Err(rpcError) => {
            println!("{} RPC Error", rpcError)
        },
    };    
}

async fn test_local_surrealdb() {
    let mut client = SurrealClient::new("ws://localhost:8000/rpc")
        .await
        .expect("RPC handshake error");

    client.signin("root", "root").await.expect("Signin error");
    client.use_namespace("kwahl", "sampleDBname").await.expect("Namespace error");

    // TODO: Parse entered date-time into just the date for volume indexing
    // let date = "2022-11-07".to_string();
    let response = client
        .send_query(
        "
        BEGIN TRANSACTION;
        CREATE transaction SET date = $date, origin = $origin, destination = $destination, amount = $amount, denom = $denom, transaction_num = $transaction_num, price = $price;
        UPDATE (SELECT origin FROM $origin) SET num_transactions +=1, volume +=$amount;
        UPDATE $origin SET transactions +=1;
        UPDATE (SELECT destination FROM $destination) SET num_transactions +=1, volume +=$amount;
        UPDATE $destination SET transactions +=1;
        UPDATE $origin_vol SET volume[0].incoming += $amount;
        UPDATE $destination_vol SET volume[0].outgoing += $amount;
        COMMIT TRANSACTION;
        "
        .to_owned(),
        json!({
            "date": "2022_11_10",
            "origin": "chain:axelar",
            "origin_vol": "axelar:scrt",
            "destination": "chain:osmosis",
            "destination_vol": "osmosis:scrt",
            "amount": 1600000,
            "denom": "SCRT",
            "transaction_num": 1,
            "price": 5
        })
        ).await;

        // Date must be a string
        // THIS ^^^^ WORKED, good job me haha. Having 'record' types (aka the origin:cosmos_hub notation) is allowed, colons can exist inside string values within the json! macro
    // let response = client
    //     .send_query(
    // "
    //         CREATE User SET username = $sbeve_origin;
    //     ".to_owned(), json!({
    //         "sbeve": "Osmosis"
    //     })
    //     ).await;
    println!("Waiting");
    match response {
        Ok(surreal_resp) => {
            println!("Waiting for surrealResp");
            match surreal_resp.await {
                Ok(data) => {
                    println!("Got surrealResp");
                    if let Some(surreal_query_result) = data.get_nth_query_result(0){
                        let result = surreal_query_result.results();
                        match serde_json::from_value::<Transaction>(result[0].clone()){
                            Ok(transaction) => {
                                println!("{}", transaction.amount)
                            },
                            Err(err) => {
                                println!("Didn't return Transaction formatted response :(");
                                println!("{}", err);
                            }
                        }
                    }
                },
                Err(recv_error) => {
                    println!("Recv Error")
                },
            }
        },
        Err(rpcError) => {
            println!("RPC Error.");
        },
    };

}

// json!({
//     "date": req.date.clone(),
//     "origin": req.origin.clone(),
//     "destination": req.destination.clone(),
//     "amount": req.amount.clone(),
//     "denom": req.denom.clone()
// })

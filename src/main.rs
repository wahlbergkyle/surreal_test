use surreal_simple_client::SurrealClient;
use serde_json::{json, Value};
use serde::Deserialize;

#[tokio::main]
async fn main() {
    test_local_surrealdb().await;
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
            CREATE transaction SET date = $date, origin = $origin, destination = $destination, amount = $amount, denom = $denom;
            UPDATE chain:$origin SET transactions +=1;
            UPDATE origin:$origin SET transactions +=1, amount +=$amount;
            UPDATE chain:$destination SET transactions +=1;
            UPDATE destination:$destination SET transactions +=1, amount +=$amount;
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
                "date": "2022_11_10".to_string(),
                "origin": "kujira".to_string(),
                "destination": "axelar".to_string(),
                "amount": "10000".to_string(),
                "denom": "SCRT".to_string()
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

#[derive(Debug, Default, Deserialize)]
pub struct Transaction {
    pub amount: u128,
    pub date: String,
    pub denom: String,
    pub destination: String, 
    pub id: String,
    pub origin: String,
    pub transaction_num: u128
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


// fn test_json() {
//     let query = "CREATE User SET username = $username";
//     let params = json!({
//         "username": "Kyle"
//     });
//     let json = json!([query, params]);
//     let result = json.to_string();
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
        CREATE transaction SET date = $date, origin = $origin, destination = $destination, amount = $amount, denom = $denom;
        "
        .to_owned(),
        json!({
            "date": "2022_11_10",
            "origin": "origin:cosmos_hub",
            "destination": "destination:osmosis",
            "amount": 5000000,
            "denom": "SCRT",
            "transaction_num": 1
        })
        ).await;

        // Date must be a string

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
                            Err(err) => println!("Didn't return Transaction formatted response :("),
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
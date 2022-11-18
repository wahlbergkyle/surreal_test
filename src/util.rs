use std::{
    io::{self, ErrorKind},
    str::FromStr,
};

use serde::Deserialize;
use surreal_simple_client::SurrealClient;

#[derive(Debug, Default, Deserialize)]
pub struct TxResponse {
    pub tx: Transaction,
}

#[derive(Debug, Default, Deserialize)]
pub struct Transaction {
    pub amount: f64,
    pub datetime: String,
    pub denom: String,
    pub destination: String,
    pub origin: String,
    // pub price: f64,
    pub transaction_uuid: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct DailyVolumeVecResponse {
    pub daily_vol: DailyVolumeByToken,
}

#[derive(Debug, Default, Deserialize)]
pub struct DailyVolumeByToken {
    pub token: String,
    pub vol: Vec<DailyVolume>,
}

#[derive(Debug, Default, Deserialize)]
pub struct DailyVolume {
    pub date: String,
    pub incoming: f64,
    pub outgoing: f64,
}

#[derive(Debug, Default, Deserialize)]
pub struct FormattedDailyVolOutput {
    pub token: String,
    pub date: String,
    pub incoming: f64,
    pub outgoing: f64,
}

#[derive(Debug, Default, Deserialize)]
pub struct AggregateVolVecResponse {
    pub aggregate_vol: AggregateVolume,
}

#[derive(Debug, Default, Deserialize)]
pub struct AggregateVolume {
    pub id: String,
    pub total_incoming: f64,
    pub total_outgoing: f64,
}

pub struct ChainSummary {
    pub chain: String,
    pub outgoing_total: f64,
    pub incoming_total: f64,
}

pub enum SupportedChains {
    Akash,
    Axelar,
    Chihuahua,
    CosmosHub,
    CrescentNetwork,
    Evmos,
    GravityBridge,
    Injective,
    Juno,
    Kujira,
    Osmosis,
    Secret,
    Sentinel,
    Stargaze,
    Stride,
}

impl FromStr for SupportedChains {
    type Err = ();

    fn from_str(c: &str) -> Result<Self, Self::Err> {
        match c.to_lowercase().as_str() {
            "akash" => Ok(SupportedChains::Akash),
            "axelar" => Ok(SupportedChains::Axelar),
            "chihuahua" => Ok(SupportedChains::Chihuahua),
            "cosmoshub" => Ok(SupportedChains::CosmosHub),
            "cosmos_hub" => Ok(SupportedChains::CosmosHub),
            "crescentnetwork" => Ok(SupportedChains::CrescentNetwork),
            "crescent_network" => Ok(SupportedChains::CrescentNetwork),
            "evmos" => Ok(SupportedChains::Evmos),
            "gravitybridge" => Ok(SupportedChains::GravityBridge),
            "gravity_bridge" => Ok(SupportedChains::GravityBridge),
            "injective" => Ok(SupportedChains::Injective),
            "juno" => Ok(SupportedChains::Juno),
            "kujira" => Ok(SupportedChains::Kujira),
            "osmosis" => Ok(SupportedChains::Osmosis),
            "secret" => Ok(SupportedChains::Secret),
            "sentinel" => Ok(SupportedChains::Sentinel),
            "stargaze" => Ok(SupportedChains::Stargaze),
            "stride" => Ok(SupportedChains::Stride),
            _ => Err(()), // TODO, fill in with API error call for unrecognized chain name
        }
    }
}

impl SupportedChains {
    pub fn surrealql_format(&self) -> String {
        match self {
            SupportedChains::Akash => "akash".to_owned(),
            SupportedChains::Axelar => "axelar".to_owned(),
            SupportedChains::Chihuahua => "chihuahua".to_owned(),
            SupportedChains::CosmosHub => "cosmos_hub".to_owned(),
            SupportedChains::CrescentNetwork => "crescent_network".to_owned(),
            SupportedChains::Evmos => "evmos".to_owned(),
            SupportedChains::GravityBridge => "gravity_bridge".to_owned(),
            SupportedChains::Injective => "injective".to_owned(),
            SupportedChains::Juno => "juno".to_owned(),
            SupportedChains::Kujira => "kujira".to_owned(),
            SupportedChains::Osmosis => "osmosis".to_owned(),
            SupportedChains::Secret => "secret".to_owned(),
            SupportedChains::Sentinel => "sentinel".to_owned(),
            SupportedChains::Stargaze => "stargaze".to_owned(),
            SupportedChains::Stride => "stride".to_owned(),
        }
    }
}

pub enum SupportedTokens {
    OSMO,
    AXL,
    ATOM,
    EVMOS,
    KUJI,
    JUNO,
    INJ,
    CRE,
    SCRT,
    STARS,
    ROWAN,
    ATK,
    HUAHUA,
    GRAV,
    DVPN,
}

impl FromStr for SupportedTokens {
    type Err = ();

    fn from_str(t: &str) -> Result<Self, Self::Err> {
        match t.to_lowercase().as_str() {
            "atk" => Ok(SupportedTokens::ATK),
            "atom" => Ok(SupportedTokens::ATOM),
            "axl" => Ok(SupportedTokens::AXL),
            "cre" => Ok(SupportedTokens::CRE),
            "dvpn" => Ok(SupportedTokens::DVPN),
            "evmos" => Ok(SupportedTokens::EVMOS),
            "grav" => Ok(SupportedTokens::GRAV),
            "huahua" => Ok(SupportedTokens::HUAHUA),
            "inj" => Ok(SupportedTokens::INJ),
            "juno" => Ok(SupportedTokens::JUNO),
            "kuji" => Ok(SupportedTokens::KUJI),
            "osmo" => Ok(SupportedTokens::OSMO),
            "rowan" => Ok(SupportedTokens::ROWAN),
            "scrt" => Ok(SupportedTokens::SCRT),
            "stars" => Ok(SupportedTokens::STARS),
            _ => Err(()),
        }
    }
}

impl SupportedTokens {
    fn surrealql_format(&self) -> &str {
        match self {
            SupportedTokens::OSMO => "osmo",
            SupportedTokens::AXL => "axl",
            SupportedTokens::ATOM => "atom",
            SupportedTokens::EVMOS => "evmos",
            SupportedTokens::KUJI => "kuji",
            SupportedTokens::JUNO => "juno",
            SupportedTokens::INJ => "inj",
            SupportedTokens::CRE => "cre",
            SupportedTokens::SCRT => "scrt",
            SupportedTokens::STARS => "stars",
            SupportedTokens::ROWAN => "rowan",
            SupportedTokens::ATK => "atk",
            SupportedTokens::HUAHUA => "huahua",
            SupportedTokens::GRAV => "grav",
            SupportedTokens::DVPN => "dvpn",
        }
    }
}

pub fn valid_chain(s: &str) -> Result<SupportedChains, io::Error> {
    match SupportedChains::from_str(s) {
        Ok(chain) => Ok(chain),
        Err(e) => Err(io::Error::new(ErrorKind::Other, "Bad chain!")),
    }
}

pub fn valid_token(s: &str) -> Result<SupportedTokens, io::Error> {
    match SupportedTokens::from_str(s) {
        Ok(token) => Ok(token),
        Err(e) => Err(io::Error::new(ErrorKind::Other, "Bad chain!")),
    }
}

pub fn sql_chain_token_vol_format(chain: &SupportedChains, token: &SupportedTokens) -> String {
    return format!(
        "{}:{}",
        SupportedChains::surrealql_format(chain),
        SupportedTokens::surrealql_format(token)
    );
}

pub fn sql_chain_format(chain: &SupportedChains) -> String {
    return format!("chain:{}", SupportedChains::surrealql_format(chain));
}

pub async fn get_client() -> SurrealClient {
    let mut client = SurrealClient::new("ws://localhost:8000/rpc")
        .await
        .expect("RPC handshake error");

    client.signin("root", "root").await.expect("Signin error");
    client
        .use_namespace("kwahl", "sampleDBname")
        .await
        .expect("Namespace error");
    client
}

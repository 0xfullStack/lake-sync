#[macro_use]
extern crate diesel;

mod db;
mod dex;
mod abi;

use std::env;
use std::str::FromStr;
use std::sync::Arc;
use env_logger::Env;
use dotenv::dotenv;
use dex::assembler::Assembler;
use dex::subscriber::Subscriber;
use db::postgres::*;
use ethers::prelude::U64;
use ethers::types::{ H256, H160 };
use crate::dex::aggregator::Aggregator;
use crate::dex::models::NewProtocol;

#[tokio::main]
async fn main() -> std::io::Result<()> {

    // Environment
    dotenv().ok();
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // DB pool
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = init_pool(&database_url).expect("Failed to create pool");

    // Node
    let http = (&env::var("CRYPTOLESS_NODE_HTTP").unwrap()).to_string();
    let ws   = (&env::var("CRYPTOLESS_NODE_WS").unwrap()).to_string();

    // Start Service
    let aggregator = Aggregator::make(
        Node { http, ws },
        Arc::new(pool),
        Protocol::UNISwapV2
    );

    // Infinite Loop
    aggregator.start_syncing().await;

    Ok(())
}


pub struct Node {
    pub http: String,
    pub ws: String
}

#[derive(Debug, Clone)]
pub enum Protocol {
    UNISwapV2,
    SushiSwapV2,
}

#[derive(Debug, Copy, Clone)]
pub enum EventType {
    PairCreated,
    Sync
}

impl Protocol {
    fn factory_address(&self) -> H160 {
        match self {
            Protocol::SushiSwapV2 => {
                H160::from_str("0xC0AEe478e3658e2610c5F7A4A2E1777cE9e4f2Ac").unwrap()
            },
            Protocol::UNISwapV2 => {
                H160::from_str("0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f").unwrap()
            }
        }
    }

    fn protocol_info(&self) -> NewProtocol {
        match self {
            Protocol::SushiSwapV2 => {
                NewProtocol {
                    name: "Sushiswap Protocol".to_string(),
                    official_url: Some("".to_string()),
                    network: "ETHEREUM_MAINNET".to_string(),
                    description: Some("".to_string()),
                    symbol: Some("sushiswap-v2".to_string()),
                    router_address: "".to_string(),
                    factory_address: "".to_string().to_lowercase()
                }
            },
            Protocol::UNISwapV2 => {
                NewProtocol {
                    name: "Uniswap Protocol".to_string(),
                    official_url: Some("https://uniswap.org/".to_string()),
                    network: "ETHEREUM_MAINNET".to_string(),
                    description: Some("Swap, earn, and build on the leading decentralized crypto trading protocol.".to_string()),
                    symbol: Some("uniswap-v2".to_string()),
                    router_address: "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D".to_string(),
                    factory_address: "0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f".to_string().to_lowercase()
                }
            }
        }
    }

    fn coinbase(&self) -> U64 {
        match self {
            Protocol::SushiSwapV2 => {
                U64::from(10000835)
            },
            Protocol::UNISwapV2 => {
                U64::from(10000835)
            }
        }
    }
}

impl EventType {
    fn topic_hash(&self) -> H256 {
        match self {
            EventType::PairCreated => {
                H256::from_str("0x0d3648bd0f6ba80134a33ba9275ac585d9d315f0ad8355cddefde31afa28d0e9").unwrap()
            }
            EventType::Sync => {
                H256::from_str("0x1c411e9a96e071241c2f21f7726b17ae89e3cab4c78be50e062b03a9fffbbad1").unwrap()
            }
        }
    }

    fn blocks_per_loop(&self) -> U64 {
        match self {
            EventType::PairCreated => {
                U64::from(1000_000)
            }
            EventType::Sync => {
                U64::from(5_000)
            }
        }
    }

    fn max_threads_count(&self) -> U64 {
        match self {
            EventType::PairCreated => {
                U64::from(10)
            }
            EventType::Sync => {
                U64::from(25)
            }
        }
    }
}


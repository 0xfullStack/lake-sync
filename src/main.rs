#[macro_use]
extern crate diesel;

mod db;
mod dex;
mod abi;

use std::{env, thread};
use std::rc::Rc;
use std::str::FromStr;
use std::sync::Arc;
use env_logger::Env;
use dotenv::dotenv;
use ethers::abi::Address;
use ethers::prelude::{BlockNumber, U256};
use ethers::prelude::builders::Event;
use dex::assembler::Assembler;
use dex::subscriber::Subscriber;
use db::postgres::*;
use ethers::prelude::U64;
use ethers::types::{ H256, H160 };
use crate::dex::aggregator::Aggregator;

#[tokio::main]
async fn main() -> std::io::Result<()> {

    // Environment
    dotenv().ok();
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // DB pool
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = init_pool(&database_url).expect("Failed to create pool");

    // Node
    let http = (&env::var("INFURA_NODE_HTTP").unwrap()).to_string();
    let ws   = (&env::var("INFURA_NODE_WS").unwrap()).to_string();

    // Start Service
    let aggregator = Aggregator::make(
        Node { http, ws },
        Rc::clone(&Rc::new(pool)),
        Protocol::UNISwapV2
    );

    // Infinite Loop
    aggregator.start_syncing().await;
    Result::Ok(())
}

pub struct Node {
    pub http: String,
    pub ws: String
}

#[derive(Debug)]
pub enum Protocol {
    UNISwapV2,
    UNISwapV3,
    SushiSwap
}

#[derive(Debug)]
pub enum EventType {
    PairCreated,
    Sync
}

impl Protocol {
    fn factory_address(&self) -> H160 {
        match self {
            Protocol::SushiSwap => {
                H160::from_str("0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f").unwrap()
            },
            Protocol::UNISwapV2 => {
                H160::from_str("0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f").unwrap()
            }
            Protocol::UNISwapV3 => {
                H160::from_str("0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f").unwrap()
            }
        }
    }

    fn star_block_number(&self) -> U64 {
        match self {
            Protocol::SushiSwap => {
                U64::from(10000835)
            },
            Protocol::UNISwapV2 => {
                U64::from(10000835)
            }
            Protocol::UNISwapV3 => {
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
                U64::from(100_000)
            }
            EventType::Sync => {
                U64::from(50_000)
            }
        }
    }
}


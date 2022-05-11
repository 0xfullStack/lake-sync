#[macro_use]
extern crate diesel;

mod db;
mod dex;
mod abi;

use std::{env, thread};
use std::sync::Arc;
use env_logger::Env;
use dotenv::dotenv;
use ethers::prelude::U256;
use dex::assembler::Assembler;
use dex::subscriber::Subscriber;
use db::postgres::*;

#[tokio::main]
async fn main() -> std::io::Result<()> {

    dotenv().ok();
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // DB pool
    // let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    // let pool = init_pool(&database_url).expect("Failed to create pool");

    let node_http = &env::var("INFURA_NODE_HTTP").unwrap();
    let assembler = Assembler::make(
        node_http.to_string(),
        String::from("0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f")
    );

    let address = assembler.fetch_pair_address(U256::from(1)).await.unwrap();
    let result = assembler.fetch_pair_info(address, U256::from(1)).await.unwrap();
    println!("{:?}", result);

    let node_wss = &env::var("INFURA_NODE_WS").unwrap();
    let subscriber = Subscriber::make(
        node_wss.to_string(),
        String::from("0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f")
    );

    subscriber.watching_with_guardian().await.map(|_| ())
}

#[derive(Debug)]
pub enum Protocol {
    UNISwapV2,
}

#[derive(Debug)]
pub enum State {
    Monitoring,
    Syncing,
    Stop
}

#[derive(Debug)]
pub enum Event {
    PairCreated,
    Sync
}

pub async fn check_sync_state() {}
pub async fn sync() {}

pub fn subscribe(event: Event) {
    match event {
        Event::PairCreated => {

        }
        Event::Sync => {
            println!("Sync")
        }
    }
}

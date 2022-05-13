#[macro_use]
extern crate diesel;

mod db;
mod dex;
mod abi;

use std::{env, thread};
use std::rc::Rc;
use std::sync::Arc;
use env_logger::Env;
use dotenv::dotenv;
use ethers::prelude::{BlockNumber, U256};
use dex::assembler::Assembler;
use dex::subscriber::Subscriber;
use db::postgres::*;
use ethers::prelude::U64;

#[tokio::main]
async fn main() -> std::io::Result<()> {

    dotenv().ok();
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // DB pool
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = init_pool(&database_url).expect("Failed to create pool");
    let rc_pool = Rc::new(pool);

    let node_http = &env::var("INFURA_NODE_HTTP").unwrap();
    let assembler = Assembler::make(
        node_http.to_string(),
        String::from("0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f"),
        Rc::clone(&rc_pool)
    );

    // assembler.polling().await;

    let node_wss = &env::var("INFURA_NODE_WS").unwrap();
    let subscriber = Subscriber::make(
        node_wss.to_string(),
        String::from("0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f")
    );

    let from = BlockNumber::Number(U64::from(14756225));
    let to = BlockNumber::Number(U64::from(14796225));
    // subscriber.sync_from_block_range(from, to).await;
    // subscriber.watching_with_guardian().await.map(|_| ())
    subscriber.start_syncing().await;

    Result::Ok(())
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

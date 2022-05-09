#[macro_use]
extern crate diesel;

mod db;
mod dex;
mod abi;

use std::thread;
use std::thread::Builder;

use std::env;
use env_logger::Env;
use dotenv::dotenv;
use ethers::prelude::U256;
use dex::assembler::Assembler;
use dex::subscriber::Subscriber;
use db::postgres::*;

#[tokio::main]
async fn main() -> std::io::Result<()> {

    // dotenv().ok();
    // env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    //
    // let child = thread::spawn(|| {
    //     println!("Thread");
    //     "Much concurrent. such wow".to_string()
    // });
    //
    // print!("Hello ");
    //
    // let value = child.join().expect("Failed joining child thread");
    //
    // println!("{}", value);
    //
    //
    // let my_thread = Builder::new().name("Worker Thred".to_string()).stack_size(1024 * 4);
    // let handle = my_thread.spawn(|| {
    //     panic!("Oops");
    // });
    //
    // let child_status = handle.unwrap().join();
    // // println!("Child status: {}", child_status);
    //
    // let nums = String::from("damn your fucking ashole");
    // let _ = thread::spawn( move || {
    //     println!("{}", nums);
    // });

    env_logger::init();
    dotenv().ok();

    // DB pool
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = init_pool(&database_url).expect("Failed to create pool");

    let node_http = env::var("INFURA_NODE_HTTP").unwrap().as_str();
    let assembler = Assembler::make(
        String::from("https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27"),
        String::from("0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f")
    );

    let address = assembler.fetch_pair_address(U256::from(1)).await.unwrap();
    let result = assembler.fetch_pair_info(address, U256::from(1)).await.unwrap();
    println!("{:?}", result);

    let node_wss = env::var("INFURA_NODE_WS").unwrap().as_str();
    let subscriber = Subscriber::make(
        String::from("wss://mainnet.infura.io/ws/v3/9aa3d95b3bc440fa88ea12eaa4456161"),
        String::from("0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f")
    );

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

#[macro_use]
extern crate diesel;

mod db;
mod dex;

use std::thread;
use std::thread::Builder;

fn main() {
    let child = thread::spawn(|| {
        println!("Thread");
        "Much concurrent. such wow".to_string()
    });

    print!("Hello ");

    let value = child.join().expect("Failed joining child thread");

    println!("{}", value);


    let my_thread = Builder::new().name("Worker Thred".to_string()).stack_size(1024 * 4);
    let handle = my_thread.spawn(|| {
        panic!("Oops");
    });

    let child_status = handle.unwrap().join();
    // println!("Child status: {}", child_status);

    let nums = String::from("damn your fucking ashole");
    let _ = thread::spawn( move || {
        println!("{}", nums);
    });
}

#[derive(Debug)]
pub enum Chain {
    Ethereum,
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
    LiquidityAdded,
    LiquidityRemoved
}

pub fn check_sync_state() {

}

pub async fn sync() {

}

pub fn subscribe(event: Event) {
    match event {
        Event::PairCreated => {

        }
        Event::LiquidityAdded => {
            println!("LiquidityAdded")
        }
        Event::LiquidityRemoved => {
            println!("LiquidityRemoved")
        }
    }
}

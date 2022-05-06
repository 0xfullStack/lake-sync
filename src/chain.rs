pub mod ethereum;

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

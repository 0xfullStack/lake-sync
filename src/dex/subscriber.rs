use std::env;
use std::ops::{Add, Sub, SubAssign};
use std::rc::Rc;
use std::sync::Arc;
use std::str::FromStr;
use std::time::Duration;
use diesel::query_dsl::InternalJoinDsl;
use dotenv::dotenv;
use ethers::abi::{ParamType, Tokenizable, Tokenize};

use tokio;
use ethers::prelude::*;
use ethers::contract::Contract;
use ethers::prelude::ValueOrArray::Value;
use ethers::types::{U64, Address};
use crate::db::postgres::PgPool;
use crate::dex::models::{NewPair, NewProtocol, *};
use crate::EventType;

#[derive(Clone)]
pub struct Subscriber {
    pub node: String,
    pub factory: Address,
    pool: Rc<PgPool>
}

enum SubscriptionError {
    ConnectionFailed,
    InvalidEncoding,
    UnKnown
}

impl Subscriber {
    pub fn make(node: String, factory: Address, pool: Rc<PgPool>) -> Subscriber {
        Subscriber { node, factory, pool }
    }

    pub async fn listening(&self) {

    }

    async fn patch_pair_sync_events() {

    }





    // pub async fn watching_with_guardian(&self) -> std::io::Result<bool> {
    //
    //     let thread_self = (self.clone(), self.clone());
    //     let pair_created_thread = tokio::spawn(async move {
    //         loop {
    //             thread_self.0.pair_created_event().await;
    //         }
    //     });
    //
    //     let pair_sync_thread = tokio::spawn(async move {
    //         // loop {
    //         //     thread_self.1.pair_sync_event().await;
    //         // }
    //     });
    //
    //     tokio::join!(pair_created_thread, pair_sync_thread);
    //     Result::Ok(true)
    // }

    pub async fn pair_created_event(&self) -> std::io::Result<bool> {
        println!("pari created subscribe begin");

        let ws = Ws::connect(self.node.clone()).await.unwrap();
        let provider = Provider::new(ws).interval(Duration::from_millis(500));
        let block_number = BlockNumber::Number(U64::from(10850000));
        let filter = Filter::default()
            .from_block(block_number)
            .to_block(BlockNumber::Latest)
            .address(ValueOrArray::Value(self.factory))
            .topic0(Value(EventType::PairCreated.topic_hash()));

        let mut stream = provider.subscribe_logs(&filter).await.unwrap();

        while let next = stream.next().await {
            match next {
                Some(log) => {
                    dbg!(log);
                },
                None => {
                    println!("pari created error occur");
                    stream.unsubscribe().await;
                    break;
                }
            }
        }
        Result::Ok(false)
    }

    async fn pair_sync_event(&self) -> std::io::Result<bool> {
        let ws = Ws::connect(self.node.clone()).await.unwrap();
        let provider = Provider::new(ws).interval(Duration::from_millis(500));
        let block_number = BlockNumber::Number(U64::from(10000835));
        let filter = Filter::default()
            .from_block(block_number)
            .topic0(Value(EventType::Sync.topic_hash()));

        let mut stream = provider.subscribe_logs(&filter).await.unwrap();
        while let next = stream.next().await {
            match next {
                Some(log) => {
                    dbg!(log);
                },
                None => {
                    println!("sync error occur");
                    stream.unsubscribe().await;
                    break;
                }
            }
        }
        Result::Ok(false)
    }

    pub fn response_to_reserves_changing(address: Address) {

    }

    fn store_into_db(&self) {
        unimplemented!()
    }
    fn filter_target_pairs_change() {
        unimplemented!()
    }
}
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
use crate::{EventType, Protocol};
use crate::dex::models;

#[derive(Clone)]
pub struct Subscriber {
    pub node: String,
    pub protocol: Protocol,
    pool: Arc<PgPool>
}

enum SubscriptionError {
    ConnectionFailed,
    InvalidEncoding,
    UnKnown
}

impl Subscriber {
    pub fn make(node: String, protocol: Protocol, pool: Arc<PgPool>) -> Subscriber {
        Subscriber {
            node, protocol,
            pool
        }
    }

    pub async fn start_watching(&self) -> std::io::Result<bool> {

        let thread_self = (self.clone(), self.clone());
        let pair_created_thread = tokio::spawn(async move {
            loop {
                thread_self.0.subscribe_pair_created_logs().await;
            }
        });

        let pair_sync_thread = tokio::spawn(async move {
            loop {
                thread_self.1.subscribe_reserve_change_logs().await;
            }
        });

        tokio::join!(pair_created_thread, pair_sync_thread);
        Ok(true)
    }

    pub async fn subscribe_pair_created_logs(&self) -> std::io::Result<bool> {
        println!(" - 1 - Subscriber: start subscribe pair created logs");
        let ws = Ws::connect(self.node.clone()).await.unwrap();
        let provider = Provider::new(ws).interval(Duration::from_millis(500));
        let filter = Filter::default()
            .address(ValueOrArray::Value(self.protocol.factory_address()))
            .topic0(Value(EventType::PairCreated.topic_hash()));

        let mut stream_result = provider.subscribe_logs(&filter).await;
        match stream_result {
            Ok(mut stream) => {
                println!(" - 1 - Subscriber: subscribe reserve change success");
                while let next = stream.next().await {
                    match next {
                        Some(log) => {
                            println!(" - 1- Subscriber: receive new pair created logs: {:?}", log);
                            self.syncing_reserve_into_db(log);
                        },
                        None => {
                            println!(" - 1 - Subscriber: start subscribe pair created logs");
                            stream.unsubscribe().await;
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                println!(" - 1 - Subscriber: subscribe reserve change failure: {:?}", e);
            }
        }
        Ok(false)
    }

    async fn subscribe_reserve_change_logs(&self) -> std::io::Result<bool> {
        println!(" - 2 - Subscriber: start subscribe reserve change logs");

        let ws = Ws::connect(self.node.clone()).await.unwrap();
        let provider = Provider::new(ws).interval(Duration::from_millis(500));
        let block_number = BlockNumber::Number(U64::from(10000835));
        let filter = Filter::default()
            .from_block(block_number)
            .topic0(Value(EventType::Sync.topic_hash()));

        let mut stream_result = provider.subscribe_logs(&filter).await;
        match stream_result {
            Ok(mut stream) => {
                println!(" - 2 - Subscriber: subscribe reserve change success");
                while let next = stream.next().await {
                    match next {
                        Some(log) => {
                            println!(" - 2 - Subscriber: receive new reserve logs: {:?}", log);
                            self.syncing_reserve_into_db(log);
                        },
                        None => {
                            stream.unsubscribe().await;
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                println!(" - 2 - Subscriber: subscribe reserve change failure: {:?}", e);
            }
        }
        Ok(false)
    }

    async fn syncing_pair_into_db(&self, log: Log) {
        let conn = &self.pool.get().unwrap();

        let data = &log.data.to_vec();
        let factory_address = self.protocol.factory_address().into_token().to_string();
        let pair_address = ethers::abi::decode(&vec![ParamType::Address, ParamType::Uint(256)], data).unwrap()[0].to_string();
        let token0 = ethers::abi::decode(&vec![ParamType::Address], log.topics[1].as_bytes()).unwrap()[0].to_string();
        let token1 = ethers::abi::decode(&vec![ParamType::Address], log.topics[2].as_bytes()).unwrap()[0].to_string();
        let block_number = log.block_number.unwrap().as_u64() as i64;
        let block_hash = log.block_hash.unwrap_or(H256::zero());
        let transaction_hash = log.transaction_hash.unwrap_or(H256::zero());

        let new_pair = NewPair {
            pair_address,
            factory_address,
            token0,
            token1,
            block_number,
            block_hash: serde_json::to_string(&block_hash).unwrap(),
            transaction_hash: serde_json::to_string_pretty(&transaction_hash).unwrap(),
            reserve0: "".to_string(),
            reserve1: "".to_string(),
        };

        let temp_new_pair = new_pair.clone();
        match batch_insert_pairs(vec![new_pair], conn) {
            Ok(_) => {
                println!(" - 1 - Subscriber: insert new pair log success: {:?}", temp_new_pair);
            }
            Err(e) => {
                println!(" - 1 - Subscriber: insert new pair log failure: {:?}", e);
            }
        }
    }

    fn syncing_reserve_into_db(&self, log: Log) {
        let conn = &self.pool.get().unwrap();
        let data = &log.data.to_vec();
        let parameters = ethers::abi::decode(&vec![ParamType::Uint(112), ParamType::Uint(112)], data).unwrap();
        let pair_address = log.address.into_token().to_string();
        let block_number = log.block_number.unwrap().as_u64() as i64;
        let block_hash = log.block_hash.unwrap_or(H256::zero());
        let transaction_hash = log.transaction_hash.unwrap_or(H256::zero());

        let new_reserve_log = NewReserveLog {
            pair_address,
            block_number,
            reserve0: parameters[0].clone().into_uint().unwrap().to_string(),
            reserve1: parameters[1].clone().into_uint().unwrap().to_string(),
            block_hash: serde_json::to_string(&block_hash).unwrap(),
            transaction_hash: serde_json::to_string_pretty(&transaction_hash).unwrap(),
        };

        let temp_new_reserve_log = new_reserve_log.clone();
        match batch_insert_reserve_logs(vec![new_reserve_log], conn) {
            Ok(_) => {
                println!(" - 2 - Subscriber: insert new reserve log success: {:?}", temp_new_reserve_log);
            }
            Err(e) => {
                println!(" - 2 - Subscriber: insert new reserve log failure: {:?}", e);
            }
        }
    }
}
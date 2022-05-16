use std::env;
use std::fmt::Display;
use std::ops::{Add, AddAssign, Div, Sub, SubAssign};
use std::rc::Rc;
use std::time::Duration;
use std::sync::Arc;
use std::str::FromStr;

use tokio;
use dotenv::dotenv;

use ethers::prelude::*;
use ethers::abi::Tokenizable;
use ethers::prelude::ValueOrArray::Value;
use ethers::contract::Contract;
use ethers::abi::{ParamType, Tokenize};
use ethers::abi::Error::Hex;
use ethers::providers::{JsonRpcClient, Http};
use ethers::types::{U64, Address};
use ethers::providers::HttpClientError;
use crate::db::postgres::PgPool;
use crate::dex::models;
use crate::dex::models::{NewPair, NewProtocol, NewReserve};
use crate::{EventType, NewReserveLog, Protocol};
use core::str;
use serde::Serialize;

#[derive(Debug)]
enum AssemblerError {
    ReachMaxLimit
}

pub struct Assembler {
    pub node: String,
    pub protocol: Protocol,
    client: Arc<Provider::<Http>>,
    pool: Rc<PgPool>
}

struct BlockRange {
    pub from: U64,
    pub to: U64,
    pub blocks_per_loop: U64
}

impl Assembler {
    pub fn make(node: String, protocol: Protocol, pool: Rc<PgPool>) -> Assembler {
        Assembler {
            protocol, pool,
            node: node.clone(),
            client: Arc::new(Provider::<Http>::try_from(node.clone()).unwrap()),
        }
    }

    pub async fn polling_pairs(&self) -> std::io::Result<bool> {
        let conn = &self.pool.get().unwrap();
        let blocks_per_loop = EventType::PairCreated.blocks_per_loop();
        let start_block_number: U64 = self.protocol.star_block_number();
        let latest_block_number: U64 = self.client.get_block_number().await.unwrap();

        let initialize_blocks_remain = latest_block_number.sub(start_block_number).add(1);
        let mut blocks_remain = initialize_blocks_remain;
        let mut meet_last_loop = false;

        while blocks_remain > U64::zero() {

            println!("start");

            let mut start_block_per_loop;
            let mut end_block_per_loop;

            if initialize_blocks_remain <= blocks_per_loop {
                start_block_per_loop = start_block_number;
                end_block_per_loop = latest_block_number;
                meet_last_loop = true;
            } else {
                // Calculate start end block
                start_block_per_loop = start_block_number.add(initialize_blocks_remain.sub(blocks_remain));
                if meet_last_loop {
                    end_block_per_loop = start_block_per_loop.sub(1).add(blocks_remain);
                } else {
                    end_block_per_loop = start_block_per_loop.sub(1).add(blocks_per_loop);
                }
            }

            println!("{}", start_block_per_loop.to_string());
            println!("{}", end_block_per_loop.to_string());

            let pairs = self.syncing_pairs(
                BlockNumber::Number(start_block_per_loop),
                BlockNumber::Number(end_block_per_loop)
            ).await;

            match models::batch_insert_pairs(pairs, conn) {
                Ok(_) => {
                    // println!("Success: {:?}", pair);
                }
                Err(e) => {
                    println!("{}", e);
                    break;
                }
            }

            // last loop flag
            if meet_last_loop {
                break;
            }
            if blocks_remain.sub(blocks_per_loop) < blocks_per_loop {
                meet_last_loop = true;
            }
            blocks_remain.sub_assign(blocks_per_loop);
        }

        println!("Pair sync finished");
        Ok(true)
    }

    async fn syncing_pairs(&self, from: BlockNumber, to: BlockNumber) -> Vec<NewPair> {
        let filter = Filter::default()
            .address(ValueOrArray::Value(self.protocol.factory_address()))
            .topic0(Value(EventType::PairCreated.topic_hash()))
            .from_block(from)
            .to_block(to);

        // thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: JsonRpcClientError(JsonRpcError(JsonRpcError { code: -32005, message: "query timeout exceeded", data: None }))'
        let logs = self.client.get_logs(&filter).await.unwrap();
        let mut pairs: Vec<NewPair> = Vec::with_capacity(logs.len());
        for log in logs {
            let data = &log.data.to_vec();
            let factory_address = self.protocol.factory_address().into_token().to_string();
            let pair_address = ethers::abi::decode(&vec![ParamType::Address, ParamType::Uint(256)], data).unwrap()[0].to_string();
            let token0 = ethers::abi::decode(&vec![ParamType::Address], log.topics[1].as_bytes()).unwrap()[0].to_string();
            let token1 = ethers::abi::decode(&vec![ParamType::Address], log.topics[2].as_bytes()).unwrap()[0].to_string();
            let block_number = log.block_number.unwrap().as_u64() as i64;
            let block_hash = log.block_hash.unwrap_or(H256::zero());
            let transaction_hash = log.transaction_hash.unwrap_or(H256::zero());

            let pair = NewPair {
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
            pairs.push(pair);
        }
        pairs
    }

    pub async fn polling_reserve_logs(&self) -> std::io::Result<bool> {
        let mut blocks_per_loop = EventType::Sync.blocks_per_loop();
        let start_block_number: U64 = self.protocol.star_block_number();
        let latest_block_number: U64 = self.client.get_block_number().await.unwrap();

        let initialize_blocks_remain = latest_block_number.sub(start_block_number).add(1);
        let mut blocks_remain = initialize_blocks_remain;
        let mut meet_last_loop = false;

        while blocks_remain > U64::zero() {

            println!(" - 1 - Start syncing reserves");

            let mut start_block_per_loop;
            let mut end_block_per_loop;

            if initialize_blocks_remain <= blocks_per_loop {
                start_block_per_loop = start_block_number;
                end_block_per_loop = latest_block_number;
            } else {
                start_block_per_loop = start_block_number.add(initialize_blocks_remain.sub(blocks_remain));
                if meet_last_loop {
                    end_block_per_loop = start_block_per_loop.sub(1).add(blocks_remain);
                } else {
                    end_block_per_loop = start_block_per_loop.sub(1).add(blocks_per_loop);
                }
            }

            let result = self.fetch_reserve_logs(
                start_block_per_loop.as_u64() as i64,
                end_block_per_loop.as_u64() as i64
            ).await;

            match result {
                Ok(logs_) => {
                    println!(" - 2 - Fetching {:?} reserve logs successfully from: {:?} to: {:?}", logs_.len(), start_block_per_loop.to_string(), end_block_per_loop.to_string());
                    self.syncing_into_db(logs_);

                    // last loop flag
                    if meet_last_loop {
                        break;
                    }
                    if blocks_remain.sub(blocks_per_loop) < blocks_per_loop {
                        meet_last_loop = true;
                    }
                    blocks_remain.sub_assign(blocks_per_loop);
                    blocks_per_loop = EventType::Sync.blocks_per_loop();
                }
                Err(e) => {
                    println!(" - 2 - Fetching reserve logs failure {:?} from: {:?} to: {:?}, and cut blocks per loop by half", e, start_block_per_loop.to_string(), end_block_per_loop.to_string());
                    println!();
                    blocks_per_loop = blocks_per_loop / 2;
                    continue;
                }
            }
        }
        println!("Reserve logs sync finished");
        Ok(true)
    }

    async fn fetch_reserve_logs(&self, from: i64, to: i64) -> Result<Vec<Log>, ProviderError> {
        let filter = Filter::default()
            .topic0(Value(EventType::Sync.topic_hash()))
            .from_block(BlockNumber::Number(U64::from(from)))
            .to_block(BlockNumber::Number(U64::from(to)));
        self.client.get_logs(&filter).await
    }

    fn syncing_into_db(&self, logs: Vec<Log>) {
        let conn = &self.pool.get().unwrap();
        let mut reserve_logs: Vec<NewReserveLog> = Vec::with_capacity(logs.len());
        for log in logs {
            let data = &log.data.to_vec();
            let factory_address = self.protocol.factory_address().into_token().to_string();
            let parameters = ethers::abi::decode(&vec![ParamType::Uint(112), ParamType::Uint(112)], data).unwrap();
            let pair_address = log.address.into_token().to_string();
            let block_number = log.block_number.unwrap().as_u64() as i64;
            let block_hash = log.block_hash.unwrap_or(H256::zero());
            let transaction_hash = log.transaction_hash.unwrap_or(H256::zero());

            let log = NewReserveLog {
                pair_address,
                factory_address,
                block_number,
                reserve0: parameters[0].clone().into_uint().unwrap().to_string(),
                reserve1: parameters[1].clone().into_uint().unwrap().to_string(),
                block_hash: serde_json::to_string(&block_hash).unwrap(),
                transaction_hash: serde_json::to_string_pretty(&transaction_hash).unwrap(),
            };
            reserve_logs.push(log);
        }

        let _count = reserve_logs.len();
        // number of parameters must be between
        match models::batch_insert_reserve_logs(reserve_logs, conn) {
            Ok(count) => {
                println!(" - 3 - Want to insert {:?}, actually storing {:?} reserve logs successfully", _count, count);
                println!();
            }
            Err(e) => {
                println!(" - 3 - Storing reserve logs {:?} failure: {:?}", e, _count);
                println!();
            }
        }
    }

    // fn load_eligible_addresses(&self, to: i64) -> Vec<Address> {
    //     let from = self.protocol.star_block_number().as_u64() as i64;
    //     let conn = &self.pool.get().unwrap();
    //     let addresses_from_db = get_addresses(conn, from, to).unwrap();
    //     println!("eligible_addresses: {}", addresses_from_db.len());
    //
    //     addresses_from_db
    //         .iter()
    //         .map(|address| {
    //             Address::from_str(address.as_str()).unwrap()
    //         })
    //         .collect()
    // }
}
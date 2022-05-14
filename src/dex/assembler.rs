use std::env;
use std::fmt::Display;
use std::ops::{Add, AddAssign, Sub, SubAssign};
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
use crate::abi::abis::{IUniSwapV2Factory, IUniswapV2Pair};
use crate::db::postgres::PgPool;
use crate::dex::models;
use crate::dex::models::{NewPair, NewProtocol, NewReserve};
use crate::{EventType, Protocol};
use core::str;
use hex::FromHex;


enum AssemblerError {}

pub struct Assembler {
    pub node: String,
    pub factory: Address,
    client: Arc<Provider::<Http>>,
    pool: Rc<PgPool>
}

impl Assembler {
    pub fn make(node: String, factory: Address, pool: Rc<PgPool>) -> Assembler {
        Assembler {
            factory, pool,
            node: node.clone(),
            client: Arc::new(Provider::<Http>::try_from(node.clone()).unwrap())
        }
    }

    pub async fn polling(&self) -> std::io::Result<bool> {
        let conn = &self.pool.get().unwrap();

        let blocks_per_loop = EventType::PairCreated.blocks_per_loop();
        let start_block_number: U64 = Protocol::UNISwapV2.star_block_number();
        let latest_block_number: U64 = self.client.get_block_number().await.unwrap();

        let initialize_blocks_remain = latest_block_number.sub(start_block_number).add(1);
        let mut blocks_remain = initialize_blocks_remain;
        let mut meet_last_loop = false;
        let mut pair_index_: i64 = 0;

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
                BlockNumber::Number(end_block_per_loop),
                pair_index_
            ).await;

            pair_index_.add_assign(pairs.len() as i64);

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
        Result::Ok(true)
    }

    async fn syncing_pairs(&self, from: BlockNumber, to: BlockNumber, last_pair_index: i64) -> Vec<NewPair> {
        let filter = Filter::default()
            .address(ValueOrArray::Value(self.factory))
            .topic0(Value(EventType::PairCreated.topic_hash()))
            .from_block(from)
            .to_block(to);

        let mut increment_pair_index = last_pair_index;
        let logs = self.client.get_logs(&filter).await.unwrap();
        let mut pairs: Vec<NewPair> = Vec::with_capacity(logs.len());
        for log in logs {
            let data = &log.data.to_vec();
            let factory_address = self.factory.into_token().to_string();
            let pair_address = ethers::abi::decode(&vec![ParamType::Address, ParamType::Uint(256)], data).unwrap()[0].to_string();
            let token0 = ethers::abi::decode(&vec![ParamType::Address], log.topics[1].as_bytes()).unwrap()[0].to_string();
            let token1 = ethers::abi::decode(&vec![ParamType::Address], log.topics[2].as_bytes()).unwrap()[0].to_string();

            increment_pair_index.add_assign(1);
            let pair = NewPair {
                pair_address,
                pair_index: increment_pair_index,
                token0,
                token1,
                reserve0: "".to_string(),
                reserve1: "".to_string(),
                factory: factory_address
            };
            pairs.push(pair);
        }
        pairs
    }

    pub async fn polling_sync(&self) -> std::io::Result<bool> {
        let conn = &self.pool.get().unwrap();

        // let start_block_number: U64 = U64::from(10008355);
        // let latest_block_number: U64 = U64::from(10042267);

        let blocks_per_loop = EventType::Sync.blocks_per_loop();
        let start_block_number: U64 = Protocol::UNISwapV2.star_block_number();
        let latest_block_number: U64 = self.client.get_block_number().await.unwrap();

        let initialize_blocks_remain = latest_block_number.sub(start_block_number).add(1);
        let mut blocks_remain = initialize_blocks_remain;
        let mut meet_last_loop = false;
        let mut pair_index_: i64 = 0;

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

            let reserves = self.syncing_reserves(
                BlockNumber::Number(start_block_per_loop),
                BlockNumber::Number(end_block_per_loop)
            ).await;

            pair_index_.add_assign(reserves.len() as i64);

            match models::batch_update_reserves(reserves, conn) {
                Ok(count) => {
                    println!("Success count: {:?}", count);
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

        println!("Reserves sync finished");
        Result::Ok(true)
    }

    async fn syncing_reserves(&self, from: BlockNumber, to: BlockNumber) -> Vec<(String, NewReserve)> {
        let filter = Filter::default()
            .topic0(Value(EventType::Sync.topic_hash()))
            .from_block(from)
            .to_block(to);

        let logs = self.client.get_logs(&filter).await.unwrap();
        let mut reserves: Vec<(String, NewReserve)> = Vec::with_capacity(logs.len());
        for log in logs {
            let data = &log.data.to_vec();
            let parameters = ethers::abi::decode(&vec![ParamType::Uint(112), ParamType::Uint(112)], data).unwrap();
            let reserve = NewReserve {
                reserve0: parameters[0].clone().into_uint().unwrap().to_string(),
                reserve1: parameters[1].clone().into_uint().unwrap().to_string()
            };
            reserves.push((log.address.into_token().to_string(), reserve));
        }
        reserves
    }
}
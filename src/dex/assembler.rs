use std::env;
use std::fmt::Display;
use std::ops::{Add, Sub, SubAssign};
use std::rc::Rc;
use std::time::Duration;
use std::sync::Arc;
use std::str::FromStr;

use tokio;
use hex::ToHex;
use dotenv::dotenv;

use ethers::prelude::*;
use ethers::abi::Tokenizable;
use ethers::prelude::ValueOrArray::Value;
use ethers::contract::Contract;
use ethers::abi::{ParamType, Tokenize};
use ethers::providers::{JsonRpcClient, Http};
use ethers::types::{U64, Address};
use ethers::providers::HttpClientError;
use crate::abi::abis::{IUniSwapV2Factory, IUniswapV2Pair};
use crate::db::postgres::PgPool;
use crate::dex::models;
use crate::dex::models::{NewPair, NewProtocol};
use crate::EventType;

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

        let blocks_per_loop = U64::from(100000);
        // let start_block_number: U64 = U64::from(10008355);
        // let latest_block_number: U64 = U64::from(10042267);
        let start_block_number: U64 = U64::from(10000835);
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

            let pairs = self.syncing_range_blocks(
                BlockNumber::Number(start_block_per_loop),
                BlockNumber::Number(end_block_per_loop),
                EventType::PairCreated
            ).await;

            match models::add_new_pairs(pairs, conn) {
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

        Result::Ok(true)
    }

    async fn syncing_range_blocks(&self, from: BlockNumber, to: BlockNumber, event: EventType) -> Vec<NewPair> {
        let filter = Filter::default()
            .address(ValueOrArray::Value(self.factory))
            .topic0(Value(event.topic_hash()))
            .from_block(from)
            .to_block(to);

        let logs = self.client.get_logs(&filter).await.unwrap();
        let mut pairs: Vec<NewPair> = Vec::with_capacity(logs.len());
        for log in logs {
            let data = &log.data.to_vec();
            let parameters = ethers::abi::decode(&vec![ParamType::Address, ParamType::Uint(256)], data).unwrap();

            let token0 = Address::from(log.topics[1]).to_string();
            let token1 = Address::from(log.topics[2]).to_string();
            let pair_address = parameters[0].to_string();

            let pair = NewPair {
                pair_address,
                pair_index: 0,
                token0,
                token1,
                reserve0: "".to_string(),
                reserve1: "".to_string(),
                factory: self.factory.to_string()
            };
            pairs.push(pair);
        }
        pairs
    }
}
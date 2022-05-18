use std::ops::{Add, Div, Mul, Sub, SubAssign};
use std::sync::Arc;

use ethers::prelude::*;
use ethers::abi::Tokenizable;
use ethers::prelude::ValueOrArray::Value;
use ethers::abi::ParamType;
use ethers::providers::Http;
use ethers::types::U64;
use crate::db::postgres::PgPool;
use crate::dex::models;
use crate::dex::models::{get_last_pair_block_height, get_last_reserve_log_block_height, NewPair, NewReserveLog};
use crate::{EventType, Protocol};

#[derive(Clone)]
pub struct Assembler {
    pub node: String,
    pub protocol: Protocol,
    client: Arc<Provider::<Http>>,
    pool: Arc<PgPool>
}

const MAX_CONCURRENCY_THREAD: i64 = 10;
impl Assembler {
    pub fn make(node: String, protocol: Protocol, pool: Arc<PgPool>) -> Assembler {
        Assembler {
            protocol, pool,
            node: node.clone(),
            client: Arc::new(Provider::<Http>::try_from(node.clone()).unwrap()),
        }
    }

    pub async fn polling_pairs(&self) -> std::io::Result<bool> {
        let conn = &self.pool.get().unwrap();
        let blocks_per_loop = EventType::PairCreated.blocks_per_loop();
        let latest_block_in_db = U64::from(get_last_pair_block_height(conn).unwrap_or(0));
        let standar_block_number = self.protocol.star_block_number();

        let start_block_number: U64;
        if latest_block_in_db > standar_block_number {
            start_block_number = latest_block_in_db + 1;
        } else {
            start_block_number = standar_block_number;
        }

        let latest_block_number: U64 = self.client.get_block_number().await.unwrap();
        let initialize_blocks_remain = latest_block_number.sub(start_block_number).add(1);
        let mut blocks_remain = initialize_blocks_remain;
        let mut meet_last_loop = false;

        while blocks_remain > U64::zero() {

            println!("start");

            let start_block_per_loop;
            let end_block_per_loop;

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

            let logs = self.fetch_pairs_logs(
                start_block_per_loop.as_u64() as i64,
                end_block_per_loop.as_u64() as i64
            ).await.unwrap();

            if logs.len() > 0 {
                self.syncing_pairs_into_db(logs);
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


    pub async fn polling_reserve_logs_by_multi_thread(&self) {

        let conn = &self.pool.get().unwrap();
        let standar_block_number = self.protocol.star_block_number();
        let mut latest_block_in_db = U64::from(get_last_reserve_log_block_height(conn).unwrap_or(0));

        let start_block_number;
        if latest_block_in_db > standar_block_number {
            start_block_number = latest_block_in_db + 1;
        } else {
            start_block_number = standar_block_number;
        }

        let latest_block_number: U64 = self.client.get_block_number().await.unwrap();
        let initialize_blocks_remain = latest_block_number.sub(start_block_number).add(1);
        let mut blocks_remain = initialize_blocks_remain;
        let mut meet_last_loop = false;


        let mut blocks_per_loop = EventType::Sync.blocks_per_loop();

        while blocks_remain > U64::zero() {

            let start_block_per_loop;
            let end_block_per_loop;

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

            let mut tasks = Vec::new();
            let range_per_loop = blocks_per_loop.div(MAX_CONCURRENCY_THREAD);
            for index in 0..=MAX_CONCURRENCY_THREAD {
                let clone_self = self.clone();
                let task = tokio::spawn(async move {
                    clone_self.polling_reserve_logs(
                        start_block_per_loop.add(range_per_loop.mul(index)),
                        start_block_per_loop.add(range_per_loop).sub(1),
                        range_per_loop
                    ).await;
                });
                tasks.push(task);
            }

            for task in tasks {
                task.await;
            }

            println!("- - Reserve logs sync from {:?} to {:?} finished", start_block_per_loop, end_block_per_loop);
            println!();

            if meet_last_loop {
                break;
            }
            if blocks_remain.sub(blocks_per_loop) < blocks_per_loop {
                meet_last_loop = true;
            }
            blocks_remain.sub_assign(blocks_per_loop);
        }
    }

    pub async fn polling_reserve_logs(&self, from: U64, to: U64, blocks_per_loop: U64) -> std::io::Result<bool> {
        let mut blocks_per_loop= blocks_per_loop;
        let start_block_number = from;
        let latest_block_number = to;

        let initialize_blocks_remain = latest_block_number.sub(start_block_number).add(1);
        let mut blocks_remain = initialize_blocks_remain;
        let mut meet_last_loop = false;

        println!(" - 1 - Start syncing reserves from {:?} to {:?}", from, to);

        while blocks_remain > U64::zero() {

            let start_block_per_loop;
            let end_block_per_loop;

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

                    if logs_.len() > 0 {
                        self.syncing_reserves_into_db(logs_);
                    }

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
                    println!(" - 2 - Fetching reserve logs failure from: {:?} to: {:?}, error: {:?}, cut by half", start_block_per_loop.to_string(), end_block_per_loop.to_string(), e);
                    blocks_per_loop = blocks_per_loop / 2;
                    continue;
                }
            }
        }
        Ok(true)
    }

    fn syncing_pairs_into_db(&self, logs: Vec<Log>) {
        let conn = &self.pool.get().unwrap();
        let mut pairs: Vec<NewPair> = Vec::with_capacity(logs.len());
        for log in logs {
            let data = &log.data.to_vec();
            let factory_address = self.protocol.factory_address().into_token().to_string();
            let pair_address = ethers::abi::decode(&vec![ParamType::Address, ParamType::Uint(256)], data).unwrap()[0].to_string();
            let token0 = ethers::abi::decode(&vec![ParamType::Address], log.topics[1].as_bytes()).unwrap()[0].to_string();
            let token1 = ethers::abi::decode(&vec![ParamType::Address], log.topics[2].as_bytes()).unwrap()[0].to_string();
            let block_number = log.block_number.unwrap().as_u64() as i64;
            let mut block_hash = serde_json::to_string(&log.block_hash.unwrap_or(H256::zero())).unwrap();
            let mut transaction_hash = serde_json::to_string(&log.transaction_hash.unwrap_or(H256::zero())).unwrap();
            block_hash.retain(|c| c != '\"');
            transaction_hash.retain(|c| c != '\"');

            let pair = NewPair {
                pair_address: format!("0x{}", pair_address),
                factory_address: format!("0x{}", factory_address),
                token0: format!("0x{}", token0),
                token1: format!("0x{}", token1),
                block_number,
                block_hash,
                transaction_hash,
                reserve0: "".to_string(),
                reserve1: "".to_string(),
            };
            pairs.push(pair);
        }
        match models::batch_insert_pairs(pairs, conn) {
            Ok(count) => {
                println!("Success: {:?}", count);
            }
            Err(e) => {
                println!("{}", e);
            }
        }
    }

    fn syncing_reserves_into_db(&self, logs: Vec<Log>) {
        let conn = &self.pool.get().unwrap();
        let mut reserve_logs: Vec<NewReserveLog> = Vec::with_capacity(logs.len());
        for log in logs {
            let data = &log.data.to_vec();
            let parameters = ethers::abi::decode(&vec![ParamType::Uint(112), ParamType::Uint(112)], data).unwrap();
            let pair_address = format!("0x{}", log.address.into_token().to_string());
            let block_number = log.block_number.unwrap().as_u64() as i64;
            let mut block_hash = serde_json::to_string(&log.block_hash.unwrap_or(H256::zero())).unwrap();
            let mut transaction_hash = serde_json::to_string(&log.transaction_hash.unwrap_or(H256::zero())).unwrap();
            block_hash.retain(|c| c != '\"');
            transaction_hash.retain(|c| c != '\"');

            let log = NewReserveLog {
                pair_address,
                block_number,
                reserve0: parameters[0].clone().into_uint().unwrap().to_string(),
                reserve1: parameters[1].clone().into_uint().unwrap().to_string(),
                block_hash,
                transaction_hash,
            };
            reserve_logs.push(log);
        }

        let _count = reserve_logs.len();
        match models::batch_insert_reserve_logs(reserve_logs, conn) {
            Ok(count) => {
                println!(" - 3 - Want to insert => actually storing: {:?} -> {:?}, reserve logs successfully", _count, count);
            }
            Err(e) => {
                println!(" - 3 - Storing reserve logs {:?} failure: {:?}", _count, e);
            }
        }
    }

    async fn fetch_pairs_logs(&self, from: i64, to: i64) -> Result<Vec<Log>, ProviderError> {
        let filter = Filter::default()
            .address(ValueOrArray::Value(self.protocol.factory_address()))
            .topic0(Value(EventType::PairCreated.topic_hash()))
            .from_block(from)
            .to_block(to);
        self.client.get_logs(&filter).await
    }

    async fn fetch_reserve_logs(&self, from: i64, to: i64) -> Result<Vec<Log>, ProviderError> {
        let filter = Filter::default()
            .topic0(Value(EventType::Sync.topic_hash()))
            .from_block(BlockNumber::Number(U64::from(from)))
            .to_block(BlockNumber::Number(U64::from(to)));
        self.client.get_logs(&filter).await
    }
}
use std::ops::{Add, Div, Mul, Sub, SubAssign};
use std::sync::Arc;
use diesel::QueryResult;

use ethers::prelude::*;
use ethers::abi::Tokenizable;
use ethers::prelude::ValueOrArray::Value;
use ethers::abi::ParamType;
use ethers::providers::Http;
use ethers::types::U64;
use std::option::Option;
use crate::db::postgres::PgPool;
use crate::dex::models;
use crate::dex::models::{get_last_pair_block_height, get_last_reserve_log_block_height, NewPair, NewReserveLog, UpdateReserve};
use crate::{EventType, Protocol};

#[derive(Clone)]
pub struct Assembler {
    pub node: String,
    pub protocol: Protocol,
    client: Arc<Provider::<Http>>,
    pool: Arc<PgPool>
}

#[derive(Clone, Copy)]
pub struct BlockRange {
    pub from: U64,
    pub to: U64,
    pub size: U64
}

impl Assembler {
    pub fn make(node: String, protocol: Protocol, pool: Arc<PgPool>) -> Assembler {
        Assembler {
            protocol, pool,
            node: node.clone(),
            client: Arc::new(Provider::<Http>::try_from(node.clone()).unwrap()),
        }
    }

    pub async fn polling(&self, event: EventType) {

        let blocks_per_loop = event.blocks_per_loop();
        let mut meet_last_loop = false;
        let mut scanned_block_cursor= None;
        loop {
            let total_range = self.get_total_block_range(event, scanned_block_cursor).await;
            if total_range.size == U64::zero() || meet_last_loop {
                break
            }

            let range_per_loop = Assembler::get_block_range_per_loop(total_range, blocks_per_loop);
            let from = range_per_loop.from;
            let to = range_per_loop.to;
            let mut thread_count = event.max_threads_count().as_u32();
            let mut blocks_per_thread = blocks_per_loop.div(thread_count);
            if range_per_loop.size < blocks_per_thread {
                self.handle_task(from, to, event).await;
                meet_last_loop = true;
            } else {
                let mut tasks = Vec::new();
                for index in 0..thread_count {
                    let clone_self = self.clone();
                    let from = from.add(blocks_per_thread.mul(index));
                    let to= from.add(blocks_per_thread).sub(1);
                    let task = tokio::spawn(async move {
                        clone_self.handle_task(from, to, event).await;
                    });
                    tasks.push(task);
                }
                for task in tasks {
                    task.await;
                }
            }
            scanned_block_cursor = Some(to);
            println!("- - {:?} Logs sync from {:?} to {:?} finished", event, from, to);
            println!();
        }
    }

    async fn handle_task(&self, from: U64, to: U64, event: EventType) {
        let result;
        match event {
            EventType::PairCreated => {
                println!(" - 1 - Start {:?} logs syncing from: {:?} to: {:?}", event, from, to);
                result = self.fetch_pairs_logs(from, to).await;
            }
            EventType::Sync => {
                println!(" - 1 - Start {:?} logs syncing from: {:?} to: {:?}", event, from, to);
                result = self.fetch_reserve_logs(from, to).await;
            }
        }

        match result {
            Ok(logs) => {
                println!(" - 2 - Fetching {:?} {:?} logs successfully from: {:?} to: {:?}", event, logs.len(), from, to);
                if logs.len() > 0 {
                    self.syncing_logs_into_db(logs, event);
                }
            }
            Err(e) => {
                println!(" - 2 - Fetching {:?} logs failure from: {:?} to: {:?}, error: {:?}, cut by half", event, from, to, e);
            }
        }
    }

    fn syncing_logs_into_db(&self, logs: Vec<Log>, event: EventType) {

        let conn = &self.pool.get().unwrap();
        let len = logs.len();
        let result: QueryResult<usize>;
        match event {
            EventType::PairCreated => {
                let mut records = Vec::with_capacity(len);
                for log in logs {
                    records.push(NewPair::construct_by(&log, self.protocol.factory_address()));
                }
                result = models::batch_insert_pairs(records, conn);
            }
            EventType::Sync => {
                let mut records = Vec::with_capacity(len);
                for log in logs {
                    records.push(NewReserveLog::construct_by(&log));
                }
                result = models::batch_insert_reserve_logs(records, conn);
            }
        }
        match result {
            Ok(count) => {
                println!(" - 3 - Insert {:?} {:?} records successfully", event, count);
            }
            Err(e) => {
                println!(" - 3 - Insert {:?} {:?} records failure: {:?}", len, event, e);
            }
        }
    }

    fn syncing_reserves_into_db(&self) {
        let conn = &self.pool.get().unwrap();

        // get_latest_pair_reserves()



        // batch_update_reserves(conn);
    }

    async fn fetch_pairs_logs(&self, from: U64, to: U64) -> Result<Vec<Log>, ProviderError> {
        let filter = Filter::default()
            .address(ValueOrArray::Value(self.protocol.factory_address()))
            .topic0(Value(EventType::PairCreated.topic_hash()))
            .from_block(BlockNumber::Number(from))
            .to_block(BlockNumber::Number(to));
        self.client.get_logs(&filter).await
    }

    async fn fetch_reserve_logs(&self, from: U64, to: U64) -> Result<Vec<Log>, ProviderError> {
        let filter = Filter::default()
            .topic0(Value(EventType::Sync.topic_hash()))
            .from_block(BlockNumber::Number(from))
            .to_block(BlockNumber::Number(to));
        self.client.get_logs(&filter).await
    }

    async fn get_total_block_range(&self, event: EventType, scanned_block_cursor: Option<U64>) -> BlockRange {
        let coinbase = self.protocol.coinbase();
        let block_height_in_db = self.get_event_block_height_in_db(event);
        let mut from = coinbase;
        if block_height_in_db > coinbase {
            from = block_height_in_db + 1;
        }
        if let Some(cursor) = scanned_block_cursor {
            if cursor > block_height_in_db {
                from = cursor;
            }
        }

        let mut to: U64 = self.client.get_block_number().await.unwrap();
        let size;
        if from >= to {
            to = from;
            size = U64::zero();
        } else {
            size = to.sub(from).add(1);
        }
        BlockRange { from, to, size }
    }

    fn get_block_range_per_loop(total_range: BlockRange, blocks_per_loop: U64) -> BlockRange {
        let from;
        let to;

        if total_range.size <= blocks_per_loop {
            from = total_range.from;
            to = total_range.to;
        } else {
            from = total_range.from;
            to = from.add(blocks_per_loop).sub(1);
        }
        let size = to.sub(from).add(1);
        BlockRange { from, to, size }
    }

    fn get_event_block_height_in_db(&self, event: EventType) -> U64 {
        let conn = &self.pool.get().unwrap();
        match event {
            EventType::PairCreated => {
                U64::from(get_last_pair_block_height(conn).unwrap_or(0))
            }
            EventType::Sync => {
                U64::from(get_last_reserve_log_block_height(conn).unwrap_or(0))
            }
        }
    }
}

impl NewPair {
    pub(crate) fn construct_by(log: &Log, factory_address: H160) -> Self {
        let data = &log.data.to_vec();
        let factory_address = factory_address.into_token().to_string();
        let pair_address = ethers::abi::decode(&vec![ParamType::Address, ParamType::Uint(256)], data).unwrap()[0].to_string();
        let token0 = ethers::abi::decode(&vec![ParamType::Address], log.topics[1].as_bytes()).unwrap()[0].to_string();
        let token1 = ethers::abi::decode(&vec![ParamType::Address], log.topics[2].as_bytes()).unwrap()[0].to_string();
        let block_number = log.block_number.unwrap().as_u64() as i64;
        let mut block_hash = serde_json::to_string(&log.block_hash.unwrap_or(H256::zero())).unwrap();
        let mut transaction_hash = serde_json::to_string(&log.transaction_hash.unwrap_or(H256::zero())).unwrap();
        block_hash.retain(|c| c != '\"');
        transaction_hash.retain(|c| c != '\"');

        NewPair {
            pair_address: format!("0x{}", pair_address),
            factory_address: format!("0x{}", factory_address),
            token0: format!("0x{}", token0),
            token1: format!("0x{}", token1),
            block_number,
            block_hash,
            transaction_hash,
            reserve0: "".to_string(),
            reserve1: "".to_string(),
        }
    }
}

impl NewReserveLog {

    pub(crate) fn construct_by(log: &Log) -> Self {
        let reserve_info = NewReserveLog::extract_reserve_info(log);
        let pair_address = reserve_info.0;
        let block_number = log.block_number.unwrap().as_u64() as i64;
        let log_index = log.log_index.unwrap().as_u64() as i64;
        let mut block_hash = serde_json::to_string(&log.block_hash.unwrap_or(H256::zero())).unwrap();
        let mut transaction_hash = serde_json::to_string(&log.transaction_hash.unwrap_or(H256::zero())).unwrap();
        block_hash.retain(|c| c != '\"');
        transaction_hash.retain(|c| c != '\"');

        NewReserveLog {
            pair_address,
            block_number,
            reserve0: reserve_info.1.reserve0,
            reserve1: reserve_info.1.reserve1,
            block_hash,
            log_index,
            transaction_hash
        }
    }

    pub (crate) fn extract_reserve_info(log: &Log) -> (String, UpdateReserve) {
        let data = &log.data.to_vec();
        let parameters = ethers::abi::decode(&vec![ParamType::Uint(112), ParamType::Uint(112)], data).unwrap();
        let reserve0 = parameters[0].clone().into_uint().unwrap().to_string();
        let reserve1 = parameters[1].clone().into_uint().unwrap().to_string();
        let pair_address = format!("0x{}", log.address.into_token().to_string());
        (pair_address, UpdateReserve { reserve0, reserve1 })
    }
}
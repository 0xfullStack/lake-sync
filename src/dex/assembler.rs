use std::ops::{Add, Div, Mul, Sub, SubAssign};
use std::sync::Arc;
use diesel::QueryResult;

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
        let total_range = self.get_total_block_range(event).await;
        let mut blocks_remain = total_range.size;
        let mut meet_last_loop = false;
        let mut blocks_per_loop = event.blocks_per_loop();

        while blocks_remain > U64::zero() {

            let range_per_loop = Assembler::get_block_range_per_loop(total_range, blocks_per_loop, meet_last_loop, blocks_remain);
            let from = range_per_loop.from;
            let to = range_per_loop.to;
            match event {
                EventType::PairCreated => {
                    println!(" - - Pair created logs syncing from: {:?} to: {:?}", from, to);
                    let mut tasks = Vec::new();
                    let thread_count = event.max_threads_count().as_u32();
                    let range_per_loop = blocks_per_loop.div(thread_count);
                    for index in 0..thread_count {
                        let clone_self = self.clone();
                        let task = tokio::spawn(async move {
                            let from = from.add(range_per_loop.mul(index));
                            let to= from.add(range_per_loop).sub(1);

                            println!(" - 1 - Start pair created logs syncing from: {:?} to: {:?}", from, to);
                            let result = clone_self.fetch_pairs_logs(from, to).await;

                            match result {
                                Ok(logs) => {
                                    println!(" - 2 - Fetching {:?} pair created logs successfully from: {:?} to: {:?}", logs.len(), from, to);
                                    if logs.len() > 0 {
                                        clone_self.syncing_into_db(logs, event);
                                    }
                                }
                                Err(e) => {
                                    println!(" - 2 - Fetching pair created logs failure from: {:?} to: {:?}, error: {:?}, cut by half", from, to, e);
                                    println!();
                                }
                            }
                        });
                        tasks.push(task);
                    }

                    for task in tasks {
                        task.await;
                    }
                    println!("- - Pair created logs sync from {:?} to {:?} finished", from, to);
                    println!();
                }
                EventType::Sync => {
                    // let mut tasks = Vec::new();
                    // let thread_count = event.max_threads_count().as_u32();
                    // let range_per_loop = blocks_per_loop.div(thread_count);
                    // for index in 0..thread_count {
                    //     let clone_self = self.clone();
                    //     let task = tokio::spawn(async move {
                    //         let from = from.add(range_per_loop.mul(index));
                    //         let to= from.add(range_per_loop).sub(1);
                    //         clone_self.polling_reserve_logs(from, to).await;
                    //     });
                    //     tasks.push(task);
                    // }
                    //
                    // for task in tasks {
                    //     task.await;
                    // }
                    //
                    // println!("- - Reserve logs sync from {:?} to {:?} finished", from, to);
                    // println!();
                }
            }

            if meet_last_loop {
                break;
            }
            if blocks_remain.sub(blocks_per_loop) < blocks_per_loop {
                meet_last_loop = true;
            }
            blocks_remain.sub_assign(blocks_per_loop);
        }
    }

    pub async fn polling_reserve_logs(&self, from: U64, to: U64) {
        let result = self.fetch_reserve_logs(from, to).await;

        match result {
            Ok(logs_) => {
                println!(" - 2 - Fetching {:?} reserve logs successfully from: {:?} to: {:?}", logs_.len(), from.to_string(), to.to_string());
                if logs_.len() > 0 {
                    self.syncing_into_db(logs_, EventType::Sync);
                }
            }
            Err(e) => {
                println!(" - 2 - Fetching reserve logs failure from: {:?} to: {:?}, error: {:?}, cut by half", from.to_string(), to.to_string(), e);
            }
        }
    }

    fn syncing_into_db(&self, logs: Vec<Log>, event: EventType) {

        let conn = &self.pool.get().unwrap();
        let len = logs.len();
        let result: QueryResult<usize>;
        match event {
            EventType::PairCreated => {
                let mut records = Vec::with_capacity(len);
                for log in logs {
                    records.push(NewPair::construct_by(log, self.protocol.factory_address()));
                }
                result = models::batch_insert_pairs(records, conn);
            }
            EventType::Sync => {
                let mut records = Vec::with_capacity(len);
                for log in logs {
                    records.push(NewReserveLog::construct_by(log));
                }
                result = models::batch_insert_reserve_logs(records, conn);
            }
        }
        match result {
            Ok(count) => {
                println!(" - 3 - Insert {:?} records successfully", count);
            }
            Err(e) => {
                println!(" - 3 - Insert {:?} records failure: {:?}", len, e);
            }
        }
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

    async fn get_total_block_range(&self, event: EventType) -> BlockRange {
        let created_at_block_number = self.protocol.created_at_block_number();
        let current_synced_block_number = self.get_event_block_height_in_db(event);
        let mut from = created_at_block_number;
        if current_synced_block_number > created_at_block_number {
            from = current_synced_block_number + 1;
        }
        let to: U64 = self.client.get_block_number().await.unwrap();
        let size = to.sub(from).add(1);

        BlockRange { from, to, size }
    }

    fn get_block_range_per_loop(total_range: BlockRange, blocks_per_loop: U64, meet_last_loop: bool, blocks_remain: U64) -> BlockRange {
        let from;
        let to;

        if total_range.size <= blocks_per_loop {
            from = total_range.from;
            to = total_range.to;
        } else {
            from = total_range.from.add(total_range.size.sub(blocks_remain));
            if meet_last_loop {
                to = from.add(blocks_remain).sub(1);
            } else {
                to = from.add(blocks_per_loop).sub(1);
            }
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
    pub(crate) fn construct_by(log: Log, factory_address: H160) -> Self {
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

    pub(crate) fn construct_by(log: Log) -> Self {
        let data = &log.data.to_vec();
        let parameters = ethers::abi::decode(&vec![ParamType::Uint(112), ParamType::Uint(112)], data).unwrap();
        let pair_address = format!("0x{}", log.address.into_token().to_string());
        let block_number = log.block_number.unwrap().as_u64() as i64;
        let mut block_hash = serde_json::to_string(&log.block_hash.unwrap_or(H256::zero())).unwrap();
        let mut transaction_hash = serde_json::to_string(&log.transaction_hash.unwrap_or(H256::zero())).unwrap();
        block_hash.retain(|c| c != '\"');
        transaction_hash.retain(|c| c != '\"');

        NewReserveLog {
            pair_address,
            block_number,
            reserve0: parameters[0].clone().into_uint().unwrap().to_string(),
            reserve1: parameters[1].clone().into_uint().unwrap().to_string(),
            block_hash,
            transaction_hash
        }
    }

}
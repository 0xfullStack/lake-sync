use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Sub};
use std::sync::Arc;
use diesel::QueryResult;

use ethers::prelude::*;
use ethers::providers::Http;
use ethers::types::U64;
use ethers::prelude::ValueOrArray::Value;
use std::option::Option;
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
            let thread_count = event.max_threads_count().as_u32();
            let blocks_per_thread = blocks_per_loop.div(thread_count);
            if range_per_loop.size < blocks_per_thread {
                self.handle_task(from, to, event).await;
                meet_last_loop = true;
            } else {
                let mut threads = Vec::new();
                for index in 0..thread_count {
                    let clone_self = self.clone();
                    let thread_from = from.add(blocks_per_thread.mul(index));
                    let mut thread_to= thread_from.add(blocks_per_thread).sub(1);

                    if thread_from > from && thread_to > to { continue }
                    if thread_to > to { thread_to = to }

                    let thread = tokio::spawn(async move {
                        clone_self.handle_task(thread_from, thread_to, event).await;
                    });
                    threads.push(thread);
                }
                for thread in threads {
                    thread.await;
                }
            }
            scanned_block_cursor = Some(to);
            println!("- - {:?} Logs sync from {:?} to {:?} finished", event, from, to);
            println!();
        }
    }

    async fn handle_task(&self, from: U64, to: U64, event: EventType) {

        println!(" - 1 - Start {:?} logs syncing from: {:?} to: {:?}", event, from, to);

        let total_size = to.sub(from).add(1);
        let mut cut_factor: U64 = U64::one();
        let mut range = BlockRange { from, to, size: to.sub(from).add(1) };

        loop {

            let result;
            match event {
                EventType::PairCreated => {
                    result = self.fetch_pairs_logs(range.from, range.to).await;
                }
                EventType::Sync => {
                    result = self.fetch_reserve_logs(range.from, range.to).await;
                }
            }

            match result {
                Ok(logs) => {
                    println!(" - 2 - Fetching {:?} {:?} logs successfully from: {:?} to: {:?}", event, logs.len(), range.from, range.to);
                    if logs.len() > 0 {
                        self.syncing_logs_into_db(logs, event);
                    }
                    if cut_factor == U64::one() {
                        break;
                    } else {
                        let size = total_size.div(cut_factor);
                        let mut loop_from = range.to.add(1);
                        let mut loop_to = loop_from.add(size).sub(1);

                        if loop_from > from && loop_to > to { break; }
                        if loop_to > to { loop_to = to; }

                        range.from = loop_from;
                        range.to = loop_to;
                        range.size = size;
                    }
                }
                Err(e) => {
                    println!(" - 2 - Fetching {:?} logs failure from: {:?} to: {:?}, error: {:?}", event, range.from, range.to, e);

                    cut_factor.mul_assign(2);
                    let size = total_size.div(cut_factor);
                    range.from = from;
                    range.to = from.add(size).sub(1);
                    range.size = size;
                    println!("       - - -> Cut by half, with factor {:?} to retry {:?} syncing", cut_factor, event);
                }
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

    async fn fetch_pairs_logs(&self, from: U64, to: U64) -> Result<Vec<Log>, ProviderError> {
        let filter = Filter::default()
            .address(Value(self.protocol.factory_address()))
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

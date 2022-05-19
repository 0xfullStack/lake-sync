use std::sync::Arc;
use std::time::Duration;
use diesel::QueryResult;

use tokio;
use ethers::prelude::*;
use ethers::prelude::ValueOrArray::Value;
use ethers::types::U64;
use crate::db::postgres::PgPool;
use crate::dex::models::{NewPair, *};
use crate::{EventType, Protocol};

#[derive(Clone)]
pub struct Subscriber {
    pub node: String,
    pub protocol: Protocol,
    pool: Arc<PgPool>
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
                thread_self.0.subscribe_pair_created_logs(EventType::PairCreated).await;
            }
        });

        let pair_sync_thread = tokio::spawn(async move {
            loop {
                thread_self.1.subscribe_reserve_change_logs(EventType::Sync).await;
            }
        });

        tokio::join!(pair_created_thread, pair_sync_thread);
        Ok(true)
    }

    pub async fn subscribe_pair_created_logs(&self, event: EventType) -> std::io::Result<bool> {
        println!(" - 1 - Subscriber: start subscribe pair created logs");
        let ws = Ws::connect(self.node.clone()).await.unwrap();
        let provider = Provider::new(ws).interval(Duration::from_millis(500));
        let filter = Filter::default()
            .address(ValueOrArray::Value(self.protocol.factory_address()))
            .topic0(Value(event.topic_hash()));

        let stream_result = provider.subscribe_logs(&filter).await;
        match stream_result {
            Ok(mut stream) => {
                println!(" - 1 - Subscriber: subscribe pair created success");
                loop {
                    let next = stream.next().await;
                    match next {
                        Some(log) => {
                            println!(" - 1- Subscriber: receive new pair created logs: {:?}", log);
                            self.syncing_into_db(log, event);
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

    async fn subscribe_reserve_change_logs(&self, event: EventType) -> std::io::Result<bool> {
        println!(" - 2 - Subscriber: start subscribe reserve change logs");

        let ws = Ws::connect(self.node.clone()).await.unwrap();
        let provider = Provider::new(ws).interval(Duration::from_millis(500));
        let block_number = BlockNumber::Number(U64::from(10000835));
        let filter = Filter::default()
            .from_block(block_number)
            .topic0(Value(event.topic_hash()));

        let stream_result = provider.subscribe_logs(&filter).await;
        match stream_result {
            Ok(mut stream) => {
                println!(" - 2 - Subscriber: subscribe reserve change success");
                loop {
                    let next = stream.next().await;
                    match next {
                        Some(log) => {
                            println!(" - 2 - Subscriber: receive new reserve logs: {:?}", log);
                            self.syncing_into_db(log, event);
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

    fn syncing_into_db(&self, log: Log, event: EventType) {
        let conn = &self.pool.get().unwrap();
        let result: QueryResult<usize>;
        match event {
            EventType::PairCreated => {
                let factory_address = self.protocol.factory_address();
                let new_pairs = vec![NewPair::construct_by(log, factory_address)];
                result = batch_insert_pairs(new_pairs, conn);
            }
            EventType::Sync => {
                let new_reserve_logs = vec![NewReserveLog::construct_by(log)];
                result = batch_insert_reserve_logs(new_reserve_logs, conn);
            }
        }

        match result {
            Ok(count) => {
                println!(" - 3 - Subscriber Insert {:?} records successfully", count);
            }
            Err(e) => {
                println!(" - 3 - Subscriber Insert records failure: {:?}", e);
            }
        }
    }
}
use std::sync::Arc;
use ethers::prelude::*;
use ethers::abi::Tokenizable;
use ethers::abi::ParamType;
use crate::{Assembler, EventType, Node, PgPool, Protocol, Subscriber};
use crate::dex::models::{add_new_protocol, NewPair, NewReserveLog};

#[derive(Debug)]
pub enum State {
    Idle,
    Monitoring,
    Syncing,
    Completed(BlockNumber)
}

pub struct Aggregator {
    subscriber: Subscriber,
    assembler: Assembler,
    protocol: Protocol
}

impl Aggregator {
    pub fn make(node: Node, db_pool: Arc<PgPool>, protocol: Protocol) -> Aggregator {
        let protocol_info = protocol.protocol_info();
        let conn = &db_pool.clone().get().unwrap();
        add_new_protocol(protocol_info, conn);

        let assembler = Assembler::make(
            node.http,
            protocol.clone(),
            db_pool.clone()
        );
        let subscriber = Subscriber::make(
            node.ws,
            protocol.clone(),
            db_pool.clone()
        );
        Aggregator { assembler, subscriber, protocol }
    }

    pub async fn start_syncing(&self) {
        self.assembler.polling(EventType::Sync).await;
        self.assembler.polling(EventType::PairCreated).await;
        self.subscriber.start_watching().await;
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
            transaction_hash
        }
    }
}

impl NewReserveLog {

    pub(crate) fn construct_by(log: &Log) -> Self {
        let data = &log.data.to_vec();
        let parameters = ethers::abi::decode(&vec![ParamType::Uint(112), ParamType::Uint(112)], data).unwrap();
        let block_number = log.block_number.unwrap().as_u64() as i64;
        let log_index = log.log_index.unwrap().as_u64() as i64;
        let reserve0 = parameters[0].clone().into_uint().unwrap().to_string();
        let reserve1 = parameters[1].clone().into_uint().unwrap().to_string();
        let pair_address = format!("0x{}", log.address.into_token().to_string());
        let mut block_hash = serde_json::to_string(&log.block_hash.unwrap_or(H256::zero())).unwrap();
        let mut transaction_hash = serde_json::to_string(&log.transaction_hash.unwrap_or(H256::zero())).unwrap();
        block_hash.retain(|c| c != '\"');
        transaction_hash.retain(|c| c != '\"');

        NewReserveLog {
            pair_address, block_number,
            reserve0, reserve1,
            block_hash, log_index, transaction_hash
        }
    }
}
use std::env;
use dotenv::dotenv;
use std::sync::Arc;
use std::str::FromStr;
use std::time::Duration;
use super::models::{NewPair, NewProtocol};

use tokio;
use ethers::prelude::*;
use ethers::contract::Contract;
use ethers::prelude::ValueOrArray::Value;
use ethers::types::{U64, Address};
use ethers::providers::HttpClientError;
use crate::abi::abis::{IUniSwapV2Factory, IUniswapV2Pair};

pub struct Subscriber {
    pub node_url: String,
    pub factory_address: Address
}

impl Subscriber {
    pub fn make(node: String, factory: String) -> Subscriber {
        Subscriber {
            node_url: node.clone(),
            factory_address: Address::from_str(factory.as_str()).unwrap(),
        }
    }

    pub fn star_watch_events(&self) {
        self.pair_created_event();
        self.pair_sync_event();
    }

    pub fn stop_watching() {}

    async fn pair_created_event(&self) {
        let event_topic = TxHash::from_str("0x0d3648bd0f6ba80134a33ba9275ac585d9d315f0ad8355cddefde31afa28d0e9").unwrap();
        let ws = Ws::connect(self.node_url.clone()).await.unwrap();
        let provider = Provider::new(ws).interval(Duration::from_millis(500));
        let client = Arc::new(provider);

        let factory_contract = IUniSwapV2Factory::new(self.factory_address.clone(), Arc::clone(&client));
        let address = Address::from_str("0xdac17f958d2ee523a2206206994597c13d831ec7").unwrap();

        // let filter = factory_contract.transfer_filter();
        // let mut stream = filter.stream().await.unwrap();
        //
        // while let Some(block) = stream.next().await {
        //     dbg!(block);
        // }
    }

    async fn pair_sync_event(&self) {
        let event_topic = TxHash::from_str("0x1c411e9a96e071241c2f21f7726b17ae89e3cab4c78be50e062b03a9fffbbad1").unwrap();
        let ws = Ws::connect(self.node_url.clone()).await.unwrap();
        let provider = Provider::new(ws).interval(Duration::from_millis(500));

        let block_number = BlockNumber::Number(U64::from(10000835));
        let filter = Filter::default()
            .topic0(Value(event_topic))
            .from_block(block_number);

        let subscribe = provider.subscribe_logs(&filter).await;
    }
}
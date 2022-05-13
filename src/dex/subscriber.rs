use std::env;
use std::rc::Rc;
use std::sync::Arc;
use std::str::FromStr;
use std::time::Duration;
use diesel::query_dsl::InternalJoinDsl;
use dotenv::dotenv;
use ethers::abi::{ParamType, Tokenizable, Tokenize};
use super::models::{NewPair, NewProtocol};

use tokio;
use ethers::prelude::*;
use ethers::contract::Contract;
use ethers::prelude::FilterBlockOption::AtBlockHash;
use ethers::prelude::ValueOrArray::Value;
use ethers::types::{U64, Address};
use crate::abi::abis::{IUniSwapV2Factory, IUniswapV2Pair};

#[derive(Debug, Clone)]
pub struct Subscriber {
    pub node_url: String,
    pub factory_address: Address
}

enum SubscriptionError {
    ConnectionFailed,
    InvalidEncoding,
    UnKnown
}

impl Subscriber {
    pub fn make(node: String, factory: String) -> Subscriber {
        Subscriber {
            node_url: node.clone(),
            factory_address: Address::from_str(factory.as_str()).unwrap(),
        }
    }

    pub async fn sync_from_block_range(&self, from: BlockNumber, to: BlockNumber) {
        let event_topic = TxHash::from_str("0x0d3648bd0f6ba80134a33ba9275ac585d9d315f0ad8355cddefde31afa28d0e9").unwrap();
        let ws = Ws::connect(self.node_url.clone()).await.unwrap();

        let provider = Provider::new(ws);

        let filter = Filter::default()
            .topic0(Value(event_topic))
            .from_block(BlockNumber::Number(U64::from(14767384 )));
            // .to_block(BlockNumber::Number(U64::from( 14767319)))
            // .address(ValueOrArray::Value(self.factory_address))



        let logs = provider.get_logs(&filter).await.unwrap();


        for log in logs {
            let data = &log.data.to_vec();


            let parameters = ethers::abi::decode(&vec![ParamType::Address, ParamType::Uint(256)], data);
            // let object = serde_derive::Deserialize(data.into_token());



            println!("{:?}", Address::from(log.topics[1]));
            println!("{:?}", Address::from(log.topics[2]));
            println!("{:?}", parameters.unwrap());
        }

        // while let next = stream.next().await {
        //     match next {
        //         Some(log) => {
        //             dbg!(log);
        //         },
        //         None => {
        //             println!("pari created error occur");
        //             stream.unsubscribe().await;
        //             break;
        //         }
        //     }
        // }
        // Result::Ok(false)
    }

    pub async fn watching_with_guardian(&self) -> std::io::Result<bool> {

        let thread_self = (self.clone(), self.clone());
        let pair_created_thread = tokio::spawn(async move {
            loop {
                thread_self.0.pair_created_event().await;
            }
        });

        let pair_sync_thread = tokio::spawn(async move {
            // loop {
            //     thread_self.1.pair_sync_event().await;
            // }
        });

        tokio::join!(pair_created_thread, pair_sync_thread);
        Result::Ok(true)
    }

    pub async fn pair_created_event(&self) -> std::io::Result<bool> {
        println!("pari created subscribe begin");

        let event_topic = TxHash::from_str("0x0d3648bd0f6ba80134a33ba9275ac585d9d315f0ad8355cddefde31afa28d0e9").unwrap();
        let ws = Ws::connect(self.node_url.clone()).await.unwrap();

        let provider = Provider::new(ws);

        let block_number = BlockNumber::Number(U64::from(10850000));
        let filter = Filter::default()
            .from_block(block_number)
            .to_block(BlockNumber::Latest)
            .address(ValueOrArray::Value(self.factory_address))
            .topic0(Value(event_topic));




        let mut stream = provider.subscribe_logs(&filter).await.unwrap();

        while let next = stream.next().await {
            match next {
                Some(log) => {
                    dbg!(log);
                },
                None => {
                    println!("pari created error occur");
                    stream.unsubscribe().await;
                    break;
                }
            }
        }
        Result::Ok(false)
    }

    async fn pair_sync_event(&self) -> std::io::Result<bool> {
        let event_topic = TxHash::from_str("0x1c411e9a96e071241c2f21f7726b17ae89e3cab4c78be50e062b03a9fffbbad1").unwrap();
        let ws = Ws::connect(self.node_url.clone()).await.unwrap();
        let provider = Provider::new(ws).interval(Duration::from_millis(500));

        let block_number = BlockNumber::Number(U64::from(10000835));
        let filter = Filter::default()
            .from_block(block_number)
            .topic0(Value(event_topic));


        let mut stream = provider.subscribe_logs(&filter).await.unwrap();
        while let next = stream.next().await {
            match next {
                Some(log) => {
                    dbg!(log);
                },
                None => {
                    println!("sync error occur");
                    stream.unsubscribe().await;
                    break;
                }
            }
        }
        Result::Ok(false)
    }

    pub fn response_to_reserves_changing(address: Address) {

    }

    fn store_into_db(&self) {
        unimplemented!()
    }

    fn filter_target_pairs_change() {
        unimplemented!()
    }
}
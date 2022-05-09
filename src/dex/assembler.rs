
// use crate::db::postgres::NewPair;
// use crate::db::schema::pairs::{factory, pair_address, token0};

// async fn fetch_all_pairs(factory: &str) -> Result<Uint, E> {
//     Result::Ok(Uint::from(1))
// }
//
// async fn fetch_pair_address(index: Uint) -> Result<String, E> {
//     Result::Ok(String::from("dddd"))
// }

// async fn assemble_pair_info(factory: Address, pair_address: Address) -> Result<NewPair, E> {
//
//     dotenv::dotenv().ok();
//     let node = &env::var("INFURA_MAINNET").unwrap().as_str();
//     let http = web3::transports::Http::new(node);
//     let web3s = web3::Web3::new(http);
//
//     let address = Address::from_str("0x3139Ffc91B99aa94DA8A2dc13f1fC36F9BDc98eE").unwrap();
//     let contract = Contract::from_json(web3s.eth(), address, include_bytes!("uniswap_v2_pair.json")).unwrap();
//
//     let token0: Address = contract.query("token0", (), None, Options::default(), None).await.unwrap();
//     let token1: Address = contract.query("token1", (), None, Options::default(), None).await.unwrap();
//     let reserves: Address = contract.query("getReserves", (), None, Options::default(), None).await.unwrap();
//
//     println!("got token0: {:?}", token0);
//
//     Result::Ok(NewPair{
//         pair_address: pair_address::to_string().as_str(),
//         pair_index: 0,
//         token0: token0::to_string().as_str(),
//         token1: token1::to_string().as_str(),
//         reserve0: 0,
//         reserve1: 0,
//         factory: factory::to_string().as_str(),
//         created_at_timestamp: None,
//         created_at_block_number: None
//     })
// }


use std::env;
use std::ops::Add;
use std::str::FromStr;
use std::time::Duration;

use web3::{
    contract::{Contract, Options},
    types::{FilterBuilder, Address, H160, U256, CallRequest},
    helpers as w3h,
    ethabi::Uint,
    transports::{Http, WebSocket}, Web3, Error,
    futures::StreamExt
};

use dotenv::dotenv;
use hex_literal::hex;
use web3::ethabi::TopicFilter;
use web3::types::{BlockNumber, Filter};
use crate::db::models::Pair;

pub struct Ethereum;

impl Ethereum  {

    pub fn init() -> Ethereum {
        let eth = Ethereum{};
        eth
    }

    async fn check_start_index() {

    }

    // async fn get_account_balance(&self, address: &str) -> U256 {
    //     let web3 = Web3::new(self.websocket);
    //     let account = Address::from_str(address).unwrap();
    //     let balance = web3.eth().balance(account, None).await.unwrap();
    //     balance
    // }

    pub async fn start_sync_from(&self, index: i64) -> Result<Pair, Error> {

        dotenv::dotenv().ok();
        let node_http = &env::var("INFURA_MAINNET_HTTP").unwrap();
        let http = Http::new(node_http).unwrap();

        let web3 = Web3::new(http);
        let factory_address = Address::from_str("0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f").unwrap();
        let factory_contract = Contract::from_json(web3.eth(), factory_address, include_bytes!("../abi/uniswap_v2_factory.json")).unwrap();

        let pairs_length: Uint = factory_contract
            .query("allPairsLength", (), None, Options::default(), None)
            .await
            .unwrap();
        let pair_address: Address = factory_contract
            .query("allPairs", (42_u32,), None, Options::default(), None)
            .await
            .unwrap();
        let pair_contract = Contract::from_json(web3.eth(), pair_address, include_bytes!("../abi/uniswap_v2_pair.json")).unwrap();
        let pair_token0: Address = pair_contract.query("token0", (), None, Options::default(), None).await.unwrap();
        let pair_token1: Address = pair_contract.query("token1", (), None, Options::default(), None).await.unwrap();
        // let token_reserves = pair_contract.query("getReserves", (), None, Options::default(), None).await.unwrap();


        let new_pair = Pair {
            pair_address: w3h::to_string(&pair_address),
            pair_index: index,
            token0: w3h::to_string(&pair_token0),
            token1: w3h::to_string(&pair_token1),
            reserve0: 1,
            reserve1: 1,
            factory: w3h::to_string(&factory_address)
        };

        Result::Ok(new_pair)
    }

    async fn handle_pair_created_event(&self) {

        dotenv::dotenv().ok();
        let node_ws = &env::var("INFURA_MAINNET_WS").unwrap();
        let ws = WebSocket::new(node_ws).await.unwrap();
        let web3 = Web3::new(ws.clone());

        let factory_address = Address::from_str("0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f").unwrap();
        let factory_contract = Contract::from_json(web3.eth(), factory_address, include_bytes!("../abi/uniswap_v2_factory.json")).unwrap();

        let pair_created_topic = vec![hex!("0d3648bd0f6ba80134a33ba9275ac585d9d315f0ad8355cddefde31afa28d0e9").into()];
        let filter = FilterBuilder::default()
            .address(vec![factory_address])
            .topics(Some(pair_created_topic), None, None, None)
            .build();

        // web3.eth_subscribe().subscribe_logs(filter).await
        let log_filter = web3.eth_filter().create_logs_filter(filter).await.unwrap();
        let logs_stream = log_filter.stream(Duration::from_secs(1));

        web3::futures::pin_mut!(logs_stream);

        let log = logs_stream.next().await.unwrap();
        println!("got log: {:?}", log);


    }

    async fn handle_sync_event() {
        let sync_topic = vec![hex!("1c411e9a96e071241c2f21f7726b17ae89e3cab4c78be50e062b03a9fffbbad1").into()];
        let filter = FilterBuilder::default()
            .topics(Some(sync_topic), None, None, None)
            .from_block(BlockNumber::from(10000835))
            .build();

        // web3.eth_subscribe().subscribe_logs(filter);
    }
}


use tokio;
use web3::{
    contract::{Contract, Options},
    // types::{FilterBuilder, Address, H160, U256, CallRequest},
    helpers as w3h,
    ethabi::Uint,
    transports::{Http, WebSocket}, Web3, Error,
    futures::{future, StreamExt}
};

use std::ops::Add;
use std::str::FromStr;
use std::time::Duration;
use std::env;
use std::sync::Arc;
use hex_literal::hex;

use ethers::prelude::*;
use ethers::prelude::{BlockNumber, Address};
use url::Url;
use crate::ValueOrArray::Value;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {

    env_logger::init();
    dotenv::dotenv().ok();

    // tokio::task::spawn(async move {
    // let ws = Ws::connect("wss://mainnet.infura.io/ws/v3/428cbc6033df445b88ebf0e2221f5b96").await.unwrap();
    // let provider = Arc::new(Provider::new(ws).interval(Duration::from_millis(500)));
    // let address = Address::from_str("0xdac17f958d2ee523a2206206994597c13d831ec7").unwrap();
    // let contract = token_contract::TokenContract::new(address, provider);
    //
    // let filter = contract.transfer_filter();
    // let mut stream = filter.stream().await.unwrap();
    //
    // while let Some(block) = stream.next().await {
    //     dbg!(block);
    // }
    // }).await;

    let url = Url::parse("wss://mainnet.infura.io/ws/v3/428cbc6033df445b88ebf0e2221f5b96").expect("Can't connect to case count URL");
    let ws = Ws::connect(url).await?;


    let filter_address = Address::from_str("0x0d4a11d5eeaac28ec3f61d100daf4d40471f1852").unwrap();
    let sync_topic = TxHash::from_str("1c411e9a96e071241c2f21f7726b17ae89e3cab4c78be50e062b03a9fffbbad1").unwrap();
    let block_number = BlockNumber::Number(U64::from(10000835));

    let filter = Filter::default()
        .address(Value(filter_address))
        .topic0(Value(sync_topic))
        .from_block(block_number);

    let provider = Provider::new(ws);

    let subscribe= provider.subscribe_logs(&filter).await;

    match subscribe {
        Result::Ok(stream) => {
            stream
                .map(|log| {
                    log.address
                })
                .for_each(|address| {
                    println!("got address: {:?}", address);
                    future::ready(())
                }).await;
        }
        Result::Err(error) => {
            println!("error occured");
        }
    }
    // println!("endpoint ready?: {}", result.ready());

    // match result {
    //     Ok(ws) => {
    //         println!("Success, get provider, etc...");
    //     },
    //     Err(error) => {
    //         panic!("Error: {}", error);
    //     }
    // }






    // NODE_MAINNET_WS=wss://chain-node.cryptoless.io/eth/mainnet/rawapi-ws
    //     NODE_MAINNET_HTTP=https://chain-node.cryptoless.io/eth/mainnet/rawapi

    // let ws = WebSocket::new(&env::var("INFURA_MAINNET_WS").unwrap()).await;
    //
    // let mut web3;
    // match ws {
    //     Result::Ok(socket) => {
    //
    //
    //         web3 = Web3::new(socket.clone());
    //     }
    //     Result::Err(error) => {
    //         println!("{:?}", error);
    //         panic!("Error")
    //     }
    // }
    //
    // // let factory_address = Address::from_str("0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f").unwrap();
    // // let factory_contract = Contract::from_json(web3.eth(), factory_address, include_bytes!("abi/uniswap_v2_factory.json")).unwrap();
    // // let pair_created_topic = vec![hex!("0d3648bd0f6ba80134a33ba9275ac585d9d315f0ad8355cddefde31afa28d0e9").into()];
    // // let filter = FilterBuilder::default()
    // //     .address(vec![factory_address])
    // //     .topics(Some(pair_created_topic), None, None, None)
    // //     .from_block(BlockNumber::from(10000835))
    // //     .build();
    //
    // // let subscribe = web3.eth_subscribe().subscribe_logs(filter).await?;
    // //
    // // subscribe.for_each(|log| {
    // //     println!("got log: {:?}", log);
    // //     future::ready(())
    // // })
    // // .await;
    //
    // let filter_address = Address::from_str("0x0d4a11d5eeaac28ec3f61d100daf4d40471f1852").unwrap();
    // let sync_topic = vec![hex!("1c411e9a96e071241c2f21f7726b17ae89e3cab4c78be50e062b03a9fffbbad1").into()];
    // let filter = FilterBuilder::default()
    //     .address(vec![filter_address])
    //     .topics(Some(sync_topic), None, None, None)
    //     .from_block(BlockNumber::from(10000835))
    //     .build();
    //
    // let subscribe = web3.eth_subscribe().subscribe_logs(filter).await;
    // match subscribe {
    //     Result::Ok(stream) => {
    //         // stream
    //
    //         stream
    //             .map(|log| {
    //                 log.unwrap().address
    //             })
    //             .for_each(|address| {
    //                 println!("got address: {:?}", address);
    //                 future::ready(())
    //             }).await;
    //     }
    //     Result::Err(error) => {
    //         println!("error occured");
    //     }
    // }
    //
    //
    //
    // // let log_filter = web3.eth_filter().create_logs_filter(filter).await.unwrap();
    // // let logs_stream = log_filter.stream(Duration::from_secs(1));
    // //
    // // web3::futures::pin_mut!(logs_stream);
    //
    // // let log = logs_stream.next().await.unwrap();
    // // println!("got log: {:?}", log);
    //
    // // println!("got log: {:?}", log);
    //
    // // let log_filter = web3.eth_filter().create_logs_filter(filter).await.unwrap();
    // // let logs_stream = log_filter.stream(Duration::from_secs(1));
    // //
    // // web3::futures::pin_mut!(logs_stream);
    // //
    // // let log = logs_stream.next().await.unwrap();
    // // println!("got log: {:?}", log);

    Ok(())
}

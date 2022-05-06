
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
use std::env;
use std::fmt::Display;
use std::rc::Rc;
use dotenv::dotenv;
use std::sync::Arc;
use std::str::FromStr;
use diesel::QueryResult;
use ethers::abi::Tokenizable;
use super::models::{NewPair, NewProtocol, *};

use tokio;
use ethers::prelude::*;
use ethers::contract::Contract;
use ethers::providers::{JsonRpcClient, Http};
use ethers::types::{U64, Address};
use ethers::providers::HttpClientError;
use hex::ToHex;
use crate::abi::abis::{IUniSwapV2Factory, IUniswapV2Pair};
use crate::db::postgres::PgPool;
use crate::dex::models;

pub struct Assembler {
    pub node_url: String,
    pub factory_address: Address,
    client: Arc<Provider::<Http>>,
    pool: Rc<PgPool>
}

impl Assembler {
    pub fn make(node: String, factory: String, pool: Rc<PgPool>) -> Assembler {
        Assembler {
            node_url: node.clone(),
            factory_address: Address::from_str(factory.as_str()).unwrap(),
            client: Arc::new(Provider::<Http>::try_from(node.clone()).unwrap()),
            pool
        }
    }

    pub async fn polling(&self) -> std::io::Result<bool> {

        let length = self.fetch_pairs_length().await.unwrap();

        //
        // let _protocol = NewProtocol {
        //     name: "Uniswap Protocol".to_string(),
        //     official_url: Option::Some("https://uniswap.org/".to_string()),
        //     network: "ETHEREUM_MAINNET".to_string(),
        //     description: Some("Swap, earn, and build on the leading decentralized crypto trading protocol.".to_string()),
        //     symbol: Some("uniswap-v2".to_string()),
        //     router_address: "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D".to_string(),
        //     factory_address: "0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f".to_string().to_uppercase()
        // };
        //
        // let conn = &self.pool.get().unwrap();
        // models::add_new_protocol(_protocol, conn);
        for index in 0..length {
            let address = self.fetch_pair_address(index).await.unwrap();
            let new_pair = self.fetch_pair_info(address, index).await.unwrap();

            println!("NewPair info: {:?}", new_pair);

            let conn = &self.pool.get().unwrap();
            match models::add_new_pair(new_pair, conn) {
                Ok(_) => {
                    println!("Success");
                }
                Err(e) => {
                    println!("{}", e);
                    break;
                }
            }
        }
        Ok(true)
    }

    async fn fetch_pairs_length(&self) -> Result<i64, HttpClientError> {
        let factory_contract = IUniSwapV2Factory::new(self.factory_address.clone(), Arc::clone(&self.client));
        let pairs_length: U256 = factory_contract.all_pairs_length().call().await.unwrap();
        Ok(pairs_length.as_u64() as i64)
    }

    async fn fetch_pair_address(&self, index: i64) -> Result<Address, HttpClientError> {
        let factory_contract = IUniSwapV2Factory::new(self.factory_address.clone(), Arc::clone(&self.client));
        let pair_address: Address = factory_contract.all_pairs(U256::from(index)).call().await.unwrap();

        Ok(pair_address)
    }

    async fn fetch_pair_info(&self, address: Address, index: i64) -> Result<NewPair, HttpClientError> {
        let pair_contract = IUniswapV2Pair::new(address.clone(), Arc::clone(&self.client));

        let token0: Address = pair_contract.token_0().call().await.unwrap();
        let token1: Address = pair_contract.token_1().call().await.unwrap();

        let mut pair_address_format = serde_json::to_string(&address).unwrap();
        let mut token0_format = serde_json::to_string(&token0).unwrap();
        let mut token1_format = serde_json::to_string(&token1).unwrap();
        let mut factory_address_format = serde_json::to_string(&self.factory_address).unwrap();

        pair_address_format.retain(|c| c != '\"');
        token0_format.retain(|c| c != '\"');
        token1_format.retain(|c| c != '\"');
        factory_address_format.retain(|c| c != '\"');

        // token0.to
        let (reserve0_, reserve1_, _) = pair_contract.get_reserves().call().await.unwrap();
        // let mid_price = f64::powi(10.0, 18 - 6) * reserve1 as f64 / reserve0 as f64;

        // let s = token0.fmt()

        // println!("{}", to);
        Ok(
            NewPair {
                pair_address: pair_address_format,
                pair_index: index,
                token0: token0_format,
                token1: token1_format,
                reserve0: reserve0_.to_string(),
                reserve1: reserve1_.to_string(),
                factory: factory_address_format
            }
        )
    }
}
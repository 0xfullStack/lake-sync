use std::env;
use dotenv::dotenv;
use std::sync::Arc;
use std::str::FromStr;
use super::models::{NewPair, NewProtocol};

use tokio;
use ethers::prelude::*;
use ethers::contract::Contract;
use ethers::providers::{JsonRpcClient, Http};
use ethers::types::{U64, Address};
use ethers::providers::HttpClientError;
use crate::abi::abis::{IUniSwapV2Factory, IUniswapV2Pair};

pub struct Assembler {
    pub node_url: String,
    pub factory_address: Address,
    client: Arc<Provider::<Http>>,
}

impl Assembler {
    pub fn make(node: String, factory: String) -> Assembler {
        Assembler {
            node_url: node.clone(),
            factory_address: Address::from_str(factory.as_str()).unwrap(),
            client: Arc::new(Provider::<Http>::try_from(node.clone()).unwrap())
        }
    }

    pub async fn fetch_pairs_length(&self) -> Result<U256, HttpClientError> {
        let factory_contract = IUniSwapV2Factory::new(self.factory_address.clone(), Arc::clone(&self.client));
        let pairs_length: U256 = factory_contract.all_pairs_length().call().await.unwrap();
        Ok(pairs_length)
    }

    pub async fn fetch_pair_address(&self, index: U256) -> Result<Address, HttpClientError> {
        let factory_contract = IUniSwapV2Factory::new(self.factory_address.clone(), Arc::clone(&self.client));
        let pair_address: Address = factory_contract.all_pairs(index).call().await.unwrap();

        Ok(pair_address)
    }

    pub async fn fetch_pair_info(&self, address: Address, index: U256) -> Result<NewPair, HttpClientError> {
        let pair_contract = IUniswapV2Pair::new(address.clone(), Arc::clone(&self.client));

        let token0: Address = pair_contract.token_0().call().await.unwrap();
        let token1: Address = pair_contract.token_1().call().await.unwrap();

        let (reserve0_, reserve1_, _) = pair_contract.get_reserves().call().await.unwrap();
        // let mid_price = f64::powi(10.0, 18 - 6) * reserve1 as f64 / reserve0 as f64;

        Ok(
            NewPair {
                pair_address: address.to_string(),
                pair_index: 0,
                token0: token0.to_string(),
                token1: token1.to_string(),
                reserve0: reserve0_ as i64,
                reserve1: reserve1_ as i64,
                factory: self.factory_address.to_string()
            }
        )
    }
}
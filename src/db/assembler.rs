
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
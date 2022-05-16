use std::ops::AddAssign;
use diesel::prelude::*;
use diesel::table;
use crate::db::schema::{protocols, pairs};
use crate::db::schema::pairs::{ pair_address, reserve0, reserve1, block_number };

#[derive(Insertable, Debug)]
#[table_name="protocols"]
pub struct NewProtocol {
    pub name: String,
    pub official_url: Option<String>,
    pub network: String,
    pub description: Option<String>,
    pub symbol: Option<String>,
    pub router_address: String,
    pub factory_address: String,
}

#[derive(Insertable, Debug)]
#[table_name="pairs"]
pub struct NewPair {
    pub pair_address: String,
    pub pair_index: i64,
    pub token0: String,
    pub token1: String,
    pub reserve0: String,
    pub reserve1: String,
    pub factory: String,
    pub block_number: i64
}

pub fn add_uniswap_v2(conn: &PgConnection)  {
    let _protocol = NewProtocol {
        name: "Uniswap Protocol".to_string(),
        official_url: Option::Some("https://uniswap.org/".to_string()),
        network: "ETHEREUM_MAINNET".to_string(),
        description: Option::Some("Swap, earn, and build on the leading decentralized crypto trading protocol.".to_string()),
        symbol: Option::Some("uniswap-v2".to_string()),
        router_address: "7a250d5630B4cF539739dF2C5dAcb4c659F2488D".to_string(),
        factory_address: "5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f".to_string().to_lowercase()
    };

    match add_new_protocol(_protocol, conn) {
        Ok(_) => {
            println!("Insert success");
        }
        Err(e) => {
            println!("{:?}", e);
        }
    }
}

pub fn add_new_protocol(protocol: NewProtocol, conn: &PgConnection) -> QueryResult<usize> {
    diesel::insert_into(protocols::table)
        .values(&protocol)
        .execute(conn)
}

pub fn update_protocol(protocol: NewProtocol, conn: &PgConnection) -> QueryResult<usize> {
    QueryResult::Ok(1)
}

pub fn batch_insert_pairs(pairs: Vec<NewPair>, conn: &PgConnection) -> QueryResult<usize> {
    diesel::insert_into(pairs::table)
        .values(&pairs)
        .execute(conn)
}

type DBError = Box<dyn std::error::Error + Send + Sync>;
pub fn get_addresses(conn: &PgConnection, from_block_number: i64, to_block_number: i64) -> Result<Vec<String>, DBError> {
    let address_list = pairs::table
        .filter(block_number.between(from_block_number, to_block_number))
        .select(pair_address)
        .load::<String>(conn)?;
    Ok(address_list)
}

#[derive(AsChangeset, Debug)]
#[table_name="pairs"]
pub struct NewReserve {
    pub reserve0: String,
    pub reserve1: String,
}

pub fn batch_update_reserves(reserves: Vec<(String, NewReserve)>, conn: &PgConnection) -> QueryResult<usize> {
    let mut execute_success_count = 0;
    for element in reserves {
        // println!("Start Update for pair_address: {:?}, reserve: {:?}", element.0, element.1);
        match update_reserve(element.0, element.1, conn) {
            Ok(_) => {
                execute_success_count.add_assign(1);
            }
            Err(e) => {
                println!("Update reserve error: {:?}", e);
            }
        }
    }

    Ok(execute_success_count)
}

pub fn update_reserve(pair_address_: String, reserve: NewReserve, conn: &PgConnection) -> QueryResult<usize> {
    diesel::update(pairs::table.filter(pair_address.eq(pair_address_.as_str())))
        .set(&reserve)
        .execute(conn)
}

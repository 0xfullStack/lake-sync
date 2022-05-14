use std::ops::AddAssign;
use diesel::prelude::*;
use diesel::{Insertable};
use diesel::table;
use crate::db::schema::{protocols, pairs};
use crate::db::schema::pairs::{pair_address, reserve0, reserve1};

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
    pub factory: String
}

/*
        let _protocol = create_protocol(
            &connection,
            "Uniswap Protocol",
            Some("https://uniswap.org/"),
            "ETHEREUM_MAINNET",
            Some("Swap, earn, and build on the leading decentralized crypto trading protocol."),
            Some("uniswap-v2"),
            "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D",
            "0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f"
        );
     */
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


#[derive(AsChangeset, Debug)]
#[table_name="pairs"]
pub struct NewReserve {
    pub reserve0: String,
    pub reserve1: String,
}

pub fn batch_update_reserves(reserves: Vec<(String, NewReserve)>, conn: &PgConnection) -> QueryResult<usize> {
    let mut execute_success_count = 0;
    for element in reserves {
        println!("Start Update for pair_address: {:?}, reserve: {:?}", element.0, element.1);
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
    diesel::update(pairs::table)
        .filter( pair_address.eq(pair_address_))
        .set(&reserve)
        .execute(conn)
}

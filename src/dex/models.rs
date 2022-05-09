use diesel::prelude::*;
use diesel::{Insertable};
use diesel::table;
use crate::db::schema::{protocols, pairs};

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
    pub reserve0: i64,
    pub reserve1: i64,
    pub factory: String
}

impl NewProtocol {
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

        // let new_protocol = NewProtocol { name, official_url, network, description, symbol, router_address, factory_address };
        diesel::insert_into(protocols::table)
            .values(&protocol)
            .execute(conn)
    }

    pub fn update_protocol(protocol: NewProtocol, conn: &PgConnection) -> QueryResult<usize> {
        QueryResult::Ok(1)
    }
}


impl NewPair {
    pub fn add_new_pair(pair: NewPair, conn: &PgConnection) -> QueryResult<usize> {
        diesel::insert_into(pairs::table)
            .values(&pair)
            .execute(conn)
    }
    pub fn update_pair(new_pair: NewPair, conn: &PgConnection) -> QueryResult<usize> {
        QueryResult::Ok(1)
    }
}

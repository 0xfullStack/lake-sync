
use diesel::prelude::{Queryable, QueryResult};
use diesel::prelude::*;
use diesel::pg::PgConnection;
use super::schema::{pairs, protocols};
use super::schema::pairs::dsl::pairs as get_paris;
use super::schema::protocols::dsl::protocols as get_protocols;
use super::postgres::{NewPair, NewProtocol};

use serde::Serialize;

#[derive(Queryable, Debug, Serialize)]
pub struct Protocol {
    pub id: i64,
    pub name: String,
    pub official_url: Option<String>,
    pub network: String,
    pub description: Option<String>,
    pub symbol: Option<String>,
    pub router_address: String,
    pub factory_address: String,
}
pub type ProtocolId = i64;
pub type PairId = i64;

impl Protocol {

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
    pub fn add_protocol(new_protocol: NewProtocol, conn: &PgConnection) -> QueryResult<usize> {

        // let new_protocol = NewProtocol { name, official_url, network, description, symbol, router_address, factory_address };
        diesel::insert_into(protocols::table)
            .values(&new_protocol)
            .execute(conn)
    }

    pub fn update_protocol(new_protocol: NewProtocol, conn: &PgConnection) -> QueryResult<usize> {
        QueryResult::Ok(1)
    }

    pub fn rm_protocol(id: ProtocolId, conn: &PgConnection) -> QueryResult<usize> {
        diesel::delete(get_protocols.find(id)).execute(conn)
    }

    pub fn get_protocols(conn: &PgConnection) -> QueryResult<Vec<Protocol>> {
        get_protocols.order(protocols::id.desc()).load::<Protocol>(conn)
    }
}

#[derive(Queryable, Debug)]
pub struct Pair {
    pub id: i64,
    pub pair_address: String,
    pub pair_index: i64,
    pub token0: String,
    pub token1: String,
    pub reserve0: i64,
    pub reserve1: i64,
    pub factory: String
}

impl Pair {
    pub fn add_pair(new_pair: NewPair, conn: &PgConnection) -> QueryResult<usize> {
        diesel::insert_into(pairs::table)
            .values(&new_pair)
            .execute(conn)
    }

    pub fn update_pair(new_pair: NewPair, conn: &PgConnection) -> QueryResult<usize> {
        QueryResult::Ok(1)
    }

    pub fn rm_pair(id: PairId, conn: &PgConnection) -> QueryResult<usize> {
        diesel::delete(get_paris.find(id)).execute(conn)
    }

    pub fn get_pairs(conn: &PgConnection) -> QueryResult<Vec<Pair>> {
        get_paris.order(pairs::id.desc()).load::<Pair>(conn)
    }
}

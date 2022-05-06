
use diesel::prelude::{Insertable, Queryable, QueryResult};
use diesel::pg::PgConnection;
use diesel::{QueryDsl, RunQueryDsl};

use super::schema::{pairs, protocols};
use super::schema::pairs::dsl::pairs as get_paris;
use super::schema::protocols::dsl::protocols as get_protocols;

#[derive(Insertable, Debug)]
#[table_name="protocols"]
pub struct Protocol {
    pub name: String,
    pub official_url: Option<String>,
    pub network: String,
    pub description: Option<String>,
    pub symbol: Option<String>,
    pub router_address: String,
    pub factory_address: String,
}

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
    pub fn add_protocol(protocol: Protocol, conn: &PgConnection) -> QueryResult<usize> {

        // let new_protocol = NewProtocol { name, official_url, network, description, symbol, router_address, factory_address };
        diesel::insert_into(protocols::table)
            .values(&protocol)
            .execute(conn)
    }

    pub fn update_protocol(protocol: Protocol, conn: &PgConnection) -> QueryResult<usize> {
        QueryResult::Ok(1)
    }

    pub fn rm_protocol(id: i64, conn: &PgConnection) -> QueryResult<usize> {
        diesel::delete(get_protocols.find(id)).execute(conn)
    }
}

#[derive(Insertable, Debug)]
#[table_name="pairs"]
pub struct Pair {
    pub pair_address: String,
    pub pair_index: i64,
    pub token0: String,
    pub token1: String,
    pub reserve0: i64,
    pub reserve1: i64,
    pub factory: String
}

impl Pair {
    pub fn add_pair(pair: Pair, conn: &PgConnection) -> QueryResult<usize> {
        diesel::insert_into(pairs::table)
            .values(&pair)
            .execute(conn)
    }

    pub fn update_pair(new_pair: Pair, conn: &PgConnection) -> QueryResult<usize> {
        QueryResult::Ok(1)
    }

    pub fn rm_pair(id: i64, conn: &PgConnection) -> QueryResult<usize> {
        diesel::delete(get_paris.find(id)).execute(conn)
    }
}

use std::ops::AddAssign;
use diesel::prelude::*;
use diesel::table;
use crate::db::schema::{Protocol, Pair, ReserveLog};
use crate::db::schema::Pair::{ pair_address, reserve0, reserve1, block_number };

#[derive(Insertable, Debug)]
#[table_name="Protocol"]
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
#[table_name="Pair"]
pub struct NewPair {
    pub pair_address: String,
    pub factory_address: String,
    pub token0: String,
    pub token1: String,
    pub block_number: i64,
    pub block_hash: String,
    pub transaction_hash: String,
    pub reserve0: String,
    pub reserve1: String
}

#[derive(Insertable, Debug)]
#[table_name="ReserveLog"]
pub struct NewReserveLog {
    pub pair_address: String,
    pub factory_address: String,
    pub reserve0: String,
    pub reserve1: String,
    pub block_number: i64,
    pub block_hash: String,
    pub transaction_hash: String
}

pub fn add_new_protocol(protocol: NewProtocol, conn: &PgConnection) -> QueryResult<usize> {
    diesel::insert_into(Protocol::table)
        .values(&protocol)
        .execute(conn)
}

pub fn batch_insert_pairs(pairs: Vec<NewPair>, conn: &PgConnection) -> QueryResult<usize> {
    diesel::insert_into(Pair::table)
        .values(&pairs)
        .execute(conn)
}

pub fn batch_insert_reserve_logs(logs: Vec<NewReserveLog>, conn: &PgConnection) -> QueryResult<usize> {
    diesel::insert_into(ReserveLog::table)
        .values(&logs)
        .execute(conn)
}

// type DBError = Box<dyn std::error::Error + Send + Sync>;
// pub fn get_addresses(conn: &PgConnection, from_block_number: i64, to_block_number: i64) -> Result<Vec<String>, DBError> {
//     let address_list = pairs::table
//         .filter(block_number.between(from_block_number, to_block_number))
//         .select(pair_address)
//         .load::<String>(conn)?;
//     Ok(address_list)
// }

#[derive(AsChangeset, Debug)]
#[table_name="Pair"]
pub struct NewReserve {
    pub reserve0: String,
    pub reserve1: String,
}

pub fn batch_update_reserves(reserves: Vec<(String, NewReserve)>, conn: &PgConnection) -> QueryResult<usize> {
    let mut execute_success_count = 0;
    for element in reserves {
        // println!("Start Update for pair_address: {:?}, reserve: {:?}", element.0, element.1);
        match update_pair_reserve(element.0, element.1, conn) {
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

pub fn update_pair_reserve(pair_address_: String, reserve: NewReserve, conn: &PgConnection) -> QueryResult<usize> {
    diesel::update(Pair::table.filter(pair_address.eq(pair_address_.as_str())))
        .set(&reserve)
        .execute(conn)
}

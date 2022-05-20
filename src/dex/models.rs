use std::ops::AddAssign;
use diesel::prelude::*;
use diesel::query_dsl::methods::ThenOrderDsl;
use crate::db::schema::{Protocol, Pair, ReserveLog};
use crate::db::schema::Pair::{pair_address, block_number};
use crate::db::schema::ReserveLog::{block_number as reserveLog_block_number, log_index, pair_address as reserveLog_pair_address, reserve0, reserve1};
use field_count::FieldCount;

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

pub fn add_new_protocol(protocol: NewProtocol, conn: &PgConnection) -> QueryResult<usize> {
    diesel::insert_into(Protocol::table)
        .values(&protocol)
        .execute(conn)
}

#[derive(Insertable, Debug, Clone)]
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

pub fn batch_insert_pairs(pairs: Vec<NewPair>, conn: &PgConnection) -> QueryResult<usize> {
    diesel::insert_into(Pair::table)
        .values(&pairs)
        .execute(conn)
}

pub fn get_last_pair_block_height(conn: &PgConnection) -> QueryResult<i64> {
    Pair::table
        .select(block_number)
        .order_by(block_number.desc())
        .first(conn)
}

#[derive(Insertable, FieldCount, Debug, Clone)]
#[table_name="ReserveLog"]
pub struct NewReserveLog {
    pub pair_address: String,
    pub reserve0: String,
    pub reserve1: String,
    pub block_number: i64,
    pub block_hash: String,
    pub transaction_hash: String,
    pub log_index: i64
}

pub fn get_last_reserve_log_block_height(conn: &PgConnection) -> QueryResult<i64> {
    ReserveLog::table
        .select(reserveLog_block_number)
        .order_by(reserveLog_block_number.desc())
        .first(conn)
}

pub fn insert_reserve_logs(logs: &[NewReserveLog], conn: &PgConnection) -> QueryResult<usize> {
    diesel::insert_into(ReserveLog::table)
        .values(logs)
        .execute(conn)
}

const DB_MAX_INSERT_PARAMETERS: usize = 65535;
pub fn batch_insert_reserve_logs(logs: Vec<NewReserveLog>, conn: &PgConnection) -> QueryResult<usize> {

    let field_count = NewReserveLog::field_count();
    let element_len = logs.len();

    if element_len * field_count >= DB_MAX_INSERT_PARAMETERS {

        let mut total_insert: usize = 0;
        let mut start_index: usize = 0;
        let mut end_index: usize;
        let mut count_per_loop: usize = element_len;
        let mut meet_last_loop = false;

        while count_per_loop * field_count >= DB_MAX_INSERT_PARAMETERS {
            count_per_loop = count_per_loop / 2;
        }

        end_index = start_index + count_per_loop;
        while !meet_last_loop {

            let count = insert_reserve_logs(&logs[start_index..end_index], conn).unwrap_or(0);
            total_insert.add_assign(count);

            if end_index + count_per_loop >= element_len {
                meet_last_loop = true;
                start_index = end_index;
                end_index = element_len;
            } else {
                start_index = end_index;
                end_index = start_index + count_per_loop;
            }
        }

        let count = insert_reserve_logs(&logs[start_index..end_index], conn).unwrap_or(0);
        total_insert.add_assign(count);
        Ok(total_insert)
    } else {
        let count = insert_reserve_logs(&logs, conn).unwrap_or(0);
        Ok(count)
    }
}

#[derive(AsChangeset, Debug)]
#[table_name="Pair"]
pub struct UpdateReserve {
    pub reserve0: String,
    pub reserve1: String,
}

pub fn get_latest_pair_reserves(pair_address_: String, conn: &PgConnection) -> QueryResult<(String, String)> {
    // SELECT * FROM "ReserveLog" WHERE pair_address = '0xe0cc5afc0ff2c76183416fb8d1a29f6799fb2cdf' ORDER BY (block_number, log_index) DESC
    ReserveLog::table
        .select((reserve0, reserve1))
        .filter(reserveLog_pair_address.eq(pair_address_.as_str()))
        .order_by((reserveLog_block_number.desc(), log_index.desc()))
        .limit(1)
        .get_result(conn)
}

pub fn update_pair_reserve(pair_address_: String, reserve: UpdateReserve, conn: &PgConnection) -> QueryResult<usize> {
    // UPDATE "Pair" SET reserve0 = '1', reserve1 = '2' WHERE pair_address = '0x295685c8fe08d8192981d21ea1fe856a07443920';

    diesel::update(Pair::table.filter(pair_address.eq(pair_address_.as_str())))
        .set(&reserve)
        .execute(conn)
}

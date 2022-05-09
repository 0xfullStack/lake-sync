use actix::{Actor, SyncContext};


use diesel::prelude::*;
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PoolError, PooledConnection};
use actix_web::{error, Error};

pub type PgPool = Pool<ConnectionManager<PgConnection>>;
type PgPooledConnection = PooledConnection<ConnectionManager<PgConnection>>;

pub fn init_pool(database_url: &str) -> Result<PgPool, PoolError> {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder().build(manager)
}

pub struct Database {
    pub pool: PgPool
}

impl Database {
    pub fn get_connection(&self) -> Result<PgPooledConnection, Error> {
        self.pool.get().map_err(|e| error::ErrorInternalServerError(e))
    }
}

impl Actor for Database {
    type Context = SyncContext<Self>;
}
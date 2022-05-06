use std::env;
use std::ops::Deref;
use dotenv;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PoolError, PooledConnection};
use actix::{Addr, SyncArbiter, Handler, Message, Actor, SyncContext};
use actix::fut::err;
use actix_web::{error, Error};
use serde_derive::{Serialize, Deserialize};
use super::models::{Protocol, Pair};
use super::schema::{pairs, protocols};

const DB_MAX_CONNECTION: usize = 3;

pub type PgPool = Pool<ConnectionManager<PgConnection>>;
type PgPooledConnection = PooledConnection<ConnectionManager<PgConnection>>;

pub fn init_pool(database_url: &str) -> Result<PgPool, PoolError> {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder().build(manager)
}

#[derive(Clone)]
pub struct State {
    pub inner: Addr<Db>,
}

impl State {
    pub fn init() -> State {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = init_pool(&database_url).expect("Failed to create pool");
        let addr = SyncArbiter::start(DB_MAX_CONNECTION, move || Db(pool.clone()));
        let state = State {
            inner: addr.clone(),
        };
        state
    }
    pub fn get(&self) -> &Addr<Db> {
        &self.inner
    }
}


pub struct Db(pub PgPool);

impl Db {
    pub fn get_connection(&self) -> Result<PgPooledConnection, Error> {
        self.0.get().map_err(|e| error::ErrorInternalServerError(e))
    }
}

impl Actor for Db {
    type Context = SyncContext<Self>;
}


impl Message for Protocol {
    type Result = Result<(), Error>;
}

impl Message for Pair {
    type Result = Result<(), Error>;
}

impl Handler<Pair> for Db {
    type Result = Result<(), Error>;

    fn handle(&mut self, msg: Pair, ctx: &mut Self::Context) -> Self::Result {
        Pair::add_pair(msg, self.get_connection()?.deref())
            .map(|_| ())
            .map_err(|_| error::ErrorInternalServerError("Faied inserting new pair"))
    }
}

impl Handler<Protocol> for Db {
    type Result = Result<(), Error>;

    fn handle(&mut self, msg: Protocol, ctx: &mut Self::Context) -> Self::Result {
        Protocol::add_protocol(msg, self.get_connection()?.deref())
            .map(|_| ())
            .map_err(|_| error::ErrorInternalServerError("Faied inserting new protocol"))
    }
}

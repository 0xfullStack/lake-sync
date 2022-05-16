use std::rc::Rc;
use std::sync::Arc;
use ethers::prelude::{Address, BlockNumber};
use crate::{Assembler, Node, PgPool, Protocol, Subscriber};
use crate::dex::models::add_new_protocol;

#[derive(Debug)]
pub enum State {
    Idle,
    Monitoring,
    Syncing,
    Completed(BlockNumber)
}

pub struct Aggregator {
    subscriber: Subscriber,
    assembler: Assembler,
    protocol: Protocol
}

impl Aggregator {

    pub fn make(node: Node, db_pool: Rc<PgPool>, protocol: Protocol) -> Aggregator {
        let protocol_info = protocol.protocol_info();
        let conn = &db_pool.clone().get().unwrap();
        add_new_protocol(protocol_info, conn);

        let assembler = Assembler::make(
            node.http,
            protocol.clone(),
            db_pool.clone()
        );
        let subscriber = Subscriber::make(
            node.ws,
            protocol.clone().factory_address(),
            db_pool.clone()
        );
        Aggregator { assembler, subscriber, protocol }
    }

    pub async fn start_syncing(&self) {

        self.assembler.polling_pairs().await;
        // self.assembler.polling_reserves().await;
        // self.subscriber.listening().await;
    }
}

use std::rc::Rc;
use std::sync::Arc;
use ethers::prelude::{Address, BlockNumber};
use crate::{Assembler, Node, PgPool, Protocol, Subscriber};

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
        let assembler = Assembler::make(
            node.http,
            protocol.factory_address(),
            db_pool.clone()
        );

        let subscriber = Subscriber::make(
            node.ws,
            protocol.factory_address(),
            db_pool.clone()
        );
        Aggregator { assembler, subscriber, protocol }
    }

    pub async fn start_syncing(&self) {
        self.assembler.polling().await;
        self.subscriber.listening().await;
    }
}

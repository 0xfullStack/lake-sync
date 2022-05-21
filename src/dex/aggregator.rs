use std::sync::Arc;
use ethers::prelude::BlockNumber;
use crate::{Assembler, EventType, Node, PgPool, Protocol, Subscriber};
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
    pub fn make(node: Node, db_pool: Arc<PgPool>, protocol: Protocol) -> Aggregator {
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
            protocol.clone(),
            db_pool.clone()
        );
        Aggregator { assembler, subscriber, protocol }
    }

    pub async fn start_syncing(&self) {
        self.assembler.polling(EventType::Sync).await;
        self.assembler.polling(EventType::PairCreated).await;
        self.subscriber.start_watching().await;
    }
}

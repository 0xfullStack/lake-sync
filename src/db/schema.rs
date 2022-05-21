table! {
    Pair (id) {
        id -> Int8,
        pair_address -> Bpchar,
        factory_address -> Bpchar,
        token0 -> Bpchar,
        token1 -> Bpchar,
        block_number -> Int8,
        block_hash -> Text,
        transaction_hash -> Text,
    }
}

table! {
    Protocol (id) {
        id -> Int8,
        name -> Varchar,
        official_url -> Nullable<Varchar>,
        network -> Varchar,
        description -> Nullable<Text>,
        symbol -> Nullable<Varchar>,
        router_address -> Bpchar,
        factory_address -> Bpchar,
    }
}

table! {
    ReserveLog (id) {
        id -> Int8,
        pair_address -> Bpchar,
        reserve0 -> Text,
        reserve1 -> Text,
        block_number -> Int8,
        block_hash -> Text,
        transaction_hash -> Text,
        log_index -> Int8,
    }
}

allow_tables_to_appear_in_same_query!(
    Pair,
    Protocol,
    ReserveLog,
);

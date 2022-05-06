table! {
    pairs (id) {
        id -> Int8,
        pair_address -> Bpchar,
        pair_index -> Int8,
        token0 -> Bpchar,
        token1 -> Bpchar,
        reserve0 -> Int8,
        reserve1 -> Int8,
        factory -> Bpchar,
    }
}

table! {
    protocols (id) {
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

allow_tables_to_appear_in_same_query!(
    pairs,
    protocols,
);

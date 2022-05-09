use ethers::prelude::*;

// Generate the type-safe contract bindings by providing the ABI definition
abigen!(
    IUniSwapV2Factory,
    "./src/abi/uniswap_v2_factory.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

abigen!(
    IUniswapV2Pair,
    "./src/abi/uniswap_v2_pair.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

// Will use simple codes instead.
// abigen!(
//     IUniswapV2Pair,
//     r#"[
//         function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast)
//     ]"#,
// );
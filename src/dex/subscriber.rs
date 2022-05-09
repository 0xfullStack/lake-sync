

// #[tokio::main]
// async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {

// tokio::task::spawn(async move {
// let ws = Ws::connect("wss://mainnet.infura.io/ws/v3/428cbc6033df445b88ebf0e2221f5b96").await.unwrap();
// let provider = Arc::new(Provider::new(ws).interval(Duration::from_millis(500)));
// let address = Address::from_str("0xdac17f958d2ee523a2206206994597c13d831ec7").unwrap();
// let contract = token_contract::TokenContract::new(address, provider);
//
// let filter = contract.transfer_filter();
// let mut stream = filter.stream().await.unwrap();
//
// while let Some(block) = stream.next().await {
//     dbg!(block);
// }
// }).await;

// let url = Url::parse("wss://mainnet.infura.io/ws/v3/428cbc6033df445b88ebf0e2221f5b96").expect("Can't connect to case count URL");
// let ws = Ws::connect(url).await?;
//
//
// let filter_address = Address::from_str("0x0d4a11d5eeaac28ec3f61d100daf4d40471f1852").unwrap();
// let sync_topic = TxHash::from_str("1c411e9a96e071241c2f21f7726b17ae89e3cab4c78be50e062b03a9fffbbad1").unwrap();
// let block_number = BlockNumber::Number(U64::from(10000835));
//
// let filter = Filter::default()
//     .address(Value(filter_address))
//     .topic0(Value(sync_topic))
//     .from_block(block_number);
//
// let provider = Provider::new(ws);
//
// let subscribe= provider.subscribe_logs(&filter).await;
//
// match subscribe {
//     Result::Ok(stream) => {
//         stream
//             .map(|log| {
//                 log.address
//             })
//             .for_each(|address| {
//                 println!("got address: {:?}", address);
//                 future::ready(())
//             }).await;
//     }
//     Result::Err(error) => {
//         println!("error occured");
//     }
// }
// println!("endpoint ready?: {}", result.ready());

// match result {
//     Ok(ws) => {
//         println!("Success, get provider, etc...");
//     },
//     Err(error) => {
//         panic!("Error: {}", error);
//     }
// }




//
// tokio::task::spawn(async move {
// let ws = Ws::connect("wss://mainnet.infura.io/ws/v3/428cbc6033df445b88ebf0e2221f5b96").await.unwrap();
// let provider = Arc::new(Provider::new(ws).interval(Duration::from_millis(500)));
// let address = Address::from_str("0xdac17f958d2ee523a2206206994597c13d831ec7").unwrap();
// let contract = token_contract::TokenContract::new(address, provider);
//
// let filter = contract.transfer_filter();
// let mut stream = filter.stream().await.unwrap();
//
// while let Some(block) = stream.next().await {
//     dbg!(block);
// }
// }).await;



// // let pair_created_topic = vec![hex!("0d3648bd0f6ba80134a33ba9275ac585d9d315f0ad8355cddefde31afa28d0e9").into()];
// // let filter = FilterBuilder::default()
// //     .address(vec![factory_address])
// //     .topics(Some(pair_created_topic), None, None, None)
// //     .from_block(BlockNumber::from(10000835))
// //     .build();
//
// // let subscribe = web3.eth_subscribe().subscribe_logs(filter).await?;
// //
// // subscribe.for_each(|log| {
// //     println!("got log: {:?}", log);
// //     future::ready(())
// // })
// // .await;
//
// let filter_address = Address::from_str("0x0d4a11d5eeaac28ec3f61d100daf4d40471f1852").unwrap();
// let sync_topic = vec![hex!("1c411e9a96e071241c2f21f7726b17ae89e3cab4c78be50e062b03a9fffbbad1").into()];
// let filter = FilterBuilder::default()
//     .address(vec![filter_address])
//     .topics(Some(sync_topic), None, None, None)
//     .from_block(BlockNumber::from(10000835))
//     .build();
//
// let subscribe = web3.eth_subscribe().subscribe_logs(filter).await;
// match subscribe {
//     Result::Ok(stream) => {
//         // stream
//
//         stream
//             .map(|log| {
//                 log.unwrap().address
//             })
//             .for_each(|address| {
//                 println!("got address: {:?}", address);
//                 future::ready(())
//             }).await;
//     }
//     Result::Err(error) => {
//         println!("error occured");
//     }
// }
//
//
//
// // let log_filter = web3.eth_filter().create_logs_filter(filter).await.unwrap();
// // let logs_stream = log_filter.stream(Duration::from_secs(1));
// //
// // web3::futures::pin_mut!(logs_stream);
//
// // let log = logs_stream.next().await.unwrap();
// // println!("got log: {:?}", log);
//
// // println!("got log: {:?}", log);
//
// // let log_filter = web3.eth_filter().create_logs_filter(filter).await.unwrap();
// // let logs_stream = log_filter.stream(Duration::from_secs(1));
// //
// // web3::futures::pin_mut!(logs_stream);
// //
// // let log = logs_stream.next().await.unwrap();
// // println!("got log: {:?}", log);

//     Ok(())
// }

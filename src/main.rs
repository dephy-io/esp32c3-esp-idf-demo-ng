// pub mod app_tests;
pub mod build_env;
pub mod crypto;
pub mod net;
pub mod nostr;
pub mod preludes;

use edge_executor::LocalExecutor;
use esp_idf_svc::hal::task::block_on;
use log::{debug, info};
use preludes::*;
use std::sync::Arc;

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let executor: Arc<LocalExecutor> = Arc::new(Default::default());

    let executor_move = executor.clone();
    info!("ETH_ADDRESS_STRING: {}", &*crypto::ETH_ADDRESS_STRING);
    info!("NOSTR_PUBKEY_STRING: {}", &*crypto::NOSTR_PUBKEY_STRING);

    block_on(executor.run(async_main(executor_move)));
    log::info!("Hello, world03!");
}

async fn async_main(executor: Arc<LocalExecutor<'_>>) {
    log::info!("Hello, world11!");
    log::debug!("Hello, world!");
    log::info!("Hello, world1!");
    // app_tests::test_ethereum_address();
    executor.spawn(t2()).await;
}

async fn t2() {
    log::debug!("Hello, world2!");
    log::info!("Hello, world13!");
}

// pub mod app_tests;
pub mod build_env;
pub mod crypto;
pub mod net;
pub mod nostr;
pub mod preludes;

use edge_executor::LocalExecutor;
use esp_idf_svc::{hal::task::block_on, timer::EspTaskTimerService};
use net::{get_public_ip, ntp_sync, wifi_create_loop};
use nostr::{create_random_event, send_new_event};
use preludes::*;
use std::sync::Arc;

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let _mounted_eventfs = esp_idf_svc::io::vfs::MountedEventfs::mount(5).unwrap();

    let executor: Arc<LocalExecutor> = Arc::new(Default::default());

    let executor_move = executor.clone();
    info!("NOSTR_PUBKEY_STRING: {}", &*crypto::NOSTR_PUBKEY_STRING);

    if let Err(e) = block_on(executor.run(async_main(executor_move))) {
        error!("Main task failed: {}", e);
    }
}

async fn async_main(executor: Arc<LocalExecutor<'_>>) -> Result<()> {
    let timer = EspTaskTimerService::new()?;
    let _wifi = wifi_create_loop(timer.clone()).await?;
    let _ntp = ntp_sync(timer.clone()).await?;

    if let Ok(Some(ip)) = get_public_ip() {
        info!("Public IP: {}", ip);
    } else {
        warn!("Failed to get public IP");
    }

    let nostr_task = executor.spawn(nostr_loop(timer));
    nostr_task.await?;
    Ok(())
}

async fn nostr_loop(timer: EspTaskTimerService) -> Result<()> {
    let mut t = timer.timer_async()?;
    loop {
        send_new_event(create_random_event()).await?;
        let _ = t.after(Duration::from_secs(APP_SEND_LOOP_DURATION)).await?;
    }
}

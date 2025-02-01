use crate::preludes::*;
use embedded_svc::http::client::Client as HttpClient;
use embedded_svc::http::Method;
use embedded_svc::io::Write;
use embedded_svc::utils::io;
use esp_idf_svc::eventloop::*;
use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::handle::RawHandle;
use esp_idf_svc::http::client::{
    Configuration as HttpConfiguration, EspHttpConnection, FollowRedirectsPolicy,
};
use esp_idf_svc::nvs::*;
use esp_idf_svc::sntp::EspSntp;
use esp_idf_svc::sntp::SyncStatus;
use esp_idf_svc::sys::{esp_crt_bundle_attach, esp_http_client_close};
use esp_idf_svc::timer::*;
use esp_idf_svc::wifi::*;
use heapless::Vec as HeaplessVec;

static COMMON_HEADERS: &'static [(&'static str, &'static str); 3] = &[
    ("User-Agent", "dephy-esp32-client/0.1.0"),
    ("accept", "*/*"),
    ("content-type", "application/json"),
];

async fn wifi_create(wifi: &mut AsyncWifi<&mut EspWifi<'_>>) -> Result<(), EspError> {
    wifi.stop().await?;
    wifi.start().await?;
    info!("Wifi started");

    wifi.connect().await?;
    info!("Wifi connected");

    wifi.wait_netif_up().await?;
    info!("Wifi netif up");

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
    info!("Wifi Interface info: {:?}", ip_info);

    Ok(())
}

pub async fn wifi_create_loop(
    timer: EspTaskTimerService,
) -> Result<esp_idf_svc::wifi::EspWifi<'static>, EspError> {
    let mut t = timer.timer_async()?;

    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;
    let peripherals = Peripherals::take()?;

    let mut esp_wifi = EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs.clone()))?;
    let mut wifi = AsyncWifi::wrap(&mut esp_wifi, sys_loop.clone(), timer)?;

    let wifi_config = if WIFI_PASSWD.len() > 0 {
        ClientConfiguration {
            ssid: WIFI_SSID.try_into().unwrap(),
            password: WIFI_PASSWD.try_into().unwrap(),
            auth_method: AuthMethod::WPA2Personal,
            ..Default::default()
        }
    } else {
        ClientConfiguration {
            ssid: WIFI_SSID.try_into().unwrap(),
            auth_method: AuthMethod::None,
            ..Default::default()
        }
    };
    wifi.set_configuration(&Configuration::Client(wifi_config))?;

    loop {
        match wifi_create(&mut wifi).await {
            Ok(_) => {
                return Ok(esp_wifi);
            }
            Err(e) => {
                // wifi.stop().await?;
                warn!("Wifi error: {:?}", e);
                warn!("Wifi not connected, retrying in 5s...");
                t.after(Duration::from_secs(5)).await?;
            }
        }
    }
}

pub async fn ntp_sync(timer: EspTaskTimerService) -> Result<EspSntp<'static>, EspError> {
    info!("Waiting for NTP time sync.");
    let mut t = timer.timer_async()?;
    let sntp = EspSntp::new_default()?;
    loop {
        match sntp.get_sync_status() {
            SyncStatus::Completed => {
                info!("Got time from NTP, now: {:?}", now_secs());
                break;
            }
            _ => {
                t.after(Duration::from_secs(3)).await?;
            }
        }
    }
    Ok(sntp)
}

pub fn request_text<'a, 'b>(
    url: &str,
    method: Option<Method>,
    user_headers: Option<&'a [(&'a str, &'a str)]>,
    body_buf: Option<&'b [u8]>,
) -> Result<(u16, String)> {
    let mut buf = [0u8; 1024];
    let (status, bytes_read) = request(url, method, user_headers, body_buf, Some(&mut buf))?;
    let buf = &buf[..bytes_read];
    let ret = std::str::from_utf8(&buf)?;
    debug!(
        "Response body (truncated to {} bytes): {:?}",
        bytes_read, ret
    );
    Ok((status, ret.to_string()))
}

pub fn request<'a, 'b>(
    url: &'a str,
    method: Option<Method>,
    user_headers: Option<&'a [(&'a str, &'a str)]>,
    body_buf: Option<&'b [u8]>,
    res_buf: Option<&'a mut [u8]>,
) -> Result<(u16, usize)> {
    let mut client = create_default_http_client()?;
    let mut headers: HeaplessVec<_, 8> = HeaplessVec::new();
    for h in COMMON_HEADERS.iter() {
        let _ = headers.push(*h);
    }
    // let len = if let Some(ref buf) = body_buf {buf.len().into()} else {"0"};
    // let _ = headers.push(("content-length", len));

    if let Some(h) = user_headers {
        for h in h {
            let _ = headers.push(*h);
        }
    }
    let mut request = client.request(
        if let Some(m) = method { m } else { Method::Get },
        url,
        headers.as_slice(),
    )?;
    if let Some(buf) = body_buf {
        request.write_all(buf)?;
        request.flush()?;
    }
    let mut response = request.submit()?;

    // Process response
    let status = response.status();
    let (_headers, mut body) = response.split();

    let ret = if let Some(buf) = res_buf {
        let bytes_read = io::try_read_full(&mut body, buf).map_err(|e| e.0)?;
        Ok((status, bytes_read))
    } else {
        Ok((status, 0))
    };
    let client = client.release();
    unsafe {
        esp!(esp_http_client_close(client.handle()))?;
    }
    drop(client);
    ret
}

pub fn create_default_http_client() -> Result<HttpClient<EspHttpConnection>> {
    let http = HttpConfiguration {
        buffer_size: None,
        buffer_size_tx: None,
        timeout: Some(Duration::from_secs(18)),
        follow_redirects_policy: FollowRedirectsPolicy::FollowGetHead,
        client_certificate: None,
        private_key: None,
        use_global_ca_store: true,
        crt_bundle_attach: Some(esp_crt_bundle_attach),
        raw_request_body: false,
    };
    let http = EspHttpConnection::new(&http)?;
    Ok(HttpClient::wrap(http))
}

pub fn get_public_ip() -> Result<Option<String>> {
    let (status, body) = request_text("https://ipinfo.io/ip", Some(Method::Get), None, None)?;
    if status == 200 {
        Ok(Some(body))
    } else {
        Ok(None)
    }
}

use paste::paste;
use std::env;
use std::fs;
use std::path::Path;
use std::str::FromStr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    embuild::espidf::sysenv::output();
    build_env()?;
    Ok(())
}

fn build_env() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("build_env.rs");
    dotenvy::from_filename("build.env")?;
    let mut lines = vec![];

    macro_rules! env_string {
        ($name: expr, $default: expr) => {
            paste! {
                lines.push(format!(
                    "pub static {}: &'static str = \"{}\";",
                    $name,
                    env::var($name).unwrap_or($default.into())
                ));
            }
        };
    }
    macro_rules! env_number {
        ($name: expr, $type: ty, $default: expr) => {
            paste! {
                lines.push(format!(
                    "pub static {}: {} = {};",
                    $name,
                    stringify!($type),
                    env::var($name).map(|s| $type::from_str(s.as_str()).unwrap()).unwrap_or($default)
                ));
            }
        };
    }

    env_string!(
        "DEPHY_ENDPOINT_HTTP",
        "https://send.testnet.dephy.io/dephy/signed_message"
    );
    env_string!("DEPHY_ENDPOINT_MQTT", "mqtt://demo-edge.dephy.io:1883");
    env_number!("APP_SEND_LOOP_DURATION", u64, 10);
    env_number!("APP_SEND_LOOP_SIZE", u8, 12);
    env_number!("KEY_ENTROPY_WAIT_DURATION", u64, 3600);
    // env_number!("LOGGER_LEVEL", u8, 3);
    env_string!(
        "TARGET_ADDRESS_HEX",
        "1111111111111111111111111111111111111111"
    );

    env_string!("WIFI_SSID", "");
    env_string!("WIFI_PASSWD", "");

    fs::write(&dest_path, lines.join("\n")).unwrap();
    println!("cargo:rerun-if-changed=build.env");

    Ok(())
}

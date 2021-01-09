extern crate reqwest;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate confy;
extern crate passwords;
extern crate cached;

use cached::proc_macro::cached;
use std::path::Path;
use passwords::PasswordGenerator;
use serde_derive::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct ProxyConfig {
    port: i64,
    authentication_key: String,
    site_id: String,
    http_auth_username: String,
    http_auth_password: String,
}

impl Default for ProxyConfig {
    fn default() -> Self {
         Self {
            port: 4000,
            authentication_key: String::from("arbnco_auth_key_goes_here"),
            site_id: String::from("site_id_goes_here"),
            http_auth_username: String::from("openhab"),
            http_auth_password: PasswordGenerator{
                length: 24, numbers: true, lowercase_letters: true,
                spaces: false, uppercase_letters: true, symbols: true,
                strict: true, exclude_similar_characters: false}.generate_one().unwrap(),
        }
    }
}

#[derive(Clone)]
struct SensorDataResult {

}

/// Returns the sensor data. This function is cached.
///
/// The most recent note from ARBNCO: “Also as a note, please limit API calls
/// to 10/min or 600/hr"
#[cached(time=25)]
fn get_sensor_data() -> Result<SensorDataResult, String> {
    let request_url = format!("well.arbnco.com/api/v1/sites/{site_id}/readings/data_range",
                              site_id = "0");

    let response = reqwest::blocking::get(&request_url);

    match response {
        Ok(resp) => {Ok(SensorDataResult{})},
        Err(err) => Err(format!("Response error: {}", err)),
    }
}


fn main() {
    let cfg: ProxyConfig = confy::load_path(Path::new("./config.toml")).unwrap();
    if cfg.authentication_key == ProxyConfig::default().authentication_key {
        println!("Please add the authentication (API) key in the configuration file (config.toml), and run again.");
        return;
    }
}

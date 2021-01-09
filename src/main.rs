extern crate reqwest;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate confy;
extern crate passwords;
extern crate cached;
extern crate actix_web;

use std::collections::HashMap;
use actix_web::dev::ServiceRequest;
use actix_web_httpauth::extractors::basic::BasicAuth;
use std::io::{Error, ErrorKind};
use actix_web_httpauth::middleware::HttpAuthentication;
use actix_web::{web, App, HttpRequest, HttpServer, Responder};
use cached::proc_macro::cached;
use std::path::Path;
use passwords::PasswordGenerator;
use serde_derive::{Serialize, Deserialize};
use lazy_static::lazy_static;

/// The configuration file structure for the proxy.
#[derive(Debug, Serialize, Deserialize)]
struct ProxyConfig {
    port: u16,
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

/// The sensor data result structure for returning to openHAB.
#[derive(Clone, Debug)]
struct SensorDataResult {
    temperature: f64,
    humidity: f64,
    co2: f64,
    ambient_light_sensor: f64,
    total_volatile_organic_compounds: f64,
    particulate_matter: f64,
    particulate_matter_25: f64,
    particulate_matter_10: f64,
}

/// Returns the sensor data. This function is cached.
///
/// The most recent note from ARBNCO: “Also as a note, please limit API calls
/// to 10/min or 600/hr"
#[cached(time=25)]
fn get_sensor_data() -> Result<SensorDataResult, String> {
    let request_url = format!("https://well.arbnco.com/api/v1/sites/{site_id}/readings",
                              site_id = CFG.site_id);

    let client = reqwest::blocking::Client::new();
    let response = client.get(&request_url).header("Authorization", &CFG.authentication_key).send();

    match response {
        Ok(resp) => {
            let deserialised: SensorDataResponse = serde_json::from_str(resp
                .text().unwrap().as_str()).unwrap();
            dbg!(&deserialised.data_range.maximum_date);
            dbg!(deserialised.data.len());
            let latest = deserialised.data.get(&deserialised.data_range.maximum_date).unwrap();

            let data: SensorDataResult = SensorDataResult{
                temperature: latest.temperature.med.unwrap_or(-99.0),
                humidity: latest.humidity.med.unwrap_or(-99.0),
                co2: latest.co2.med.unwrap_or(-99.0),
                ambient_light_sensor: latest.als.med.unwrap_or(-99.0),
                total_volatile_organic_compounds: latest.tvoc.med.unwrap_or(-99.0),
                particulate_matter: latest.pm.med.unwrap_or(-99.0),
                particulate_matter_10: latest.pm10.med.unwrap_or(-99.0),
                particulate_matter_25: latest.pm25.med.unwrap_or(-99.0),
            };
            Ok(data)
        },
        Err(err) => Err(format!("Response error: {}", err)),
    }
}

lazy_static! {
    /// Lazily load the global (static) configuration file.
    static ref CFG: ProxyConfig = confy::load_path(Path::new("./config.toml")).unwrap();
}

/// Validator function for HTTP Authentication
async fn validator(
    req: ServiceRequest,
    cred: BasicAuth,
) -> Result<ServiceRequest, actix_web::Error> {
        let uid = cred.user_id();
        let pass = cred.password();
        if uid == CFG.http_auth_username.as_str() &&
            pass.is_some() &&
            pass.unwrap() == CFG.http_auth_password.as_str() {
            return Ok(req);
        }
        return Err(actix_web::Error::from(Error::new(ErrorKind::PermissionDenied, "Invalid credentials")))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if CFG.authentication_key == ProxyConfig::default().authentication_key {
        println!("Please add the authentication (API) key in the configuration file (config.toml), and run again.");
        std::process::exit(1);
    }

    let resp = get_sensor_data().unwrap();
    println!("{:#?}", resp);

    HttpServer::new(|| {
        let auth = HttpAuthentication::basic(validator);
        App::new()
            .wrap(auth)
            .route("/", web::get().to(greet))
    })
    .bind(("127.0.0.1", CFG.port))?
    .run()
    .await
}

async fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", &name)
}


#[derive(Serialize, Deserialize)]
pub struct SensorDataResponse {
    #[serde(rename = "total_records")]
    total_records: i64,

    #[serde(rename = "page_size")]
    page_size: i64,

    #[serde(rename = "page")]
    page: i64,

    #[serde(rename = "total_pages")]
    total_pages: i64,

    #[serde(rename = "data_range")]
    data_range: DataRange,

    #[serde(rename = "data")]
    data: HashMap<String, Datum>,
}

#[derive(Serialize, Deserialize)]
pub struct Datum {
    #[serde(rename = "count")]
    count: i64,

    #[serde(rename = "temperature")]
    temperature: Stats,

    #[serde(rename = "humidity")]
    humidity: Stats,

    #[serde(rename = "co2")]
    co2: Stats,

    #[serde(rename = "als")]
    als: Stats,

    #[serde(rename = "tvoc")]
    tvoc: Stats,

    #[serde(rename = "pm")]
    pm: Stats,

    #[serde(rename = "pm25")]
    pm25: Stats,

    #[serde(rename = "pm10")]
    pm10: Stats,
}

#[derive(Serialize, Deserialize)]
pub struct Stats {
    #[serde(rename = "min")]
    min: Option<f64>,

    #[serde(rename = "med")]
    med: Option<f64>,

    #[serde(rename = "max")]
    max: Option<f64>,
}

#[derive(Serialize, Deserialize)]
pub struct DataRange {
    #[serde(rename = "minimum_date")]
    minimum_date: String,

    #[serde(rename = "maximum_date")]
    maximum_date: String,
}

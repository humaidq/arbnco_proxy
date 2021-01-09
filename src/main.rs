extern crate reqwest;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate confy;
extern crate passwords;
extern crate cached;
extern crate actix_web;

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

/// The ARBNCO data structure for deserialisation.
#[derive(Debug, Deserialize)]
struct SensorDataResponse {

}

/// The sensor data result structure for returning to openHAB.
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

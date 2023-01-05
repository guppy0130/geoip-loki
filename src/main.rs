use std::net::IpAddr;
use std::path::PathBuf;
use std::{collections::HashMap, vec};

use chrono::Local;
use clap::Parser;
use maxminddb::geoip2;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, clap::Parser)]
#[command(author, version, about, long_about = None)]
struct Opts {
    /// IP address to report
    ip_address: IpAddr,
    /// Path to GeoLite2-City.mmdb
    #[arg(short, long)]
    database: PathBuf,
    /// Loki endpoint
    #[arg(short, long)]
    loki_url: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct LokiStream {
    /// k/v label pairs
    stream: HashMap<String, String>,
    /// log entries (timestamp, log line)
    // suspect this may need a custom deserializer
    values: Vec<(String, String)>,
}

/// https://grafana.com/docs/loki/latest/api/#push-log-entries-to-loki
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct LokiStreams {
    streams: Vec<LokiStream>,
}

fn main() {
    let options = Opts::parse();
    let timestamp = Local::now().timestamp_nanos();
    let labels: HashMap<String, String> = HashMap::from_iter([
        ("application".to_owned(), env!("CARGO_PKG_NAME").to_owned()),
        ("level".to_owned(), "info".to_owned()),
    ]);
    let mut body: Map<String, Value> = Map::new();

    let reader = maxminddb::Reader::open_mmap(options.database).unwrap();
    let lookup_data: geoip2::City = reader.lookup(options.ip_address).unwrap();

    if let Some(location) = lookup_data.location {
        if location.latitude.is_some() && location.longitude.is_some() {
            body.insert("lat".to_string(), location.latitude.unwrap().into());
            body.insert("lon".to_string(), location.longitude.unwrap().into());
        }
        if let Some(accuracy) = location.accuracy_radius {
            // https://blog.maxmind.com/2022/06/using-maxminds-accuracy-radius/
            // units are kilometers
            body.insert("accuracy_radius_km".to_string(), accuracy.into());
        }
    }

    if let Some(city) = lookup_data.city {
        if let Some(city_name) = city.names.unwrap_or_default().get("en") {
            body.insert("city".to_string(), Value::String(city_name.to_string()));
        }
    }

    if let Some(continent) = lookup_data.continent {
        if let Some(continent_name) = continent.names.unwrap_or_default().get("en") {
            body.insert(
                "continent".to_string(),
                Value::String(continent_name.to_string()),
            );
        }
    }

    if let Some(country) = lookup_data.country {
        if let Some(country_name) = country.names.unwrap_or_default().get("en") {
            body.insert(
                "country".to_string(),
                Value::String(country_name.to_string()),
            );
        }
    }

    body.insert(
        "ip_address".to_string(),
        Value::String(options.ip_address.to_string()),
    );

    let entry: LokiStream = LokiStream {
        stream: labels,
        values: vec![(timestamp.to_string(), Value::Object(body).to_string())],
    };
    let streams = LokiStreams {
        streams: vec![entry],
    };

    // ok, this is going to be a one-off request occurring near the end of the process, so it
    // doesn't make sense to fork this out to a different thread, especially if we're not going to
    // do anything concurrently.
    let client = reqwest::blocking::Client::new();
    let res = client.post(options.loki_url).json(&streams).send();

    match res {
        Ok(r) => match r.status() {
            StatusCode::NO_CONTENT => println!("sent ok"),
            _ => panic!("{:?}", r.text()),
        },
        Err(e) => panic!("{:#}", e),
    }
}

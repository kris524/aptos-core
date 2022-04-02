// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
#![recursion_limit = "128"]

use aptos_logger::prelude::*;
use aptos_metrics::json_metrics::get_git_rev;
use reqwest;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env,
    time::{SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

const GA_MEASUREMENT_ID: &str = "GA_MEASUREMENT_ID";
const GA_API_SECRET: &str = "GA_API_SECRET";
const APTOS_TELEMETRY_OPTOUT: &str = "APTOS_TELEMETRY_OPTOUT";

// By default, send telemetry data to Aptos Labs
// This will help with improving the Aptos ecosystem
const APTOS_GA_MEASUREMENT_ID: &str = "G-ZX4L6WPCFZ";
const APTOS_GA_API_SECRET: &str = "ArtslKPTTjeiMi1n-IR39g";

#[derive(Debug, Serialize, Deserialize)]
struct MetricsDump {
    client_id: String,
    user_id: String,
    timestamp_micros: String,
    events: Vec<MetricsEvent>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MetricsEvent {
    name: String,
    params: HashMap<String, String>,
}

#[derive(Deserialize)]
struct Ip {
    origin: String,
}

pub fn is_optout() -> bool {
    env::var(APTOS_TELEMETRY_OPTOUT).is_ok()
}

pub async fn send_data(event_name: String, event_params: HashMap<String, String>) {
    if is_optout() {
        debug!("Error sending data: optout of Aptos telemetry");
        return;
    }

    // parse environment variables
    let api_secret;
    let measurement_id;
    match env::var(GA_API_SECRET) {
        Ok(val) => api_secret = val,
        Err(_) => api_secret = APTOS_GA_API_SECRET.to_string(),
    };
    match env::var(GA_MEASUREMENT_ID) {
        Ok(val) => measurement_id = val,
        Err(_) => measurement_id = APTOS_GA_MEASUREMENT_ID.to_string(),
    };

    // dump event params in a new hashmap with some default params to include
    let mut new_event_params: HashMap<String, String> = HashMap::new();
    // attempt to get IP address
    let resp = reqwest::get("http://httpbin.org/ip").await;
    let ip_origin = match resp {
        Ok(json) => match json.json::<Ip>().await {
            Ok(ip) => ip.origin,
            Err(_) => String::new(),
        },
        Err(_) => String::new(),
    };
    new_event_params.insert("IP_ADDRESS".to_string(), ip_origin);
    new_event_params.insert("GIT_REV".to_string(), get_git_rev());
    for (k, v) in event_params {
        new_event_params.insert(k, v);
    }

    let metrics_event = MetricsEvent {
        name: event_name,
        params: new_event_params,
    };

    let metrics_dump = MetricsDump {
        client_id: Uuid::new_v4().to_string(),
        user_id: Uuid::new_v4().to_string(), // get peer_id for this?
        timestamp_micros: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_micros()
            .to_string(),
        events: vec![metrics_event],
    };

    let client = reqwest::Client::new();
    let res = client
        .post(format!(
            "https://www.google-analytics.com/mp/collect?&measurement_id={}&api_secret={}",
            measurement_id, api_secret
        ))
        .json::<MetricsDump>(&metrics_dump)
        .send()
        .await;
    match res {
        Ok(_) => debug!("Sent telemetry data {:?}", &metrics_dump),
        Err(e) => debug!("{:?}", e),
    }
}

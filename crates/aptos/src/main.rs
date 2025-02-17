// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

//! Aptos is a one stop tool for operations, debugging, and other operations with the blockchain
//!
//! TODO: Examples
//!
#![forbid(unsafe_code)]

use aptos::Tool;
use clap::Parser;
use std::process::exit;

#[tokio::main]
async fn main() {
    // Run the corresponding tools
    let result = Tool::parse().execute().await;

    // At this point, we'll want to print and determine whether to exit for an error code
    match result {
        Ok(inner) => println!("{}", inner),
        Err(inner) => {
            println!("{}", inner);
            exit(1);
        }
    }
}

// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

pub mod common;
pub mod op;

use aptos_telemetry::send_data;

use crate::common::types::{CliResult, Error};
use clap::Parser;
use std::collections::HashMap;
use structopt::StructOpt;

/// CLI tool for interacting with the Aptos blockchain and nodes
///
#[derive(Debug, Parser)]
#[clap(name = "aptos")]
pub enum Tool {
    #[clap(subcommand)]
    Op(op::OpTool),
}

impl Tool {
    pub async fn execute(self) -> CliResult {
        let mut metrics_params: HashMap<String, String> = HashMap::new();
        metrics_params.insert("execute".to_string(), "1".to_string());
        send_data("APTOS_CLI_PUSH_METRICS".to_string(), metrics_params).await;
        match self {
            Tool::Op(op_tool) => op_tool.execute().await,
        }
    }
}

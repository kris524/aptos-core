// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

pub mod common;
pub mod op;

use crate::common::types::{CliResult, Error};
use clap::Parser;

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
        match self {
            Tool::Op(op_tool) => op_tool.execute().await,
        }
    }
}

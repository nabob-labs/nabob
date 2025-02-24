// Copyright (c) Nabob Labs
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    nabob_workspace_svc::WorkspaceCommand::parse()
        .run()
        .await
}

// Copyright (c) Nabob Labs
// SPDX-License-Identifier: Apache-2.0

use crate::common::types::{CliCommand, CliTypedResult};
use nabob_workspace_svc::WorkspaceCommand;
use async_trait::async_trait;

#[async_trait]
impl CliCommand<()> for WorkspaceCommand {
    fn command_name(&self) -> &'static str {
        "Workspace"
    }

    async fn execute(self) -> CliTypedResult<()> {
        self.run().await?;

        Ok(())
    }
}

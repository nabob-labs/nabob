// Copyright © Nabob Labs
// SPDX-License-Identifier: Apache-2.0

use crate::{
    move_tool::{ArgWithType, FunctionArgType},
    CliResult, Tool,
};
use clap::Parser;
use std::str::FromStr;

/// In order to ensure that there aren't duplicate input arguments for untested CLI commands,
/// we call help on every command to ensure it at least runs
#[tokio::test]
async fn ensure_every_command_args_work() {
    assert_cmd_not_panic(&["nabob"]).await;

    assert_cmd_not_panic(&["nabob", "account"]).await;
    assert_cmd_not_panic(&["nabob", "account", "create", "--help"]).await;
    assert_cmd_not_panic(&["nabob", "account", "create-resource-account", "--help"]).await;
    assert_cmd_not_panic(&["nabob", "account", "fund-with-faucet", "--help"]).await;
    assert_cmd_not_panic(&["nabob", "account", "list", "--help"]).await;
    assert_cmd_not_panic(&["nabob", "account", "lookup-address", "--help"]).await;
    assert_cmd_not_panic(&["nabob", "account", "rotate-key", "--help"]).await;
    assert_cmd_not_panic(&["nabob", "account", "transfer", "--help"]).await;

    assert_cmd_not_panic(&["nabob", "config"]).await;
    assert_cmd_not_panic(&["nabob", "config", "generate-shell-completions", "--help"]).await;
    assert_cmd_not_panic(&["nabob", "config", "init", "--help"]).await;
    assert_cmd_not_panic(&["nabob", "config", "set-global-config", "--help"]).await;
    assert_cmd_not_panic(&["nabob", "config", "show-global-config"]).await;
    assert_cmd_not_panic(&["nabob", "config", "show-profiles"]).await;

    assert_cmd_not_panic(&["nabob", "genesis"]).await;
    assert_cmd_not_panic(&["nabob", "genesis", "generate-genesis", "--help"]).await;
    assert_cmd_not_panic(&["nabob", "genesis", "generate-keys", "--help"]).await;
    assert_cmd_not_panic(&["nabob", "genesis", "generate-layout-template", "--help"]).await;
    assert_cmd_not_panic(&["nabob", "genesis", "set-validator-configuration", "--help"]).await;
    assert_cmd_not_panic(&["nabob", "genesis", "setup-git", "--help"]).await;
    assert_cmd_not_panic(&["nabob", "genesis", "generate-admin-write-set", "--help"]).await;

    assert_cmd_not_panic(&["nabob", "governance"]).await;
    assert_cmd_not_panic(&["nabob", "governance", "execute-proposal", "--help"]).await;
    assert_cmd_not_panic(&["nabob", "governance", "generate-upgrade-proposal", "--help"]).await;
    assert_cmd_not_panic(&["nabob", "governance", "propose", "--help"]).await;
    assert_cmd_not_panic(&["nabob", "governance", "vote", "--help"]).await;
    assert_cmd_not_panic(&["nabob", "governance", "delegation_pool", "--help"]).await;
    assert_cmd_not_panic(&["nabob", "governance", "delegation_pool", "vote", "--help"]).await;
    assert_cmd_not_panic(&[
        "nabob",
        "governance",
        "delegation_pool",
        "propose",
        "--help",
    ])
    .await;

    assert_cmd_not_panic(&["nabob", "info"]).await;

    assert_cmd_not_panic(&["nabob", "init", "--help"]).await;

    assert_cmd_not_panic(&["nabob", "key"]).await;
    assert_cmd_not_panic(&["nabob", "key", "generate", "--help"]).await;
    assert_cmd_not_panic(&["nabob", "key", "extract-peer", "--help"]).await;

    assert_cmd_not_panic(&["nabob", "move"]).await;
    assert_cmd_not_panic(&["nabob", "move", "clean", "--help"]).await;
    assert_cmd_not_panic(&["nabob", "move", "compile", "--help"]).await;
    assert_cmd_not_panic(&["nabob", "move", "compile-script", "--help"]).await;
    assert_cmd_not_panic(&["nabob", "move", "decompile", "--help"]).await;
    assert_cmd_not_panic(&["nabob", "move", "disassemble", "--help"]).await;
    assert_cmd_not_panic(&["nabob", "move", "download", "--help"]).await;
    assert_cmd_not_panic(&["nabob", "move", "init", "--help"]).await;
    assert_cmd_not_panic(&["nabob", "move", "list", "--help"]).await;
    assert_cmd_not_panic(&["nabob", "move", "prove", "--help"]).await;
    assert_cmd_not_panic(&["nabob", "move", "publish", "--help"]).await;
    assert_cmd_not_panic(&["nabob", "move", "run", "--help"]).await;
    assert_cmd_not_panic(&["nabob", "move", "run-script", "--help"]).await;
    assert_cmd_not_panic(&["nabob", "move", "test", "--help"]).await;
    assert_cmd_not_panic(&["nabob", "move", "transactional-test", "--help"]).await;
    assert_cmd_not_panic(&["nabob", "move", "view", "--help"]).await;

    assert_cmd_not_panic(&["nabob", "node"]).await;
    assert_cmd_not_panic(&["nabob", "node", "check-network-connectivity", "--help"]).await;
    assert_cmd_not_panic(&["nabob", "node", "get-stake-pool", "--help"]).await;
    assert_cmd_not_panic(&["nabob", "node", "analyze-validator-performance", "--help"]).await;
    assert_cmd_not_panic(&["nabob", "node", "bootstrap-db-from-backup", "--help"]).await;
    assert_cmd_not_panic(&["nabob", "node", "initialize-validator", "--help"]).await;
    assert_cmd_not_panic(&["nabob", "node", "join-validator-set", "--help"]).await;
    assert_cmd_not_panic(&["nabob", "node", "leave-validator-set", "--help"]).await;
    assert_cmd_not_panic(&["nabob", "node", "run-local-testnet", "--help"]).await;
    assert_cmd_not_panic(&["nabob", "node", "show-validator-config", "--help"]).await;
    assert_cmd_not_panic(&["nabob", "node", "show-validator-set", "--help"]).await;
    assert_cmd_not_panic(&["nabob", "node", "show-validator-stake", "--help"]).await;
    assert_cmd_not_panic(&["nabob", "node", "update-consensus-key", "--help"]).await;
    assert_cmd_not_panic(&[
        "nabob",
        "node",
        "update-validator-network-addresses",
        "--help",
    ])
    .await;

    assert_cmd_not_panic(&["nabob", "stake"]).await;
    assert_cmd_not_panic(&["nabob", "stake", "add-stake", "--help"]).await;
    assert_cmd_not_panic(&["nabob", "stake", "increase-lockup", "--help"]).await;
    assert_cmd_not_panic(&["nabob", "stake", "initialize-stake-owner", "--help"]).await;
    assert_cmd_not_panic(&["nabob", "stake", "set-delegated-voter", "--help"]).await;
    assert_cmd_not_panic(&["nabob", "stake", "set-operator", "--help"]).await;
    assert_cmd_not_panic(&["nabob", "stake", "unlock-stake", "--help"]).await;
    assert_cmd_not_panic(&["nabob", "stake", "withdraw-stake", "--help"]).await;
}

/// Ensure we can parse URLs for args
#[tokio::test]
async fn ensure_can_parse_args_with_urls() {
    let result = ArgWithType::from_str("string:https://naboblabs.com").unwrap();
    matches!(result._ty, FunctionArgType::String);
    assert_eq!(
        result.arg,
        bcs::to_bytes(&"https://naboblabs.com".to_string()).unwrap()
    );
}

async fn assert_cmd_not_panic(args: &[&str]) {
    // When a command fails, it will have a panic in it due to an improperly setup command
    // thread 'main' panicked at 'Command propose: Argument names must be unique, but 'assume-yes' is
    // in use by more than one argument or group', ...

    match run_cmd(args).await {
        Ok(inner) => assert!(
            !inner.contains("panic"),
            "Failed to not panic cmd {}: {}",
            args.join(" "),
            inner
        ),
        Err(inner) => assert!(
            !inner.contains("panic"),
            "Failed to not panic cmd {}: {}",
            args.join(" "),
            inner
        ),
    }
}

async fn run_cmd(args: &[&str]) -> CliResult {
    let tool: Tool = Tool::try_parse_from(args).map_err(|msg| msg.to_string())?;
    tool.execute().await
}

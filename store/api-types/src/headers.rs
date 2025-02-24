// Copyright © Nabob Labs
// SPDX-License-Identifier: Apache-2.0

/// Chain ID of the current chain
pub const X_NABOB_CHAIN_ID: &str = "X-Nabob-Chain-Id";
/// Current epoch of the chain
pub const X_NABOB_EPOCH: &str = "X-Nabob-Epoch";
/// Current ledger version of the chain
pub const X_NABOB_LEDGER_VERSION: &str = "X-Nabob-Ledger-Version";
/// Oldest non-pruned ledger version of the chain
pub const X_NABOB_LEDGER_OLDEST_VERSION: &str = "X-Nabob-Ledger-Oldest-Version";
/// Current block height of the chain
pub const X_NABOB_BLOCK_HEIGHT: &str = "X-Nabob-Block-Height";
/// Oldest non-pruned block height of the chain
pub const X_NABOB_OLDEST_BLOCK_HEIGHT: &str = "X-Nabob-Oldest-Block-Height";
/// Current timestamp of the chain
pub const X_NABOB_LEDGER_TIMESTAMP: &str = "X-Nabob-Ledger-TimestampUsec";
/// Cursor used for pagination.
pub const X_NABOB_CURSOR: &str = "X-Nabob-Cursor";
/// The cost of the call in terms of gas. Only applicable to calls that result in
/// function execution in the VM, e.g. view functions, txn simulation.
pub const X_NABOB_GAS_USED: &str = "X-Nabob-Gas-Used";
/// Provided by the client to identify what client it is.
pub const X_NABOB_CLIENT: &str = "x-nabob-client";

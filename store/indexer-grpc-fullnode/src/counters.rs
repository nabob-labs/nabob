// Copyright © Nabob Labs
// SPDX-License-Identifier: Apache-2.0

use nabob_metrics_core::{register_int_counter, register_int_gauge_vec, IntCounter, IntGaugeVec};
use once_cell::sync::Lazy;

/// Number of times the indexer has been unable to fetch a transaction. Ideally zero.
pub static UNABLE_TO_FETCH_TRANSACTION: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "indexer_grpc_unable_to_fetch_transaction_count",
        "Number of times the indexer has been unable to fetch a transaction from storage"
    )
    .unwrap()
});

/// Channel size
pub static CHANNEL_SIZE: Lazy<IntGaugeVec> = Lazy::new(|| {
    register_int_gauge_vec!(
        "indexer_grpc_fullnode_channel_size",
        "Channel size for full node",
        &["step"],
    )
    .unwrap()
});

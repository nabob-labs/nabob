// Copyright © Nabob Labs
// SPDX-License-Identifier: Apache-2.0

use crate::error::QuorumStoreError;
use nabob_consensus_types::{common::Payload, payload_pull_params::PayloadPullParameters};
use nabob_types::validator_txn::ValidatorTransaction;
use nabob_validator_txpool::TransactionFilter;
use futures::future::BoxFuture;

pub mod mixed;
pub mod user;
pub mod validator;

#[async_trait::async_trait]
pub trait PayloadClient: Send + Sync {
    async fn pull_payload(
        &self,
        config: PayloadPullParameters,
        validator_txn_filter: TransactionFilter,
        wait_callback: BoxFuture<'static, ()>,
    ) -> anyhow::Result<(Vec<ValidatorTransaction>, Payload), QuorumStoreError>;
}

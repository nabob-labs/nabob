// Copyright © Nabob Labs
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

//! This module defines error types used by `NabobDB`.
use nabob_types::state_store::errors::StateViewError;
use std::sync::mpsc::RecvError;
use thiserror::Error;

/// This enum defines errors commonly used among `NabobDB` APIs.
#[derive(Debug, Error)]
pub enum NabobDbError {
    /// A requested item is not found.
    #[error("{0} not found.")]
    NotFound(String),
    /// Requested too many items.
    #[error("Too many items requested: at least {0} requested, max is {1}")]
    TooManyRequested(u64, u64),
    #[error("Missing state root node at version {0}, probably pruned.")]
    MissingRootError(u64),
    /// Other non-classified error.
    #[error("NabobDB Other Error: {0}")]
    Other(String),
    #[error("NabobDB RocksDb Error: {0}")]
    RocksDbIncompleteResult(String),
    #[error("NabobDB RocksDB Error: {0}")]
    OtherRocksDbError(String),
    #[error("NabobDB bcs Error: {0}")]
    BcsError(String),
    #[error("NabobDB IO Error: {0}")]
    IoError(String),
    #[error("NabobDB Recv Error: {0}")]
    RecvError(String),
    #[error("NabobDB ParseInt Error: {0}")]
    ParseIntError(String),
}

impl From<anyhow::Error> for NabobDbError {
    fn from(error: anyhow::Error) -> Self {
        Self::Other(format!("{}", error))
    }
}

impl From<bcs::Error> for NabobDbError {
    fn from(error: bcs::Error) -> Self {
        Self::BcsError(format!("{}", error))
    }
}

impl From<RecvError> for NabobDbError {
    fn from(error: RecvError) -> Self {
        Self::RecvError(format!("{}", error))
    }
}

impl From<std::io::Error> for NabobDbError {
    fn from(error: std::io::Error) -> Self {
        Self::IoError(format!("{}", error))
    }
}

impl From<std::num::ParseIntError> for NabobDbError {
    fn from(error: std::num::ParseIntError) -> Self {
        Self::Other(format!("{}", error))
    }
}

impl From<NabobDbError> for StateViewError {
    fn from(error: NabobDbError) -> Self {
        match error {
            NabobDbError::NotFound(msg) => StateViewError::NotFound(msg),
            NabobDbError::Other(msg) => StateViewError::Other(msg),
            _ => StateViewError::Other(format!("{}", error)),
        }
    }
}

impl From<StateViewError> for NabobDbError {
    fn from(error: StateViewError) -> Self {
        match error {
            StateViewError::NotFound(msg) => NabobDbError::NotFound(msg),
            StateViewError::Other(msg) => NabobDbError::Other(msg),
            StateViewError::BcsError(err) => NabobDbError::BcsError(err.to_string()),
        }
    }
}

// Copyright (c) Nabob Labs
// SPDX-License-Identifier: Apache-2.0

//! This is a library that provides functionalities required for running a local Nabob network,
//! use by `nabob-workspace-server`` and the CLI's localnet command.
//!
//! Currently it only contains some utility functions, but more code will be moved here over
//! time.

pub mod docker;
pub mod health_checker;
pub mod indexer_api;
pub mod processors;

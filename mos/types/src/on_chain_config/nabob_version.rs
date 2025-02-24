// Copyright © Nabob Labs
// SPDX-License-Identifier: Apache-2.0

use crate::on_chain_config::OnChainConfig;
use serde::{Deserialize, Serialize};

/// Defines the version of Nabob Validator software.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct NabobVersion {
    pub major: u64,
}

impl OnChainConfig for NabobVersion {
    const MODULE_IDENTIFIER: &'static str = "version";
    const TYPE_IDENTIFIER: &'static str = "Version";
}

// NOTE: version number for release 1.2 Nabob
// Items gated by this version number include:
//  - the EntryFunction payload type
pub const NABOB_VERSION_2: NabobVersion = NabobVersion { major: 2 };

// NOTE: version number for release 1.3 of Nabob
// Items gated by this version number include:
//  - Multi-agent transactions
pub const NABOB_VERSION_3: NabobVersion = NabobVersion { major: 3 };

// NOTE: version number for release 1.4 of Nabob
// Items gated by this version number include:
//  - Conflict-Resistant Sequence Numbers
pub const NABOB_VERSION_4: NabobVersion = NabobVersion { major: 4 };

// Maximum current known version
pub const NABOB_MAX_KNOWN_VERSION: NabobVersion = NABOB_VERSION_4;

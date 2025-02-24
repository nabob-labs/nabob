// Copyright © Nabob Labs
// SPDX-License-Identifier: Apache-2.0

//! Constant values useful for indexing.

use once_cell::sync::Lazy;

/// Type string for NabobCoin.
pub const NABOB_COIN_TYPE_STR: &str = "0x1::nabob_coin::NabobCoin";

pub static BOB_METADATA_ADDRESS_RAW: Lazy<[u8; 32]> = Lazy::new(|| {
    let mut addr = [0u8; 32];
    addr[31] = 10u8;
    addr
});

pub static BOB_METADATA_ADDRESS_HEX: Lazy<String> =
    Lazy::new(|| format!("0x{}", hex::encode(*BOB_METADATA_ADDRESS_RAW)));

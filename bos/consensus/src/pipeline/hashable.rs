// Copyright © Nabob Labs
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use nabob_crypto::HashValue;

pub trait Hashable {
    fn hash(&self) -> HashValue;
}

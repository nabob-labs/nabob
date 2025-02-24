// Copyright © Nabob Labs
// SPDX-License-Identifier: Apache-2.0

use super::*;
use nabob_schemadb::{schema::fuzzing::assert_encode_decode, test_no_panic_decoding};
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_encode_decode(
        state_key in any::<StateKey>(),
    ) {
        assert_encode_decode::<StateKeysSchema>(&state_key, &());
    }
}

test_no_panic_decoding!(StateKeysSchema);

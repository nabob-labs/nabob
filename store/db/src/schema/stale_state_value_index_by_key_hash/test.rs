// Copyright © Nabob Labs
// SPDX-License-Identifier: Apache-2.0

use super::*;
use nabob_schemadb::{schema::fuzzing::assert_encode_decode, test_no_panic_decoding};
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_encode_decode(
        stale_state_value_index_with_hash in any::<StaleStateValueByKeyHashIndex>(),
    ) {
        assert_encode_decode::<StaleStateValueIndexByKeyHashSchema>(&stale_state_value_index_with_hash, &());
    }
}

test_no_panic_decoding!(StaleStateValueIndexByKeyHashSchema);

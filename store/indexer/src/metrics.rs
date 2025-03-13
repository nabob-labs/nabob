// Copyright © Nabob Labs
// SPDX-License-Identifier: Apache-2.0

use nabob_metrics_core::{exponential_buckets, register_histogram_vec, HistogramVec};
use once_cell::sync::Lazy;

pub static TIMER: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "nabob_internal_indexer_timer_seconds",
        "Various timers for performance analysis.",
        &["name"],
        exponential_buckets(/*start=*/ 1e-9, /*factor=*/ 2.0, /*count=*/ 32).unwrap(),
    )
    .unwrap()
});

#![forbid(unsafe_code)]

// Copyright © Nabob Labs
// SPDX-License-Identifier: Apache-2.0

use nabob_telemetry_svc::NabobTelemetryServiceArgs;
use clap::Parser;

#[tokio::main]
async fn main() {
    nabob_logger::Logger::new().init();
    NabobTelemetryServiceArgs::parse().run().await;
}

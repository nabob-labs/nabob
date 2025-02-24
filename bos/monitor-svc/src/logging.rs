// Copyright © Nabob Labs
// SPDX-License-Identifier: Apache-2.0

use crate::Error;
use nabob_logger::Schema;
use nabob_monitor_types::request::PeerMonitoringServiceRequest;
use serde::Serialize;

#[derive(Schema)]
pub struct LogSchema<'a> {
    name: LogEntry,
    error: Option<&'a Error>,
    message: Option<&'a str>,
    response: Option<&'a str>,
    request: Option<&'a PeerMonitoringServiceRequest>,
}

impl<'a> LogSchema<'a> {
    pub fn new(name: LogEntry) -> Self {
        Self {
            name,
            error: None,
            message: None,
            response: None,
            request: None,
        }
    }
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LogEntry {
    PeerMonitoringServiceError,
    ReceivedPeerMonitoringRequest,
    SentPeerMonitoringResponse,
}

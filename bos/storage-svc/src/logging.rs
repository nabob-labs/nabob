// Copyright © Nabob Labs
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::Error;
use nabob_config::network_id::PeerNetworkId;
use nabob_logger::Schema;
use nabob_storage_types::requests::StorageServiceRequest;
use serde::Serialize;

#[derive(Schema)]
pub struct LogSchema<'a> {
    name: LogEntry,
    error: Option<&'a Error>,
    message: Option<&'a str>,
    optimistic_fetch_related: Option<bool>,
    peer_network_id: Option<&'a PeerNetworkId>,
    response: Option<&'a str>,
    request: Option<&'a StorageServiceRequest>,
}

impl<'a> LogSchema<'a> {
    pub fn new(name: LogEntry) -> Self {
        Self {
            name,
            error: None,
            message: None,
            optimistic_fetch_related: None,
            peer_network_id: None,
            response: None,
            request: None,
        }
    }
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LogEntry {
    OptimisticFetchRefresh,
    OptimisticFetchRequest,
    OptimisticFetchResponse,
    ReceivedCacheUpdateNotification,
    ReceivedCommitNotification,
    ReceivedStorageRequest,
    RequestModeratorIgnoredPeer,
    RequestModeratorRefresh,
    SentStorageResponse,
    StorageServiceError,
    StorageSummaryRefresh,
    SubscriptionRefresh,
    SubscriptionRequest,
    SubscriptionResponse,
}

// Copyright © Nabob Labs
// SPDX-License-Identifier: Apache-2.0

use super::{DefaultProcessingResult, ProcessorName, ProcessorTrait};
use crate::{
    db::{
        common::models::{
            object_models::v2_object_utils::{
                ObjectAggregatedData, ObjectAggregatedDataMapping, ObjectWithMetadata,
            },
            token_v2_models::{
                raw_token_claims::{
                    CurrentTokenPendingClaimConvertible, RawCurrentTokenPendingClaim,
                    TokenV1Claimed,
                },
                raw_v1_token_royalty::{
                    CurrentTokenRoyaltyV1Convertible, RawCurrentTokenRoyaltyV1,
                },
                raw_v2_token_activities::{RawTokenActivityV2, TokenActivityV2Convertible},
                raw_v2_token_datas::{
                    CurrentTokenDataV2Convertible, RawCurrentTokenDataV2, RawTokenDataV2,
                    TokenDataV2Convertible,
                },
                raw_v2_token_metadata::{
                    CurrentTokenV2MetadataConvertible, RawCurrentTokenV2Metadata,
                },
                raw_v2_token_ownerships::{
                    CurrentTokenOwnershipV2Convertible, CurrentTokenOwnershipV2PK, NFTOwnershipV2,
                    RawCurrentTokenOwnershipV2, RawTokenOwnershipV2, TokenOwnershipV2Convertible,
                },
                v2_token_utils::{
                    Burn, BurnEvent, Mint, MintEvent, TokenV2Burned, TokenV2Minted, TransferEvent,
                },
            },
        },
        postgres::models::{
            fungible_asset_models::v2_fungible_asset_utils::FungibleAssetMetadata,
            resources::{FromWriteResource, V2TokenResource},
            token_models::{
                token_claims::CurrentTokenPendingClaim,
                tokens::{CurrentTokenPendingClaimPK, TableHandleToOwner, TableMetadataForToken},
            },
            token_v2_models::{
                v1_token_royalty::CurrentTokenRoyaltyV1,
                v2_collections::{CollectionV2, CurrentCollectionV2, CurrentCollectionV2PK},
                v2_token_activities::TokenActivityV2,
                v2_token_datas::{CurrentTokenDataV2, CurrentTokenDataV2PK, TokenDataV2},
                v2_token_metadata::{CurrentTokenV2Metadata, CurrentTokenV2MetadataPK},
                v2_token_ownerships::{CurrentTokenOwnershipV2, TokenOwnershipV2},
            },
        },
    },
    gap_detectors::ProcessingResult,
    schema,
    utils::{
        counters::PROCESSOR_UNKNOWN_TYPE_COUNT,
        database::{execute_in_chunks, get_config_table_chunk_size, ArcDbPool, DbContext},
        table_flags::TableFlags,
        util::{get_entry_function_from_user_request, parse_timestamp, standardize_address},
    },
    IndexerGrpcProcessorConfig,
};
use ahash::{AHashMap, AHashSet};
use anyhow::bail;
use nabob_protos::transaction::v1::{transaction::TxnData, write_set_change::Change, Transaction};
use async_trait::async_trait;
use diesel::{
    pg::{upsert::excluded, Pg},
    query_builder::QueryFragment,
    ExpressionMethods,
};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use tracing::error;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TokenV2ProcessorConfig {
    #[serde(default = "IndexerGrpcProcessorConfig::default_query_retries")]
    pub query_retries: u32,
    #[serde(default = "IndexerGrpcProcessorConfig::default_query_retry_delay_ms")]
    pub query_retry_delay_ms: u64,
}

pub struct TokenV2Processor {
    connection_pool: ArcDbPool,
    config: TokenV2ProcessorConfig,
    per_table_chunk_sizes: AHashMap<String, usize>,
    deprecated_tables: TableFlags,
}

impl TokenV2Processor {
    pub fn new(
        connection_pool: ArcDbPool,
        config: TokenV2ProcessorConfig,
        per_table_chunk_sizes: AHashMap<String, usize>,
        deprecated_tables: TableFlags,
    ) -> Self {
        Self {
            connection_pool,
            config,
            per_table_chunk_sizes,
            deprecated_tables,
        }
    }
}

impl Debug for TokenV2Processor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = &self.connection_pool.state();
        write!(
            f,
            "TokenV2TransactionProcessor {{ connections: {:?}  idle_connections: {:?} }}",
            state.connections, state.idle_connections
        )
    }
}

#[allow(clippy::too_many_arguments)]
async fn insert_to_db(
    conn: ArcDbPool,
    name: &'static str,
    start_version: u64,
    end_version: u64,
    collections_v2: &[CollectionV2],
    token_datas_v2: &[TokenDataV2],
    token_ownerships_v2: &[TokenOwnershipV2],
    current_collections_v2: &[CurrentCollectionV2],
    (current_token_datas_v2, current_deleted_token_datas_v2): (
        &[CurrentTokenDataV2],
        &[CurrentTokenDataV2],
    ),
    (current_token_ownerships_v2, current_deleted_token_ownerships_v2): (
        &[CurrentTokenOwnershipV2],
        &[CurrentTokenOwnershipV2],
    ),
    token_activities_v2: &[TokenActivityV2],
    current_token_v2_metadata: &[CurrentTokenV2Metadata],
    current_token_royalties_v1: &[CurrentTokenRoyaltyV1],
    current_token_claims: &[CurrentTokenPendingClaim],
    per_table_chunk_sizes: &AHashMap<String, usize>,
) -> Result<(), diesel::result::Error> {
    tracing::trace!(
        name = name,
        start_version = start_version,
        end_version = end_version,
        "Inserting to db",
    );

    let coll_v2 = execute_in_chunks(
        conn.clone(),
        insert_collections_v2_query,
        collections_v2,
        get_config_table_chunk_size::<CollectionV2>("collections_v2", per_table_chunk_sizes),
    );
    let td_v2 = execute_in_chunks(
        conn.clone(),
        insert_token_datas_v2_query,
        token_datas_v2,
        get_config_table_chunk_size::<TokenDataV2>("token_datas_v2", per_table_chunk_sizes),
    );
    let to_v2 = execute_in_chunks(
        conn.clone(),
        insert_token_ownerships_v2_query,
        token_ownerships_v2,
        get_config_table_chunk_size::<TokenOwnershipV2>(
            "token_ownerships_v2",
            per_table_chunk_sizes,
        ),
    );
    let cc_v2 = execute_in_chunks(
        conn.clone(),
        insert_current_collections_v2_query,
        current_collections_v2,
        get_config_table_chunk_size::<CurrentCollectionV2>(
            "current_collections_v2",
            per_table_chunk_sizes,
        ),
    );
    let ctd_v2 = execute_in_chunks(
        conn.clone(),
        insert_current_token_datas_v2_query,
        current_token_datas_v2,
        get_config_table_chunk_size::<CurrentTokenDataV2>(
            "current_token_datas_v2",
            per_table_chunk_sizes,
        ),
    );
    let cdtd_v2 = execute_in_chunks(
        conn.clone(),
        insert_current_deleted_token_datas_v2_query,
        current_deleted_token_datas_v2,
        get_config_table_chunk_size::<CurrentTokenDataV2>(
            "current_token_datas_v2",
            per_table_chunk_sizes,
        ),
    );
    let cto_v2 = execute_in_chunks(
        conn.clone(),
        insert_current_token_ownerships_v2_query,
        current_token_ownerships_v2,
        get_config_table_chunk_size::<CurrentTokenOwnershipV2>(
            "current_token_ownerships_v2",
            per_table_chunk_sizes,
        ),
    );
    let cdto_v2 = execute_in_chunks(
        conn.clone(),
        insert_current_deleted_token_ownerships_v2_query,
        current_deleted_token_ownerships_v2,
        get_config_table_chunk_size::<CurrentTokenOwnershipV2>(
            "current_token_ownerships_v2",
            per_table_chunk_sizes,
        ),
    );
    let ta_v2 = execute_in_chunks(
        conn.clone(),
        insert_token_activities_v2_query,
        token_activities_v2,
        get_config_table_chunk_size::<TokenActivityV2>(
            "token_activities_v2",
            per_table_chunk_sizes,
        ),
    );
    let ct_v2 = execute_in_chunks(
        conn.clone(),
        insert_current_token_v2_metadatas_query,
        current_token_v2_metadata,
        get_config_table_chunk_size::<CurrentTokenV2Metadata>(
            "current_token_v2_metadata",
            per_table_chunk_sizes,
        ),
    );
    let ctr_v1 = execute_in_chunks(
        conn.clone(),
        insert_current_token_royalties_v1_query,
        current_token_royalties_v1,
        get_config_table_chunk_size::<CurrentTokenRoyaltyV1>(
            "current_token_royalty_v1",
            per_table_chunk_sizes,
        ),
    );
    let ctc_v1 = execute_in_chunks(
        conn,
        insert_current_token_claims_query,
        current_token_claims,
        get_config_table_chunk_size::<CurrentTokenPendingClaim>(
            "current_token_pending_claims",
            per_table_chunk_sizes,
        ),
    );

    let (
        coll_v2_res,
        td_v2_res,
        to_v2_res,
        cc_v2_res,
        ctd_v2_res,
        cdtd_v2_res,
        cto_v2_res,
        cdto_v2_res,
        ta_v2_res,
        ct_v2_res,
        ctr_v1_res,
        ctc_v1_res,
    ) = tokio::join!(
        coll_v2, td_v2, to_v2, cc_v2, ctd_v2, cdtd_v2, cto_v2, cdto_v2, ta_v2, ct_v2, ctr_v1,
        ctc_v1
    );

    for res in [
        coll_v2_res,
        td_v2_res,
        to_v2_res,
        cc_v2_res,
        ctd_v2_res,
        cdtd_v2_res,
        cto_v2_res,
        cdto_v2_res,
        ta_v2_res,
        ct_v2_res,
        ctr_v1_res,
        ctc_v1_res,
    ] {
        res?;
    }

    Ok(())
}

pub fn insert_collections_v2_query(
    items_to_insert: Vec<CollectionV2>,
) -> (
    impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send,
    Option<&'static str>,
) {
    use schema::collections_v2::dsl::*;
    (
        diesel::insert_into(schema::collections_v2::table)
            .values(items_to_insert)
            .on_conflict((transaction_version, write_set_change_index))
            .do_update()
            .set((
                collection_properties.eq(excluded(collection_properties)),
                inserted_at.eq(excluded(inserted_at)),
            )),
        None,
    )
}

pub fn insert_token_datas_v2_query(
    items_to_insert: Vec<TokenDataV2>,
) -> (
    impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send,
    Option<&'static str>,
) {
    use schema::token_datas_v2::dsl::*;

    (
        diesel::insert_into(schema::token_datas_v2::table)
            .values(items_to_insert)
            .on_conflict((transaction_version, write_set_change_index))
            .do_update()
            .set((
                maximum.eq(excluded(maximum)),
                supply.eq(excluded(supply)),
                is_fungible_v2.eq(excluded(is_fungible_v2)),
                inserted_at.eq(excluded(inserted_at)),
                decimals.eq(excluded(decimals)),
            )),
        None,
    )
}

pub fn insert_token_ownerships_v2_query(
    items_to_insert: Vec<TokenOwnershipV2>,
) -> (
    impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send,
    Option<&'static str>,
) {
    use schema::token_ownerships_v2::dsl::*;

    (
        diesel::insert_into(schema::token_ownerships_v2::table)
            .values(items_to_insert)
            .on_conflict((transaction_version, write_set_change_index))
            .do_update()
            .set((
                is_fungible_v2.eq(excluded(is_fungible_v2)),
                inserted_at.eq(excluded(inserted_at)),
            )),
        None,
    )
}

pub fn insert_current_collections_v2_query(
    items_to_insert: Vec<CurrentCollectionV2>,
) -> (
    impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send,
    Option<&'static str>,
) {
    use schema::current_collections_v2::dsl::*;

    (
        diesel::insert_into(schema::current_collections_v2::table)
            .values(items_to_insert)
            .on_conflict(collection_id)
            .do_update()
            .set((
                creator_address.eq(excluded(creator_address)),
                collection_name.eq(excluded(collection_name)),
                description.eq(excluded(description)),
                uri.eq(excluded(uri)),
                current_supply.eq(excluded(current_supply)),
                max_supply.eq(excluded(max_supply)),
                total_minted_v2.eq(excluded(total_minted_v2)),
                mutable_description.eq(excluded(mutable_description)),
                mutable_uri.eq(excluded(mutable_uri)),
                table_handle_v1.eq(excluded(table_handle_v1)),
                token_standard.eq(excluded(token_standard)),
                last_transaction_version.eq(excluded(last_transaction_version)),
                collection_properties.eq(excluded(collection_properties)),
                last_transaction_timestamp.eq(excluded(last_transaction_timestamp)),
                inserted_at.eq(excluded(inserted_at)),
            )),
        Some(" WHERE current_collections_v2.last_transaction_version <= excluded.last_transaction_version "),
     )
}

pub fn insert_current_token_datas_v2_query(
    items_to_insert: Vec<CurrentTokenDataV2>,
) -> (
    impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send,
    Option<&'static str>,
) {
    use schema::current_token_datas_v2::dsl::*;

    (
        diesel::insert_into(schema::current_token_datas_v2::table)
            .values(items_to_insert)
            .on_conflict(token_data_id)
            .do_update()
            .set((
                collection_id.eq(excluded(collection_id)),
                token_name.eq(excluded(token_name)),
                maximum.eq(excluded(maximum)),
                supply.eq(excluded(supply)),
                largest_property_version_v1.eq(excluded(largest_property_version_v1)),
                token_uri.eq(excluded(token_uri)),
                description.eq(excluded(description)),
                token_properties.eq(excluded(token_properties)),
                token_standard.eq(excluded(token_standard)),
                is_fungible_v2.eq(excluded(is_fungible_v2)),
                last_transaction_version.eq(excluded(last_transaction_version)),
                last_transaction_timestamp.eq(excluded(last_transaction_timestamp)),
                inserted_at.eq(excluded(inserted_at)),
                decimals.eq(excluded(decimals)),
                // Intentionally not including is_deleted because it should always be true in this part
                // and doesn't need to override
            )),
        Some(" WHERE current_token_datas_v2.last_transaction_version <= excluded.last_transaction_version "),
    )
}

pub fn insert_current_deleted_token_datas_v2_query(
    items_to_insert: Vec<CurrentTokenDataV2>,
) -> (
    impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send,
    Option<&'static str>,
) {
    use schema::current_token_datas_v2::dsl::*;

    (
        diesel::insert_into(schema::current_token_datas_v2::table)
            .values(items_to_insert)
            .on_conflict(token_data_id)
            .do_update()
            .set((
                last_transaction_version.eq(excluded(last_transaction_version)),
                last_transaction_timestamp.eq(excluded(last_transaction_timestamp)),
                inserted_at.eq(excluded(inserted_at)),
                is_deleted_v2.eq(excluded(is_deleted_v2)),
            )),
        Some(" WHERE current_token_datas_v2.last_transaction_version <= excluded.last_transaction_version "),
    )
}

pub fn insert_current_token_ownerships_v2_query(
    items_to_insert: Vec<CurrentTokenOwnershipV2>,
) -> (
    impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send,
    Option<&'static str>,
) {
    use schema::current_token_ownerships_v2::dsl::*;

    (
        diesel::insert_into(schema::current_token_ownerships_v2::table)
            .values(items_to_insert)
            .on_conflict((token_data_id, property_version_v1, owner_address, storage_id))
            .do_update()
            .set((
                amount.eq(excluded(amount)),
                table_type_v1.eq(excluded(table_type_v1)),
                token_properties_mutated_v1.eq(excluded(token_properties_mutated_v1)),
                is_soulbound_v2.eq(excluded(is_soulbound_v2)),
                token_standard.eq(excluded(token_standard)),
                is_fungible_v2.eq(excluded(is_fungible_v2)),
                last_transaction_version.eq(excluded(last_transaction_version)),
                last_transaction_timestamp.eq(excluded(last_transaction_timestamp)),
                inserted_at.eq(excluded(inserted_at)),
                non_transferrable_by_owner.eq(excluded(non_transferrable_by_owner)),
            )),
        Some(" WHERE current_token_ownerships_v2.last_transaction_version <= excluded.last_transaction_version "),
    )
}

pub fn insert_current_deleted_token_ownerships_v2_query(
    items_to_insert: Vec<CurrentTokenOwnershipV2>,
) -> (
    impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send,
    Option<&'static str>,
) {
    use schema::current_token_ownerships_v2::dsl::*;

    (
        diesel::insert_into(schema::current_token_ownerships_v2::table)
            .values(items_to_insert)
            .on_conflict((token_data_id, property_version_v1, owner_address, storage_id))
            .do_update()
            .set((
                amount.eq(excluded(amount)),
                last_transaction_version.eq(excluded(last_transaction_version)),
                last_transaction_timestamp.eq(excluded(last_transaction_timestamp)),
                is_fungible_v2.eq(excluded(is_fungible_v2)),
                inserted_at.eq(excluded(inserted_at)),
            )),
        Some(" WHERE current_token_ownerships_v2.last_transaction_version <= excluded.last_transaction_version "),
    )
}

pub fn insert_token_activities_v2_query(
    items_to_insert: Vec<TokenActivityV2>,
) -> (
    impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send,
    Option<&'static str>,
) {
    use schema::token_activities_v2::dsl::*;

    (
        diesel::insert_into(schema::token_activities_v2::table)
            .values(items_to_insert)
            .on_conflict((transaction_version, event_index))
            .do_update()
            .set((
                is_fungible_v2.eq(excluded(is_fungible_v2)),
                inserted_at.eq(excluded(inserted_at)),
            )),
        None,
    )
}

pub fn insert_current_token_v2_metadatas_query(
    items_to_insert: Vec<CurrentTokenV2Metadata>,
) -> (
    impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send,
    Option<&'static str>,
) {
    use schema::current_token_v2_metadata::dsl::*;

    (
        diesel::insert_into(schema::current_token_v2_metadata::table)
            .values(items_to_insert)
            .on_conflict((object_address, resource_type))
            .do_update()
            .set((
                data.eq(excluded(data)),
                state_key_hash.eq(excluded(state_key_hash)),
                last_transaction_version.eq(excluded(last_transaction_version)),
                inserted_at.eq(excluded(inserted_at)),
            )),
        Some(" WHERE current_token_v2_metadata.last_transaction_version <= excluded.last_transaction_version "),
    )
}

pub fn insert_current_token_royalties_v1_query(
    items_to_insert: Vec<CurrentTokenRoyaltyV1>,
) -> (
    impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send,
    Option<&'static str>,
) {
    use schema::current_token_royalty_v1::dsl::*;

    (
        diesel::insert_into(schema::current_token_royalty_v1::table)
            .values(items_to_insert)
            .on_conflict(token_data_id)
            .do_update()
            .set((
                payee_address.eq(excluded(payee_address)),
                royalty_points_numerator.eq(excluded(royalty_points_numerator)),
                royalty_points_denominator.eq(excluded(royalty_points_denominator)),
                last_transaction_version.eq(excluded(last_transaction_version)),
                last_transaction_timestamp.eq(excluded(last_transaction_timestamp)),
            )),
        Some(" WHERE current_token_royalty_v1.last_transaction_version <= excluded.last_transaction_version "),
    )
}

pub fn insert_current_token_claims_query(
    items_to_insert: Vec<CurrentTokenPendingClaim>,
) -> (
    impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send,
    Option<&'static str>,
) {
    use schema::current_token_pending_claims::dsl::*;

    (
        diesel::insert_into(schema::current_token_pending_claims::table)
            .values(items_to_insert)
            .on_conflict((
                token_data_id_hash, property_version, from_address, to_address
            ))
            .do_update()
            .set((
                collection_data_id_hash.eq(excluded(collection_data_id_hash)),
                creator_address.eq(excluded(creator_address)),
                collection_name.eq(excluded(collection_name)),
                name.eq(excluded(name)),
                amount.eq(excluded(amount)),
                table_handle.eq(excluded(table_handle)),
                last_transaction_version.eq(excluded(last_transaction_version)),
                inserted_at.eq(excluded(inserted_at)),
                token_data_id.eq(excluded(token_data_id)),
                collection_id.eq(excluded(collection_id)),
            )),
        Some(" WHERE current_token_pending_claims.last_transaction_version <= excluded.last_transaction_version "),
    )
}

#[async_trait]
impl ProcessorTrait for TokenV2Processor {
    fn name(&self) -> &'static str {
        ProcessorName::TokenV2Processor.into()
    }

    async fn process_transactions(
        &self,
        transactions: Vec<Transaction>,
        start_version: u64,
        end_version: u64,
        _: Option<u64>,
    ) -> anyhow::Result<ProcessingResult> {
        let processing_start = std::time::Instant::now();
        let last_transaction_timestamp = transactions.last().unwrap().timestamp;

        let conn = self.get_conn().await;

        // First get all token related table metadata from the batch of transactions. This is in case
        // an earlier transaction has metadata (in resources) that's missing from a later transaction.
        let table_handle_to_owner =
            TableMetadataForToken::get_table_handle_to_owner_from_transactions(&transactions);

        let db_connection = DbContext {
            conn,
            query_retries: self.config.query_retries,
            query_retry_delay_ms: self.config.query_retry_delay_ms,
        };
        // Token V2 processing which includes token v1
        let (
            mut collections_v2,
            raw_token_datas_v2,
            raw_token_ownerships_v2,
            current_collections_v2,
            raw_current_token_datas_v2,
            raw_current_deleted_token_datas_v2,
            raw_current_token_ownerships_v2,
            raw_current_deleted_token_ownerships_v2,
            raw_token_activities_v2,
            raw_current_token_v2_metadata,
            raw_current_token_royalties_v1,
            raw_current_token_claims,
        ) = parse_v2_token(
            &transactions,
            &table_handle_to_owner,
            &mut Some(db_connection),
        )
        .await;

        let postgres_current_token_claims: Vec<CurrentTokenPendingClaim> = raw_current_token_claims
            .into_iter()
            .map(CurrentTokenPendingClaim::from_raw)
            .collect();

        let postgres_current_token_royalties_v1: Vec<CurrentTokenRoyaltyV1> =
            raw_current_token_royalties_v1
                .into_iter()
                .map(CurrentTokenRoyaltyV1::from_raw)
                .collect();

        let mut postgres_current_token_v2_metadata: Vec<CurrentTokenV2Metadata> =
            raw_current_token_v2_metadata
                .into_iter()
                .map(CurrentTokenV2Metadata::from_raw)
                .collect();

        let postgres_token_activities_v2: Vec<TokenActivityV2> = raw_token_activities_v2
            .into_iter()
            .map(TokenActivityV2::from_raw)
            .collect();

        let mut postgres_token_datas_v2: Vec<TokenDataV2> = raw_token_datas_v2
            .into_iter()
            .map(TokenDataV2::from_raw)
            .collect();

        let postgres_current_token_datas_v2: Vec<CurrentTokenDataV2> = raw_current_token_datas_v2
            .into_iter()
            .map(CurrentTokenDataV2::from_raw)
            .collect();

        let postgres_current_deleted_token_datas_v2: Vec<CurrentTokenDataV2> =
            raw_current_deleted_token_datas_v2
                .into_iter()
                .map(CurrentTokenDataV2::from_raw)
                .collect();

        let mut postgres_token_ownerships_v2: Vec<TokenOwnershipV2> = raw_token_ownerships_v2
            .into_iter()
            .map(TokenOwnershipV2::from_raw)
            .collect();

        let postgres_current_token_ownerships_v2: Vec<CurrentTokenOwnershipV2> =
            raw_current_token_ownerships_v2
                .into_iter()
                .map(CurrentTokenOwnershipV2::from_raw)
                .collect();

        let postgres_current_deleted_token_ownerships_v2: Vec<CurrentTokenOwnershipV2> =
            raw_current_deleted_token_ownerships_v2
                .into_iter()
                .map(CurrentTokenOwnershipV2::from_raw)
                .collect();

        let processing_duration_in_secs = processing_start.elapsed().as_secs_f64();
        let db_insertion_start = std::time::Instant::now();

        if self
            .deprecated_tables
            .contains(TableFlags::TOKEN_OWNERSHIPS_V2)
        {
            postgres_token_ownerships_v2.clear();
        }
        if self.deprecated_tables.contains(TableFlags::TOKEN_DATAS_V2) {
            postgres_token_datas_v2.clear();
        }
        if self.deprecated_tables.contains(TableFlags::COLLECTIONS_V2) {
            collections_v2.clear();
        }
        if self
            .deprecated_tables
            .contains(TableFlags::CURRENT_TOKEN_V2_METADATA)
        {
            postgres_current_token_v2_metadata.clear();
        }

        let tx_result = insert_to_db(
            self.get_pool(),
            self.name(),
            start_version,
            end_version,
            &collections_v2,
            &postgres_token_datas_v2,
            &postgres_token_ownerships_v2,
            &current_collections_v2,
            (
                &postgres_current_token_datas_v2,
                &postgres_current_deleted_token_datas_v2,
            ),
            (
                &postgres_current_token_ownerships_v2,
                &postgres_current_deleted_token_ownerships_v2,
            ),
            &postgres_token_activities_v2,
            &postgres_current_token_v2_metadata,
            &postgres_current_token_royalties_v1,
            &postgres_current_token_claims,
            &self.per_table_chunk_sizes,
        )
        .await;

        let db_insertion_duration_in_secs = db_insertion_start.elapsed().as_secs_f64();
        match tx_result {
            Ok(_) => Ok(ProcessingResult::DefaultProcessingResult(
                DefaultProcessingResult {
                    start_version,
                    end_version,
                    processing_duration_in_secs,
                    db_insertion_duration_in_secs,
                    last_transaction_timestamp,
                },
            )),
            Err(e) => {
                error!(
                    start_version = start_version,
                    end_version = end_version,
                    processor_name = self.name(),
                    error = ?e,
                    "[Parser] Error inserting transactions to db",
                );
                bail!(e)
            },
        }
    }

    fn connection_pool(&self) -> &ArcDbPool {
        &self.connection_pool
    }
}

pub async fn parse_v2_token_for_parquet(
    transactions: &[Transaction],
    table_handle_to_owner: &TableHandleToOwner,
) -> (
    Vec<CollectionV2>,
    Vec<RawTokenDataV2>,
    Vec<RawTokenOwnershipV2>,
    Vec<CurrentCollectionV2>,
    Vec<RawCurrentTokenDataV2>,
    Vec<RawCurrentTokenDataV2>,
    Vec<RawCurrentTokenOwnershipV2>,
    Vec<RawCurrentTokenOwnershipV2>, // deleted token ownerships
    Vec<RawTokenActivityV2>,
    Vec<RawCurrentTokenV2Metadata>,
    Vec<RawCurrentTokenRoyaltyV1>,
    Vec<RawCurrentTokenPendingClaim>,
) {
    parse_v2_token(transactions, table_handle_to_owner, &mut None).await
}

pub async fn parse_v2_token(
    transactions: &[Transaction],
    table_handle_to_owner: &TableHandleToOwner,
    db_context: &mut Option<DbContext<'_>>,
) -> (
    Vec<CollectionV2>,
    Vec<RawTokenDataV2>,
    Vec<RawTokenOwnershipV2>,
    Vec<CurrentCollectionV2>,
    Vec<RawCurrentTokenDataV2>,
    Vec<RawCurrentTokenDataV2>,
    Vec<RawCurrentTokenOwnershipV2>,
    Vec<RawCurrentTokenOwnershipV2>, // deleted token ownerships
    Vec<RawTokenActivityV2>,
    Vec<RawCurrentTokenV2Metadata>,
    Vec<RawCurrentTokenRoyaltyV1>,
    Vec<RawCurrentTokenPendingClaim>,
) {
    // Token V2 and V1 combined
    let mut collections_v2 = vec![];
    let mut token_datas_v2 = vec![];
    let mut token_ownerships_v2 = vec![];
    let mut token_activities_v2 = vec![];

    let mut current_collections_v2: AHashMap<CurrentCollectionV2PK, CurrentCollectionV2> =
        AHashMap::new();
    let mut current_token_datas_v2: AHashMap<CurrentTokenDataV2PK, RawCurrentTokenDataV2> =
        AHashMap::new();
    let mut current_deleted_token_datas_v2: AHashMap<CurrentTokenDataV2PK, RawCurrentTokenDataV2> =
        AHashMap::new();
    let mut current_token_ownerships_v2: AHashMap<
        CurrentTokenOwnershipV2PK,
        RawCurrentTokenOwnershipV2,
    > = AHashMap::new();
    let mut current_deleted_token_ownerships_v2 = AHashMap::new();
    // Optimization to track prior ownership in case a token gets burned so we can lookup the ownership
    let mut prior_nft_ownership: AHashMap<String, NFTOwnershipV2> = AHashMap::new();
    // Get Metadata for token v2 by object
    // We want to persist this through the entire batch so that even if a token is burned,
    // we can still get the object core metadata for it
    let mut token_v2_metadata_helper: ObjectAggregatedDataMapping = AHashMap::new();
    // Basically token properties
    let mut current_token_v2_metadata: AHashMap<
        CurrentTokenV2MetadataPK,
        RawCurrentTokenV2Metadata,
    > = AHashMap::new();
    let mut current_token_royalties_v1: AHashMap<CurrentTokenDataV2PK, RawCurrentTokenRoyaltyV1> =
        AHashMap::new();
    // migrating this from v1 token model as we don't have any replacement table for this
    let mut all_current_token_claims: AHashMap<
        CurrentTokenPendingClaimPK,
        RawCurrentTokenPendingClaim,
    > = AHashMap::new();

    // Code above is inefficient (multiple passthroughs) so I'm approaching TokenV2 with a cleaner code structure
    for txn in transactions {
        let txn_version = txn.version;
        let txn_data = match txn.txn_data.as_ref() {
            Some(data) => data,
            None => {
                PROCESSOR_UNKNOWN_TYPE_COUNT
                    .with_label_values(&["TokenV2Processor"])
                    .inc();
                tracing::warn!(
                    transaction_version = txn_version,
                    "Transaction data doesn't exist"
                );
                continue;
            },
        };
        let txn_version = txn.version as i64;
        let txn_timestamp = parse_timestamp(txn.timestamp.as_ref().unwrap(), txn_version);
        let transaction_info = txn.info.as_ref().expect("Transaction info doesn't exist!");

        if let TxnData::User(user_txn) = txn_data {
            let user_request = user_txn
                .request
                .as_ref()
                .expect("Sends is not present in user txn");
            let entry_function_id_str = get_entry_function_from_user_request(user_request);

            // Get burn events for token v2 by object
            let mut tokens_burned: TokenV2Burned = AHashMap::new();

            // Get mint events for token v2 by object
            let mut tokens_minted: TokenV2Minted = AHashSet::new();

            // Get claim events for token v1 by table handle
            let mut tokens_claimed: TokenV1Claimed = AHashMap::new();

            // Loop 1: Need to do a first pass to get all the object addresses and insert them into the helper
            for wsc in transaction_info.changes.iter() {
                if let Change::WriteResource(wr) = wsc.change.as_ref().unwrap() {
                    if let Some(object) = ObjectWithMetadata::from_write_resource(wr).unwrap() {
                        token_v2_metadata_helper.insert(
                            standardize_address(&wr.address.to_string()),
                            ObjectAggregatedData {
                                object,
                                ..ObjectAggregatedData::default()
                            },
                        );
                    }
                }
            }

            // Loop 2: Get the metdata relevant to parse v1 and v2 tokens
            // Need to do a second pass to get all the structs related to the object
            for wsc in transaction_info.changes.iter() {
                if let Change::WriteResource(wr) = wsc.change.as_ref().unwrap() {
                    let address = standardize_address(&wr.address.to_string());
                    if let Some(aggregated_data) = token_v2_metadata_helper.get_mut(&address) {
                        if let Some(v2_token_resource) =
                            V2TokenResource::from_write_resource(wr).unwrap()
                        {
                            match v2_token_resource {
                                V2TokenResource::FixedSupply(fixed_supply) => {
                                    aggregated_data.fixed_supply = Some(fixed_supply);
                                },
                                V2TokenResource::UnlimitedSupply(unlimited_supply) => {
                                    aggregated_data.unlimited_supply = Some(unlimited_supply);
                                },
                                V2TokenResource::NabobCollection(nabob_collection) => {
                                    aggregated_data.nabob_collection = Some(nabob_collection);
                                },
                                V2TokenResource::PropertyMapModel(property_map) => {
                                    aggregated_data.property_map = Some(property_map);
                                },
                                V2TokenResource::ConcurrentSupply(concurrent_supply) => {
                                    aggregated_data.concurrent_supply = Some(concurrent_supply);
                                },
                                V2TokenResource::TokenV2(token) => {
                                    aggregated_data.token = Some(token);
                                },
                                V2TokenResource::TokenIdentifiers(token_identifier) => {
                                    aggregated_data.token_identifier = Some(token_identifier);
                                },
                                V2TokenResource::Untransferable(untransferable) => {
                                    aggregated_data.untransferable = Some(untransferable);
                                },
                                _ => {},
                            }
                        }
                        if let Some(fungible_asset_metadata) =
                            FungibleAssetMetadata::from_write_resource(wr).unwrap()
                        {
                            aggregated_data.fungible_asset_metadata = Some(fungible_asset_metadata);
                        }
                    }
                }
            }

            // Loop 3: Pass through events to get the burn events and token activities v2
            // This needs to be here because we need the metadata parsed in loop 2 for token activities
            // and burn / transfer events need to come before the next loop
            // Also parses token v1 claim events, which will be used in Loop 4 to build the claims table
            for (index, event) in user_txn.events.iter().enumerate() {
                if let Some(burn_event) = Burn::from_event(event, txn_version).unwrap() {
                    tokens_burned.insert(burn_event.get_token_address(), burn_event.clone());
                } else if let Some(mint_event) = Mint::from_event(event, txn_version).unwrap() {
                    tokens_minted.insert(mint_event.get_token_address());
                } else if let Some(old_burn_event) =
                    BurnEvent::from_event(event, txn_version).unwrap()
                {
                    let burn_event = Burn::new(
                        standardize_address(event.key.as_ref().unwrap().account_address.as_str()),
                        old_burn_event.index.clone(),
                        old_burn_event.get_token_address(),
                        "".to_string(),
                    );
                    tokens_burned.insert(burn_event.get_token_address(), burn_event);
                } else if let Some(mint_event) = MintEvent::from_event(event, txn_version).unwrap()
                {
                    tokens_minted.insert(mint_event.get_token_address());
                } else if let Some(transfer_events) =
                    TransferEvent::from_event(event, txn_version).unwrap()
                {
                    if let Some(aggregated_data) =
                        token_v2_metadata_helper.get_mut(&transfer_events.get_object_address())
                    {
                        // we don't want index to be 0 otherwise we might have collision with write set change index
                        // note that these will be multiplied by -1 so that it doesn't conflict with wsc index
                        let index = if index == 0 {
                            user_txn.events.len()
                        } else {
                            index
                        };
                        aggregated_data
                            .transfer_events
                            .push((index as i64, transfer_events));
                    }
                }
                // handling all the token v1 events
                if let Some(event) = RawTokenActivityV2::get_v1_from_parsed_event(
                    event,
                    txn_version,
                    txn_timestamp,
                    index as i64,
                    &entry_function_id_str,
                    &mut tokens_claimed,
                )
                .unwrap()
                {
                    token_activities_v2.push(event);
                }
                // handling all the token v2 events
                if let Some(event) = RawTokenActivityV2::get_nft_v2_from_parsed_event(
                    event,
                    txn_version,
                    txn_timestamp,
                    index as i64,
                    &entry_function_id_str,
                    &token_v2_metadata_helper,
                )
                .await
                .unwrap()
                {
                    token_activities_v2.push(event);
                }
            }

            // Loop 4: Pass through the changes for collection, token data, token ownership, and token royalties
            for (index, wsc) in transaction_info.changes.iter().enumerate() {
                let wsc_index = index as i64;
                match wsc.change.as_ref().unwrap() {
                    Change::WriteTableItem(table_item) => {
                        // TODO: revisit when we migrate collection_v2 for parquet
                        // for not it will be only handled for postgres
                        if let Some(ref mut db_context) = db_context {
                            if let Some((collection, current_collection)) =
                                CollectionV2::get_v1_from_write_table_item(
                                    table_item,
                                    txn_version,
                                    wsc_index,
                                    txn_timestamp,
                                    table_handle_to_owner,
                                    &mut db_context.conn,
                                    db_context.query_retries,
                                    db_context.query_retry_delay_ms,
                                )
                                .await
                                .unwrap()
                            {
                                collections_v2.push(collection);
                                current_collections_v2.insert(
                                    current_collection.collection_id.clone(),
                                    current_collection,
                                );
                            }
                        }

                        if let Some((token_data, current_token_data)) =
                            RawTokenDataV2::get_v1_from_write_table_item(
                                table_item,
                                txn_version,
                                wsc_index,
                                txn_timestamp,
                            )
                            .unwrap()
                        {
                            token_datas_v2.push(token_data);
                            current_token_datas_v2.insert(
                                current_token_data.token_data_id.clone(),
                                current_token_data,
                            );
                        }
                        if let Some(current_token_royalty) =
                            RawCurrentTokenRoyaltyV1::get_v1_from_write_table_item(
                                table_item,
                                txn_version,
                                txn_timestamp,
                            )
                            .unwrap()
                        {
                            current_token_royalties_v1.insert(
                                current_token_royalty.token_data_id.clone(),
                                current_token_royalty,
                            );
                        }
                        if let Some((token_ownership, current_token_ownership)) =
                            RawTokenOwnershipV2::get_v1_from_write_table_item(
                                table_item,
                                txn_version,
                                wsc_index,
                                txn_timestamp,
                                table_handle_to_owner,
                            )
                            .unwrap()
                        {
                            token_ownerships_v2.push(token_ownership);
                            if let Some(cto) = current_token_ownership {
                                prior_nft_ownership.insert(
                                    cto.token_data_id.clone(),
                                    NFTOwnershipV2 {
                                        token_data_id: cto.token_data_id.clone(),
                                        owner_address: cto.owner_address.clone(),
                                        is_soulbound: cto.is_soulbound_v2,
                                    },
                                );
                                current_token_ownerships_v2.insert(
                                    (
                                        cto.token_data_id.clone(),
                                        cto.property_version_v1.clone(),
                                        cto.owner_address.clone(),
                                        cto.storage_id.clone(),
                                    ),
                                    cto,
                                );
                            }
                        }
                        if let Some(current_token_token_claim) =
                            RawCurrentTokenPendingClaim::from_write_table_item(
                                table_item,
                                txn_version,
                                txn_timestamp,
                                table_handle_to_owner,
                            )
                            .unwrap()
                        {
                            all_current_token_claims.insert(
                                (
                                    current_token_token_claim.token_data_id_hash.clone(),
                                    current_token_token_claim.property_version.clone(),
                                    current_token_token_claim.from_address.clone(),
                                    current_token_token_claim.to_address.clone(),
                                ),
                                current_token_token_claim,
                            );
                        }
                    },
                    Change::DeleteTableItem(table_item) => {
                        if let Some((token_ownership, current_token_ownership)) =
                            RawTokenOwnershipV2::get_v1_from_delete_table_item(
                                table_item,
                                txn_version,
                                wsc_index,
                                txn_timestamp,
                                table_handle_to_owner,
                            )
                            .unwrap()
                        {
                            token_ownerships_v2.push(token_ownership);
                            if let Some(cto) = current_token_ownership {
                                prior_nft_ownership.insert(
                                    cto.token_data_id.clone(),
                                    NFTOwnershipV2 {
                                        token_data_id: cto.token_data_id.clone(),
                                        owner_address: cto.owner_address.clone(),
                                        is_soulbound: cto.is_soulbound_v2,
                                    },
                                );
                                current_deleted_token_ownerships_v2.insert(
                                    (
                                        cto.token_data_id.clone(),
                                        cto.property_version_v1.clone(),
                                        cto.owner_address.clone(),
                                        cto.storage_id.clone(),
                                    ),
                                    cto,
                                );
                            }
                        }
                        if let Some(current_token_token_claim) =
                            RawCurrentTokenPendingClaim::from_delete_table_item(
                                table_item,
                                txn_version,
                                txn_timestamp,
                                table_handle_to_owner,
                                &tokens_claimed,
                            )
                            .unwrap()
                        {
                            all_current_token_claims.insert(
                                (
                                    current_token_token_claim.token_data_id_hash.clone(),
                                    current_token_token_claim.property_version.clone(),
                                    current_token_token_claim.from_address.clone(),
                                    current_token_token_claim.to_address.clone(),
                                ),
                                current_token_token_claim,
                            );
                        }
                    },
                    Change::WriteResource(resource) => {
                        if let Some((collection, current_collection)) =
                            CollectionV2::get_v2_from_write_resource(
                                resource,
                                txn_version,
                                wsc_index,
                                txn_timestamp,
                                &token_v2_metadata_helper,
                            )
                            .unwrap()
                        {
                            collections_v2.push(collection);
                            current_collections_v2.insert(
                                current_collection.collection_id.clone(),
                                current_collection,
                            );
                        }
                        if let Some((raw_token_data, current_token_data)) =
                            RawTokenDataV2::get_v2_from_write_resource(
                                resource,
                                txn_version,
                                wsc_index,
                                txn_timestamp,
                                &token_v2_metadata_helper,
                            )
                            .unwrap()
                        {
                            // Add NFT ownership
                            let (mut ownerships, current_ownerships) =
                                RawTokenOwnershipV2::get_nft_v2_from_token_data(
                                    &raw_token_data,
                                    &token_v2_metadata_helper,
                                )
                                .unwrap();
                            if let Some(current_nft_ownership) = ownerships.first() {
                                // Note that the first element in ownerships is the current ownership. We need to cache
                                // it in prior_nft_ownership so that moving forward if we see a burn we'll know
                                // where it came from.
                                prior_nft_ownership.insert(
                                    current_nft_ownership.token_data_id.clone(),
                                    NFTOwnershipV2 {
                                        token_data_id: current_nft_ownership.token_data_id.clone(),
                                        owner_address: current_nft_ownership
                                            .owner_address
                                            .as_ref()
                                            .unwrap()
                                            .clone(),
                                        is_soulbound: current_nft_ownership.is_soulbound_v2,
                                    },
                                );
                            }
                            token_ownerships_v2.append(&mut ownerships);
                            current_token_ownerships_v2.extend(current_ownerships);
                            token_datas_v2.push(raw_token_data);
                            current_token_datas_v2.insert(
                                current_token_data.token_data_id.clone(),
                                current_token_data,
                            );
                        }

                        // Add burned NFT handling for token datas (can probably be merged with below)
                        // This handles the case where token is burned but objectCore is still there
                        if let Some(deleted_token_data) =
                            RawTokenDataV2::get_burned_nft_v2_from_write_resource(
                                resource,
                                txn_version,
                                txn_timestamp,
                                &tokens_burned,
                            )
                            .await
                            .unwrap()
                        {
                            current_deleted_token_datas_v2.insert(
                                deleted_token_data.token_data_id.clone(),
                                deleted_token_data,
                            );
                        }
                        // Add burned NFT handling
                        // This handles the case where token is burned but objectCore is still there
                        if let Some((nft_ownership, current_nft_ownership)) =
                            RawTokenOwnershipV2::get_burned_nft_v2_from_write_resource(
                                resource,
                                txn_version,
                                wsc_index,
                                txn_timestamp,
                                &prior_nft_ownership,
                                &tokens_burned,
                                &token_v2_metadata_helper,
                                db_context,
                            )
                            .await
                            .unwrap()
                        {
                            token_ownerships_v2.push(nft_ownership);
                            prior_nft_ownership.insert(
                                current_nft_ownership.token_data_id.clone(),
                                NFTOwnershipV2 {
                                    token_data_id: current_nft_ownership.token_data_id.clone(),
                                    owner_address: current_nft_ownership.owner_address.clone(),
                                    is_soulbound: current_nft_ownership.is_soulbound_v2,
                                },
                            );
                            current_deleted_token_ownerships_v2.insert(
                                (
                                    current_nft_ownership.token_data_id.clone(),
                                    current_nft_ownership.property_version_v1.clone(),
                                    current_nft_ownership.owner_address.clone(),
                                    current_nft_ownership.storage_id.clone(),
                                ),
                                current_nft_ownership,
                            );
                        }

                        // Track token properties
                        if let Some(token_metadata) =
                            RawCurrentTokenV2Metadata::from_write_resource(
                                resource,
                                txn_version,
                                &token_v2_metadata_helper,
                                txn_timestamp,
                            )
                            .unwrap()
                        {
                            current_token_v2_metadata.insert(
                                (
                                    token_metadata.object_address.clone(),
                                    token_metadata.resource_type.clone(),
                                ),
                                token_metadata,
                            );
                        }
                    },
                    Change::DeleteResource(resource) => {
                        // Add burned NFT handling for token datas (can probably be merged with below)
                        if let Some(deleted_token_data) =
                            RawTokenDataV2::get_burned_nft_v2_from_delete_resource(
                                resource,
                                txn_version,
                                txn_timestamp,
                                &tokens_burned,
                            )
                            .await
                            .unwrap()
                        {
                            current_deleted_token_datas_v2.insert(
                                deleted_token_data.token_data_id.clone(),
                                deleted_token_data,
                            );
                        }
                        if let Some((nft_ownership, current_nft_ownership)) =
                            RawTokenOwnershipV2::get_burned_nft_v2_from_delete_resource(
                                resource,
                                txn_version,
                                wsc_index,
                                txn_timestamp,
                                &prior_nft_ownership,
                                &tokens_burned,
                                db_context,
                            )
                            .await
                            .unwrap()
                        {
                            token_ownerships_v2.push(nft_ownership);
                            prior_nft_ownership.insert(
                                current_nft_ownership.token_data_id.clone(),
                                NFTOwnershipV2 {
                                    token_data_id: current_nft_ownership.token_data_id.clone(),
                                    owner_address: current_nft_ownership.owner_address.clone(),
                                    is_soulbound: current_nft_ownership.is_soulbound_v2,
                                },
                            );
                            current_deleted_token_ownerships_v2.insert(
                                (
                                    current_nft_ownership.token_data_id.clone(),
                                    current_nft_ownership.property_version_v1.clone(),
                                    current_nft_ownership.owner_address.clone(),
                                    current_nft_ownership.storage_id.clone(),
                                ),
                                current_nft_ownership,
                            );
                        }
                    },
                    _ => {},
                }
            }
        }
    }

    // Getting list of values and sorting by pk in order to avoid postgres deadlock since we're doing multi threaded db writes
    let mut current_collections_v2 = current_collections_v2
        .into_values()
        .collect::<Vec<CurrentCollectionV2>>();
    let mut current_token_datas_v2 = current_token_datas_v2
        .into_values()
        .collect::<Vec<RawCurrentTokenDataV2>>();
    let mut current_deleted_token_datas_v2 = current_deleted_token_datas_v2
        .into_values()
        .collect::<Vec<RawCurrentTokenDataV2>>();
    let mut current_token_ownerships_v2 = current_token_ownerships_v2
        .into_values()
        .collect::<Vec<RawCurrentTokenOwnershipV2>>();
    let mut current_token_v2_metadata = current_token_v2_metadata
        .into_values()
        .collect::<Vec<RawCurrentTokenV2Metadata>>();
    let mut current_deleted_token_ownerships_v2 = current_deleted_token_ownerships_v2
        .into_values()
        .collect::<Vec<RawCurrentTokenOwnershipV2>>();
    let mut current_token_royalties_v1 = current_token_royalties_v1
        .into_values()
        .collect::<Vec<RawCurrentTokenRoyaltyV1>>();
    let mut all_current_token_claims = all_current_token_claims
        .into_values()
        .collect::<Vec<RawCurrentTokenPendingClaim>>();
    // Sort by PK
    current_collections_v2.sort_by(|a, b| a.collection_id.cmp(&b.collection_id));
    current_deleted_token_datas_v2.sort_by(|a, b| a.token_data_id.cmp(&b.token_data_id));
    current_token_datas_v2.sort_by(|a, b| a.token_data_id.cmp(&b.token_data_id));
    current_token_ownerships_v2.sort();
    current_token_v2_metadata.sort();
    current_deleted_token_ownerships_v2.sort();
    current_token_royalties_v1.sort();
    all_current_token_claims.sort();

    (
        collections_v2,
        token_datas_v2,
        token_ownerships_v2,
        current_collections_v2,
        current_token_datas_v2,
        current_deleted_token_datas_v2,
        current_token_ownerships_v2,
        current_deleted_token_ownerships_v2,
        token_activities_v2,
        current_token_v2_metadata,
        current_token_royalties_v1,
        all_current_token_claims,
    )
}

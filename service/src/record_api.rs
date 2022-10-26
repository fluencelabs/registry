/*
 * Copyright 2021 Fluence Labs Limited
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
use crate::error::ServiceError;
use crate::error::ServiceError::MissingRecordWeight;
use crate::misc::check_weight_result;
use crate::record::{Record, RecordInternal, RecordMetadata};
use crate::record_storage_impl::merge_records;
use crate::results::{GetRecordsResult, MergeResult, RegistryResult, RepublishRecordsResult};
use crate::storage_impl::get_storage;
use crate::tetraplets_checkers::{check_timestamp_tetraplets, check_weight_tetraplets};
use crate::{load_config, wrapped_try, WeightResult};
use marine_rs_sdk::marine;

#[marine]
pub fn get_record_metadata_bytes(
    key_id: String,
    issued_by: String,
    timestamp_issued: u64,
    value: String,
    peer_id: String,
    relay_id: Vec<String>,
    service_id: Vec<String>,
    solution: Vec<u8>,
) -> Vec<u8> {
    RecordMetadata {
        key_id,
        issued_by,
        timestamp_issued,
        value,
        peer_id,
        relay_id,
        service_id,
        solution,
        ..Default::default()
    }
    .signature_bytes()
}

#[marine]
pub fn create_record_metadata(
    key_id: String,
    issued_by: String,
    timestamp_issued: u64,
    value: String,
    peer_id: String,
    relay_id: Vec<String>,
    service_id: Vec<String>,
    solution: Vec<u8>,
    signature: Vec<u8>,
) -> RecordMetadata {
    RecordMetadata {
        key_id,
        issued_by,
        timestamp_issued,
        value,
        peer_id,
        relay_id,
        service_id,
        solution,
        issuer_signature: signature,
    }
}

#[marine]
pub fn get_record_bytes(metadata: RecordMetadata, timestamp_created: u64) -> Vec<u8> {
    Record {
        metadata,
        timestamp_created,
        ..Default::default()
    }
    .signature_bytes()
}

#[marine]
pub fn put_record(
    metadata: RecordMetadata,
    timestamp_created: u64,
    signature: Vec<u8>,
    weight: WeightResult,
    current_timestamp_sec: u64,
) -> RegistryResult {
    wrapped_try(|| {
        let cp = marine_rs_sdk::get_call_parameters();
        check_weight_tetraplets(&cp, 3, 0)?;
        check_timestamp_tetraplets(&cp, 4)?;
        check_weight_result(&cp.init_peer_id, &weight)?;
        let record = Record {
            metadata,
            timestamp_created,
            signature,
        };
        record.verify(current_timestamp_sec)?;

        let storage = get_storage()?;
        storage.check_key_existence(&record.metadata.key_id)?;
        storage.update_record(RecordInternal {
            record,
            weight: weight.weight,
        })
    })
    .into()
}

/// Return all values by key
#[marine]
pub fn get_records(key_id: String, current_timestamp_sec: u64) -> GetRecordsResult {
    wrapped_try(|| {
        let call_parameters = marine_rs_sdk::get_call_parameters();
        check_timestamp_tetraplets(&call_parameters, 1)?;
        let storage = get_storage()?;
        storage.check_key_existence(&key_id)?;
        storage
            .get_records(key_id, current_timestamp_sec)
            .map(|records| records.into_iter().map(|r| r.record).collect())
    })
    .into()
}

/// Return all values by key
#[marine]
pub fn get_stale_local_records(current_timestamp_sec: u64) -> GetRecordsResult {
    wrapped_try(|| {
        let call_parameters = marine_rs_sdk::get_call_parameters();
        check_timestamp_tetraplets(&call_parameters, 0)?;
        let storage = get_storage()?;

        // TODO: add some meaningful constant for expiring local records
        let stale_timestamp_sec = current_timestamp_sec - load_config().expired_timeout + 100;
        storage
            .get_local_stale_records(stale_timestamp_sec)
            .map(|records| records.into_iter().map(|r| r.record).collect())
    })
    .into()
}

/// If the key exists, then merge new records with existing (last-write-wins) and put
#[marine]
pub fn republish_records(
    records: Vec<Record>,
    weights: Vec<WeightResult>,
    current_timestamp_sec: u64,
) -> RepublishRecordsResult {
    wrapped_try(|| {
        if records.is_empty() {
            return Ok(0);
        }

        let key_id = records[0].metadata.key_id.clone();
        let call_parameters = marine_rs_sdk::get_call_parameters();
        check_timestamp_tetraplets(&call_parameters, 2)?;
        let mut records_to_merge = vec![];

        for (i, record) in records.into_iter().enumerate() {
            record.verify(current_timestamp_sec)?;
            check_weight_tetraplets(&call_parameters, 1, i)?;
            let weight_result = weights.get(i).ok_or_else(|| {
                MissingRecordWeight(
                    record.metadata.peer_id.clone(),
                    record.metadata.issued_by.clone(),
                )
            })?;
            check_weight_result(&record.metadata.issued_by, weight_result)?;
            if record.metadata.key_id != key_id {
                return Err(ServiceError::RecordsPublishingError);
            }

            records_to_merge.push(RecordInternal {
                record,
                weight: weight_result.weight,
            });
        }

        let storage = get_storage()?;
        storage.check_key_existence(&key_id)?;
        storage.merge_and_update_records(key_id, records_to_merge, current_timestamp_sec)
    })
    .into()
}

#[marine]
pub fn merge_two(a: Vec<Record>, b: Vec<Record>) -> MergeResult {
    merge_records(
        a.into_iter()
            .chain(b.into_iter())
            .map(|record| RecordInternal {
                record,
                ..Default::default()
            })
            .collect(),
    )
    .map(|recs| recs.into_iter().map(|r| r.record).collect())
    .into()
}

#[marine]
pub fn merge(records: Vec<Vec<Record>>) -> MergeResult {
    merge_records(
        records
            .into_iter()
            .flatten()
            .map(|record| RecordInternal {
                record,
                ..Default::default()
            })
            .collect(),
    )
    .map(|recs| recs.into_iter().map(|r| r.record).collect())
    .into()
}

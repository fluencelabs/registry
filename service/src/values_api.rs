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
use boolinator::Boolinator;

use crate::error::ServiceError;
use crate::key_storage_impl::check_key_existence;
use crate::record::Record;
use crate::record_storage_impl::{
    delete_record, get_records, merge_and_update_records, merge_records, put_record,
};
use crate::results::{
    DhtResult, GetValuesResult, MergeResult, PutHostValueResult, RepublishValuesResult,
};
use crate::storage_impl::get_connection;
use crate::tetraplets_checkers::{
    check_host_value_tetraplets, check_timestamp_tetraplets, check_weight_tetraplets,
};
use crate::{wrapped_try, WeightResult};
use marine_rs_sdk::marine;

#[marine]
pub fn get_value_bytes(
    value: String,
    relay_id: Vec<String>,
    service_id: Vec<String>,
    timestamp_created: u64,
) -> Vec<u8> {
    let cp = marine_rs_sdk::get_call_parameters();
    Record::signature_bytes(
        value,
        cp.init_peer_id.clone(),
        cp.init_peer_id,
        relay_id,
        service_id,
        timestamp_created,
    )
}

// TODO: check that timestamp_created not in the future
#[marine]
pub fn put_value(
    key: String,
    value: String,
    relay_id: Vec<String>,
    service_id: Vec<String>,
    timestamp_created: u64,
    signature: Vec<u8>,
    weight: WeightResult,
    current_timestamp_sec: u64,
) -> DhtResult {
    wrapped_try(|| {
        let cp = marine_rs_sdk::get_call_parameters();
        check_timestamp_tetraplets(&cp, 7)?;
        check_weight_tetraplets(&cp, 6, &weight)?;
        let record = Record {
            value,
            peer_id: cp.init_peer_id.clone(),
            set_by: cp.init_peer_id,
            relay_id,
            service_id,
            timestamp_created,
            signature,
            weight: weight.weight,
        };
        record.verify(key.clone())?;

        put_record(key, record, false, current_timestamp_sec).map(|_| {})
    })
    .into()
}

#[marine]
pub fn get_host_value_bytes(
    value: String,
    relay_id: Vec<String>,
    service_id: Vec<String>,
    timestamp_created: u64,
) -> Vec<u8> {
    let cp = marine_rs_sdk::get_call_parameters();
    Record::signature_bytes(
        value,
        cp.host_id,
        cp.init_peer_id,
        relay_id,
        service_id,
        timestamp_created,
    )
}
#[marine]
pub fn put_host_value(
    key: String,
    value: String,
    timestamp_created: u64,
    relay_id: Vec<String>,
    service_id: Vec<String>,
    signature: Vec<u8>,
    weight: WeightResult,
    current_timestamp_sec: u64,
) -> PutHostValueResult {
    let result_key = key.clone();
    let mut result: PutHostValueResult = wrapped_try(|| {
        let cp = marine_rs_sdk::get_call_parameters();
        check_timestamp_tetraplets(&cp, 7)?;
        check_weight_tetraplets(&cp, 6, &weight)?;
        let record = Record {
            value,
            peer_id: cp.host_id,
            set_by: cp.init_peer_id,
            relay_id,
            service_id,
            timestamp_created,
            signature,
            weight: weight.weight,
        };
        record.verify(key.clone())?;

        put_record(key, record.clone(), true, current_timestamp_sec)?;

        Ok(record)
    })
    .into();

    // key is needed to be passed to propagate_host_value
    result.key = result_key;

    result
}

/// Used for replication of host values to other nodes.
/// Similar to republish_values but with an additional check that value.set_by == init_peer_id
#[marine]
pub fn propagate_host_value(
    mut set_host_value: PutHostValueResult,
    current_timestamp_sec: u64,
    weight: WeightResult,
) -> DhtResult {
    wrapped_try(|| {
        if !set_host_value.success || set_host_value.value.len() != 1 {
            return Err(ServiceError::InvalidSetHostValueResult);
        }

        let key = set_host_value.key;
        let record = &mut set_host_value.value[0];
        record.verify(key.clone())?;

        let call_parameters = marine_rs_sdk::get_call_parameters();
        check_host_value_tetraplets(&call_parameters, 0, record)?;
        check_timestamp_tetraplets(&call_parameters, 1)?;
        check_weight_tetraplets(&call_parameters, 2, &weight)?;

        record.weight = weight.weight;
        merge_and_update_records(key, set_host_value.value, current_timestamp_sec).map(|_| ())
    })
    .into()
}

/// Return all values by key
#[marine]
pub fn get_values(key: String, current_timestamp_sec: u64) -> GetValuesResult {
    wrapped_try(|| {
        let call_parameters = marine_rs_sdk::get_call_parameters();
        check_timestamp_tetraplets(&call_parameters, 1)?;
        let connection = get_connection()?;
        check_key_existence(&connection, key.clone(), current_timestamp_sec)?;
        get_records(&connection, key, Some(current_timestamp_sec))
    })
    .into()
}

/// If the key exists, then merge new records with existing (last-write-wins) and put
#[marine]
pub fn republish_values(
    key: String,
    records: Vec<Record>,
    current_timestamp_sec: u64,
) -> RepublishValuesResult {
    wrapped_try(|| {
        let call_parameters = marine_rs_sdk::get_call_parameters();
        check_timestamp_tetraplets(&call_parameters, 2)?;
        for record in records.iter() {
            record.verify(key.clone())?;
        }

        merge_and_update_records(key, records, current_timestamp_sec)
    })
    .into()
}

// #[marine]
// pub fn renew_host_value(key: String, current_timestamp_sec: u64) -> DhtResult {
//     renew_host_value_impl(key, current_timestamp_sec).into()
// }

/// Remove host value by key and caller peer_id
#[marine]
pub fn clear_host_value(key: String, current_timestamp_sec: u64) -> DhtResult {
    wrapped_try(|| {
        let call_parameters = marine_rs_sdk::get_call_parameters();
        check_timestamp_tetraplets(&call_parameters, 1)?;
        let connection = get_connection()?;

        check_key_existence(&connection, key.clone(), current_timestamp_sec)?;

        let peer_id = call_parameters.host_id;
        let set_by = call_parameters.init_peer_id;
        delete_record(&connection, &key, peer_id, set_by)?;

        (connection.changes() == 1).as_result((), ServiceError::HostValueNotFound(key))
    })
    .into()
}

#[marine]
pub fn merge(records: Vec<Vec<Record>>) -> MergeResult {
    merge_records(records.into_iter().flatten().collect()).into()
}

#[marine]
pub fn merge_two(a: Vec<Record>, b: Vec<Record>) -> MergeResult {
    merge_records(a.into_iter().chain(b.into_iter()).collect()).into()
}

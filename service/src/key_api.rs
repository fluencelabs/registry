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
use crate::key::{Key, KeyInternal};
use crate::misc::check_weight_result;
use crate::results::{DhtResult, GetKeyMetadataResult, RegisterKeyResult};
use crate::storage_impl::get_storage;
use crate::tetraplets_checkers::{check_timestamp_tetraplets, check_weight_tetraplets};
use crate::{wrapped_try, WeightResult};
use marine_rs_sdk::marine;

#[marine]
pub fn get_key_bytes(
    label: String,
    mut peer_id: Vec<String>,
    timestamp_created: u64,
    challenge: Vec<u8>,
    challenge_type: String,
) -> Vec<u8> {
    Key {
        label,
        peer_id: peer_id
            .pop()
            .unwrap_or(marine_rs_sdk::get_call_parameters().init_peer_id),
        timestamp_created,
        challenge,
        challenge_type,
        ..Default::default()
    }
    .signature_bytes()
}

#[marine]
pub fn get_key_id(label: String, peer_id: String) -> String {
    Key::get_key_id(&label, &peer_id)
}

/// register new key if not exists with caller peer_id, update if exists with same peer_id or return error
#[marine]
pub fn register_key(
    key: String,
    peer_id: Vec<String>,
    timestamp_created: u64,
    challenge: Vec<u8>,
    challenge_type: String,
    signature: Vec<u8>,
    pin: bool,
    weight: WeightResult,
    current_timestamp_sec: u64,
) -> RegisterKeyResult {
    wrapped_try(|| {
        let call_parameters = marine_rs_sdk::get_call_parameters();
        check_weight_tetraplets(&call_parameters, 7, 0)?;
        check_timestamp_tetraplets(&call_parameters, 8)?;
        let peer_id = peer_id
            .get(0)
            .unwrap_or(&call_parameters.init_peer_id)
            .clone();
        check_weight_result(&peer_id, &weight)?;
        let key = Key::new(
            key,
            peer_id,
            timestamp_created,
            challenge,
            challenge_type,
            signature,
        );
        key.verify(current_timestamp_sec)?;

        let key_id = key.key_id.clone();
        let weight = weight.weight;
        let storage = get_storage()?;
        storage.update_key_timestamp(&key.key_id, current_timestamp_sec)?;
        storage.update_key(KeyInternal {
            key,
            timestamp_published: 0,
            pinned: pin,
            weight,
        })?;

        Ok(key_id)
    })
    .into()
}

#[marine]
pub fn get_key_metadata(key_id: String, current_timestamp_sec: u64) -> GetKeyMetadataResult {
    wrapped_try(|| {
        let call_parameters = marine_rs_sdk::get_call_parameters();
        check_timestamp_tetraplets(&call_parameters, 1)?;

        let storage = get_storage()?;
        storage.update_key_timestamp(&key_id, current_timestamp_sec)?;
        storage.get_key(key_id)
    })
    .into()
}

/// Used for replication, same as register_key, but key.pinned is ignored, updates timestamp_accessed
#[marine]
pub fn republish_key(mut key: Key, weight: WeightResult, current_timestamp_sec: u64) -> DhtResult {
    wrapped_try(|| {
        let call_parameters = marine_rs_sdk::get_call_parameters();
        check_weight_tetraplets(&call_parameters, 1, 0)?;
        check_weight_result(&key.peer_id, &weight)?;
        check_timestamp_tetraplets(&call_parameters, 2)?;
        key.verify(current_timestamp_sec)?;

        // just to be sure
        key.key_id = Key::get_key_id(&key.label, &key.peer_id);

        let storage = get_storage()?;
        storage.update_key_timestamp(&key.key_id, current_timestamp_sec)?;
        match storage.update_key(KeyInternal {
            key,
            timestamp_published: 0,
            pinned: false,
            weight: weight.weight,
        }) {
            // we should ignore this error for republish
            Err(ServiceError::KeyAlreadyExistsNewerTimestamp(_, _)) => Ok(()),
            other => other,
        }
    })
    .into()
}
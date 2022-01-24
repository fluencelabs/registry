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
use crate::key::Key;
use crate::key_storage_impl::{get_key, update_key};
use crate::results::{DhtResult, GetKeyMetadataResult};
use crate::storage_impl::get_connection;
use crate::tetraplets_checkers::{check_timestamp_tetraplets, check_weight_tetraplets};
use crate::{wrapped_try, WeightResult};
use marine_rs_sdk::marine;

#[marine]
pub fn get_key_bytes(key: String, timestamp_created: u64) -> Vec<u8> {
    Key::signature_bytes(
        key,
        marine_rs_sdk::get_call_parameters().init_peer_id,
        timestamp_created,
    )
}

/// register new key if not exists with caller peer_id, update if exists with same peer_id or return error
// TODO: check that timestamp_created not in the future
#[marine]
pub fn register_key(
    key: String,
    timestamp_created: u64,
    signature: Vec<u8>,
    pin: bool,
    weight: WeightResult,
    current_timestamp_sec: u64,
) -> DhtResult {
    wrapped_try(|| {
        let call_parameters = marine_rs_sdk::get_call_parameters();
        let peer_id = call_parameters.init_peer_id.clone();
        check_timestamp_tetraplets(&call_parameters, 5)?;
        check_weight_tetraplets(&call_parameters, 4, &weight)?;
        Key::verify_signature(
            key.clone(),
            peer_id.clone(),
            timestamp_created,
            signature.clone(),
        )?;

        update_key(
            &get_connection()?,
            key,
            timestamp_created,
            current_timestamp_sec,
            peer_id,
            signature,
            pin,
            weight.weight,
        )
    })
    .into()
}

#[marine]
pub fn get_key_metadata(key: String, current_timestamp_sec: u64) -> GetKeyMetadataResult {
    wrapped_try(|| {
        let call_parameters = marine_rs_sdk::get_call_parameters();
        check_timestamp_tetraplets(&call_parameters, 1)?;

        get_key(&get_connection()?, key, current_timestamp_sec)
    })
    .into()
}

/// Used for replication, same as register_key, but key.pinned is ignored, updates timestamp_accessed
// TODO: ??? weight from local tg should be passed ???
#[marine]
pub fn republish_key(key: Key, current_timestamp_sec: u64) -> DhtResult {
    wrapped_try(|| {
        let call_parameters = marine_rs_sdk::get_call_parameters();
        check_timestamp_tetraplets(&call_parameters, 1)?;
        key.verify()?;

        // Key.pinned is ignored in republish
        update_key(
            &get_connection()?,
            key.key,
            key.timestamp_created,
            current_timestamp_sec,
            key.peer_id,
            key.signature,
            false,
            key.weight,
        )
    })
    .into()
}

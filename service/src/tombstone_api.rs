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
use crate::results::{GetTombstonesResult, RegistryResult};
use crate::storage_impl::get_storage;
use crate::tetraplets_checkers::check_timestamp_tetraplets;
use crate::tombstone::Tombstone;
use crate::wrapped_try;
use marine_rs_sdk::marine;

#[marine]
pub fn get_tombstone_bytes(
    key_id: String,
    issued_by: String,
    peer_id: String,
    timestamp_issued: u64,
    solution: Vec<u8>,
) -> Vec<u8> {
    Tombstone {
        key_id,
        issued_by,
        peer_id,
        timestamp_issued,
        solution,
        ..Default::default()
    }
    .signature_bytes()
}

#[marine]
pub fn add_tombstone(
    key_id: String,
    issued_by: String,
    peer_id: String,
    timestamp_issued: u64,
    solution: Vec<u8>,
    signature: Vec<u8>,
    current_timestamp_sec: u64,
) -> RegistryResult {
    wrapped_try(|| {
        let cp = marine_rs_sdk::get_call_parameters();
        check_timestamp_tetraplets(&cp, 5)?;
        let tombstone = Tombstone {
            key_id,
            issued_by,
            peer_id,
            timestamp_issued,
            solution,
            issuer_signature: signature,
        };
        tombstone.verify(current_timestamp_sec)?;

        let storage = get_storage()?;
        storage.check_key_existence(&tombstone.key_id)?;
        storage.write_tombstone(tombstone)
    })
    .into()
}

/// Return all tombstones by key id
#[marine]
pub fn get_tombstones(key_id: String, current_timestamp_sec: u64) -> GetTombstonesResult {
    wrapped_try(|| {
        let call_parameters = marine_rs_sdk::get_call_parameters();
        check_timestamp_tetraplets(&call_parameters, 1)?;
        let storage = get_storage()?;
        storage.check_key_existence(&key_id)?;
        storage.get_tombstones(key_id, current_timestamp_sec)
    })
    .into()
}

/// If the key exists, then merge tombstones with existing (last-write-wins)
#[marine]
pub fn republish_tombstones(
    tombstones: Vec<Tombstone>,
    current_timestamp_sec: u64,
) -> RegistryResult {
    wrapped_try(|| {
        if tombstones.is_empty() {
            return Ok(());
        }

        let key_id = tombstones[0].key_id.clone();
        let call_parameters = marine_rs_sdk::get_call_parameters();
        check_timestamp_tetraplets(&call_parameters, 1)?;

        for tombstone in tombstones.iter() {
            tombstone.verify(current_timestamp_sec)?;

            if tombstone.key_id != key_id {
                return Err(ServiceError::TombstonesPublishingError);
            }
        }

        let storage = get_storage()?;
        storage.check_key_existence(&key_id)?;
        for tombstone in tombstones.into_iter() {
            storage.write_tombstone(tombstone)?;
        }

        Ok(())
    })
    .into()
}

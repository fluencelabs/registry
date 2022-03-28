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
use crate::misc::extract_public_key;
use fluence_keypair::Signature;
use marine_rs_sdk::marine;
use sha2::{Digest, Sha256};

#[marine]
#[derive(Debug, Default, Clone)]
pub struct Record {
    pub route_id: String,
    pub value: String,
    pub peer_id: String,
    pub set_by: String,
    pub relay_id: Vec<String>,
    pub service_id: Vec<String>,
    pub timestamp_created: u64,
    pub solution: Vec<u8>,
    pub signature: Vec<u8>,
}

#[derive(Default, Debug, Clone)]
pub struct RecordInternal {
    pub record: Record,
    pub weight: u32,
}

impl Record {
    pub fn signature_bytes(&self) -> Vec<u8> {
        let mut metadata = Vec::new();
        metadata.extend(self.route_id.as_bytes());
        metadata.extend(self.value.as_bytes());
        metadata.extend(self.peer_id.as_bytes());
        metadata.extend(self.set_by.as_bytes());

        if !self.relay_id.is_empty() {
            metadata.extend(self.relay_id.len().to_le_bytes());

            for id in &self.relay_id {
                metadata.extend(id.as_bytes());
            }
        }

        if !self.service_id.is_empty() {
            metadata.extend(self.service_id.len().to_le_bytes());

            for id in &self.service_id {
                metadata.extend(id.as_bytes());
            }
        }

        metadata.extend(self.timestamp_created.to_le_bytes());
        metadata.extend(&self.solution);
        let mut hasher = Sha256::new();
        hasher.update(metadata);
        hasher.finalize().to_vec()
    }

    pub fn verify(&self, current_timestamp_sec: u64) -> Result<(), ServiceError> {
        if self.timestamp_created > current_timestamp_sec {
            return Err(ServiceError::InvalidRecordTimestamp);
        }

        // TODO: now we have signatures only by js peers
        // so for all record is true only signatures by set_by
        // later we should add signatures by peer_id (for host records) and by relays
        let pk = extract_public_key(self.set_by.clone())?;
        let bytes = self.signature_bytes();
        let signature = Signature::from_bytes(pk.get_key_format(), self.signature.clone());
        pk.verify(&bytes, &signature).map_err(|e| {
            ServiceError::InvalidRecordSignature(self.route_id.clone(), self.value.clone(), e)
        })
    }
}

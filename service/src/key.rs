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
use marine_rs_sdk::marine;
use sha2::{Digest, Sha256};

#[marine]
#[derive(Default, Clone)]
pub struct Key {
    pub key: String,
    pub peer_id: String,
    pub timestamp_created: u64,
    pub signature: Vec<u8>,
    pub pinned: bool,
    pub weight: u32,
}

impl Key {
    pub fn signature_bytes(key: String, peer_id: String, timestamp_created: u64) -> Vec<u8> {
        let mut metadata = Vec::new();
        metadata.extend(key.as_bytes());
        metadata.extend(peer_id.as_bytes());
        metadata.extend(timestamp_created.to_le_bytes());

        let mut hasher = Sha256::new();
        hasher.update(metadata);
        hasher.finalize().to_vec()
    }

    pub fn verify(&self) -> Result<(), ServiceError> {
        Self::verify_signature(
            self.key.clone(),
            self.peer_id.clone(),
            self.timestamp_created,
            self.signature.clone(),
        )
    }

    pub fn verify_signature(
        key: String,
        peer_id: String,
        timestamp_created: u64,
        signature: Vec<u8>,
    ) -> Result<(), ServiceError> {
        if signature.eq(&Self::signature_bytes(
            key.clone(),
            peer_id,
            timestamp_created,
        )) {
            Ok(())
        } else {
            Err(ServiceError::InvalidKeySignature(key))
        }
    }
}

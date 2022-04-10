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
#[derive(Default, Clone)]
pub struct Key {
    pub id: String,
    pub label: String,
    pub owner_peer_id: String,
    pub timestamp_created: u64,
    pub challenge: Vec<u8>,
    pub challenge_type: String,
    pub signature: Vec<u8>,
}

#[derive(Default, Clone)]
pub struct KeyInternal {
    pub key: Key,
    pub timestamp_published: u64,
    pub pinned: bool,
    pub weight: u32,
}

impl Key {
    pub fn new(
        label: String,
        owner_peer_id: String,
        timestamp_created: u64,
        challenge: Vec<u8>,
        challenge_type: String,
        signature: Vec<u8>,
    ) -> Self {
        let id = Self::get_id(&label, &owner_peer_id);

        Self {
            id,
            label,
            owner_peer_id,
            timestamp_created,
            challenge,
            challenge_type,
            signature,
        }
    }

    pub fn get_id(label: &str, owner_peer_id: &str) -> String {
        format!("{}{}", label, owner_peer_id)
    }

    pub fn signature_bytes(&self) -> Vec<u8> {
        let mut metadata = Vec::new();
        metadata.extend(self.label.as_bytes());
        metadata.extend(self.owner_peer_id.as_bytes());
        metadata.extend(self.timestamp_created.to_le_bytes());
        metadata.extend(&self.challenge);
        metadata.extend(self.challenge_type.as_bytes());

        let mut hasher = Sha256::new();
        hasher.update(metadata);
        hasher.finalize().to_vec()
    }

    pub fn verify(&self, current_timestamp_sec: u64) -> Result<(), ServiceError> {
        if self.timestamp_created > current_timestamp_sec {
            return Err(ServiceError::InvalidKeyTimestamp);
        }

        self.verify_signature()
    }

    pub fn verify_signature(&self) -> Result<(), ServiceError> {
        let pk = extract_public_key(self.owner_peer_id.clone())?;
        let bytes = self.signature_bytes();
        let signature = Signature::from_bytes(pk.get_key_format(), self.signature.clone());
        pk.verify(&bytes, &signature)
            .map_err(|e| ServiceError::InvalidKeySignature(self.label.clone(), e))
    }
}

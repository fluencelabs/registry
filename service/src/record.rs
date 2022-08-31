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
pub struct RecordMetadata {
    pub key_id: String,
    pub issued_by: String,
    pub timestamp_issued: u64,
    pub value: String,
    pub peer_id: String,
    pub relay_id: Vec<String>,
    pub service_id: Vec<String>,
    pub solution: Vec<u8>,
    pub issuer_signature: Vec<u8>,
}

impl RecordMetadata {
    pub fn signature_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(self.key_id.len() as u8);
        bytes.extend(self.key_id.as_bytes());

        bytes.push(self.issued_by.len() as u8);
        bytes.extend(self.issued_by.as_bytes());

        bytes.extend(self.timestamp_issued.to_le_bytes());

        bytes.push(self.value.len() as u8);
        bytes.extend(self.value.as_bytes());

        bytes.push(self.peer_id.len() as u8);
        bytes.extend(self.peer_id.as_bytes());

        bytes.extend(self.relay_id.len().to_le_bytes());
        for id in &self.relay_id {
            bytes.push(id.len() as u8);
            bytes.extend(id.as_bytes());
        }

        bytes.extend(self.service_id.len().to_le_bytes());
        for id in &self.service_id {
            bytes.push(id.len() as u8);
            bytes.extend(id.as_bytes());
        }

        bytes.push(self.solution.len() as u8);
        bytes.extend(&self.solution);

        let mut hasher = Sha256::new();
        hasher.update(bytes);
        hasher.finalize().to_vec()
    }

    pub fn verify(&self, current_timestamp_sec: u64) -> Result<(), ServiceError> {
        if self.timestamp_issued > current_timestamp_sec {
            return Err(ServiceError::InvalidRecordMetadataTimestamp);
        }

        let pk = extract_public_key(self.issued_by.clone())?;
        let bytes = self.signature_bytes();
        let signature = Signature::from_bytes(pk.get_key_format(), self.issuer_signature.clone());
        pk.verify(&bytes, &signature).map_err(|e| {
            ServiceError::InvalidRecordMetadataSignature(
                self.key_id.clone(),
                self.issued_by.clone(),
                e,
            )
        })
    }
}

#[marine]
#[derive(Debug, Default, Clone)]
pub struct Record {
    pub metadata: RecordMetadata,
    pub timestamp_created: u64,
    pub signature: Vec<u8>,
}

#[derive(Default, Debug, Clone)]
pub struct RecordInternal {
    pub record: Record,
    pub weight: u32,
}

impl Record {
    pub fn signature_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        let mut metadata = self.metadata.signature_bytes();
        metadata.push(self.metadata.issuer_signature.len() as u8);
        metadata.extend(&self.metadata.issuer_signature);

        bytes.push(metadata.len() as u8);
        bytes.append(&mut metadata);

        bytes.extend(self.timestamp_created.to_le_bytes());
        let mut hasher = Sha256::new();
        hasher.update(metadata);
        hasher.finalize().to_vec()
    }

    pub fn verify(&self, current_timestamp_sec: u64) -> Result<(), ServiceError> {
        if self.timestamp_created > current_timestamp_sec {
            return Err(ServiceError::InvalidRecordTimestamp);
        }

        let pk = extract_public_key(self.metadata.peer_id.clone())?;
        let bytes = self.signature_bytes();
        let signature = Signature::from_bytes(pk.get_key_format(), self.signature.clone());
        pk.verify(&bytes, &signature).map_err(|e| {
            ServiceError::InvalidRecordSignature(
                self.metadata.key_id.clone(),
                self.metadata.peer_id.clone(),
                e,
            )
        })
    }
}

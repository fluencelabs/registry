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
#[cfg(test)]
mod tests {
    use fluence_keypair::KeyPair;
    use std::fs;

    use marine_rs_sdk::{CallParameters, SecurityTetraplet};
    use rusqlite::Connection;
    marine_rs_sdk_test::include_test_env!("/marine_test_env.rs");
    use marine_test_env::registry::{
        PutHostRecordResult, Record, RegistryResult, ServiceInterface,
    };

    use crate::defaults::{
        CONFIG_FILE, DB_PATH, KEYS_TABLE_NAME, KEYS_TIMESTAMPS_TABLE_NAME, RECORDS_TABLE_NAME,
        TRUSTED_TIMESTAMP_FUNCTION_NAME, TRUSTED_TIMESTAMP_SERVICE_ID,
        TRUSTED_WEIGHT_FUNCTION_NAME, TRUSTED_WEIGHT_SERVICE_ID,
    };
    use crate::error::ServiceError::{
        InvalidKeyTimestamp, InvalidTimestampTetraplet, InvalidWeightPeerId,
        KeyAlreadyExistsNewerTimestamp,
    };
    use crate::tests::tests::marine_test_env::registry::{Key, RegisterKeyResult, WeightResult};

    const HOST_ID: &str = "some_host_id";

    impl PartialEq for Key {
        fn eq(&self, other: &Self) -> bool {
            self.id == other.id
                && self.label == other.label
                && self.timestamp_created == other.timestamp_created
                && self.signature == other.signature
                && self.owner_peer_id == other.owner_peer_id
        }
    }

    impl Eq for Key {}

    fn clear_env() {
        let connection = Connection::open(DB_PATH).unwrap();

        connection
            .execute(f!("DROP TABLE IF EXISTS {KEYS_TABLE_NAME}").as_str(), [])
            .unwrap();
        connection
            .execute(
                f!("DROP TABLE IF EXISTS {KEYS_TIMESTAMPS_TABLE_NAME}").as_str(),
                [],
            )
            .unwrap();
        connection
            .execute(f!("DROP TABLE IF EXISTS {RECORDS_TABLE_NAME}").as_str(), [])
            .unwrap();

        if fs::metadata(CONFIG_FILE).is_ok() {
            fs::remove_file(CONFIG_FILE).unwrap();
        }
    }

    struct CPWrapper {
        pub cp: CallParameters,
    }

    impl CPWrapper {
        pub fn new(init_peer_id: &str) -> Self {
            Self {
                cp: CallParameters {
                    init_peer_id: init_peer_id.to_string(),
                    service_id: "".to_string(),
                    service_creator_peer_id: "".to_string(),
                    host_id: HOST_ID.to_string(),
                    particle_id: "".to_string(),
                    tetraplets: vec![],
                },
            }
        }

        pub fn add_timestamp_tetraplets(mut self, arg_number: usize) -> Self {
            if self.cp.tetraplets.len() <= arg_number {
                self.cp.tetraplets.resize(arg_number + 1, vec![]);
            }

            self.cp.tetraplets[arg_number] = vec![SecurityTetraplet {
                peer_pk: HOST_ID.to_string(),
                service_id: TRUSTED_TIMESTAMP_SERVICE_ID.to_string(),
                function_name: TRUSTED_TIMESTAMP_FUNCTION_NAME.to_string(),
                json_path: "".to_string(),
            }];

            self
        }

        fn add_weight_tetraplets(mut self, arg_number: usize) -> Self {
            if self.cp.tetraplets.len() < arg_number {
                self.cp.tetraplets.resize(arg_number + 1, vec![]);
            }

            self.cp.tetraplets[arg_number] = vec![SecurityTetraplet {
                peer_pk: HOST_ID.to_string(),
                service_id: TRUSTED_WEIGHT_SERVICE_ID.to_string(),
                function_name: TRUSTED_WEIGHT_FUNCTION_NAME.to_string(),
                json_path: "".to_string(),
            }];

            self
        }

        pub fn get(&self) -> CallParameters {
            self.cp.clone()
        }

        pub fn reset(&mut self) {
            self.cp.tetraplets = vec![];
        }
    }

    fn get_weight(peer_id: String, weight: u32) -> WeightResult {
        WeightResult {
            success: true,
            weight,
            peer_id,
            error: "".to_string(),
        }
    }

    fn get_invalid_weight() -> WeightResult {
        WeightResult {
            success: false,
            weight: 0,
            peer_id: "".to_string(),
            error: "get_weight call failed".to_string(),
        }
    }

    fn get_signed_key_bytes(
        registry: &mut ServiceInterface,
        kp: &KeyPair,
        label: String,
        timestamp_created: u64,
        challenge: Vec<u8>,
        challenge_type: String,
    ) -> Vec<u8> {
        let issuer_peer_id = kp.get_peer_id().to_base58();
        let key_bytes = registry.get_key_bytes(
            label.clone(),
            vec![issuer_peer_id.clone()],
            timestamp_created,
            challenge,
            challenge_type,
        );
        kp.sign(&key_bytes).unwrap().to_vec().to_vec()
    }

    fn register_key(
        registry: &mut ServiceInterface,
        kp: &KeyPair,
        label: String,
        timestamp_created: u64,
        current_timestamp: u64,
        pin: bool,
        weight: u32,
    ) -> RegisterKeyResult {
        let issuer_peer_id = kp.get_peer_id().to_base58();
        let challenge = vec![];
        let challenge_type = "".to_string();
        let signature = get_signed_key_bytes(
            registry,
            kp,
            label.clone(),
            timestamp_created,
            challenge.clone(),
            challenge_type.clone(),
        );
        let cp = CPWrapper::new(&issuer_peer_id)
            .add_weight_tetraplets(7)
            .add_timestamp_tetraplets(8);
        let weight = get_weight(issuer_peer_id.clone(), weight);
        registry.register_key_cp(
            label,
            vec![issuer_peer_id],
            timestamp_created,
            challenge,
            challenge_type,
            signature,
            pin,
            weight,
            current_timestamp,
            cp.get(),
        )
    }

    fn register_key_checked(
        registry: &mut ServiceInterface,
        kp: &KeyPair,
        key: String,
        timestamp_created: u64,
        current_timestamp: u64,
        pin: bool,
        weight: u32,
    ) -> String {
        let result = register_key(
            registry,
            kp,
            key,
            timestamp_created,
            current_timestamp,
            pin,
            weight,
        );
        assert!(result.success, "{}", result.error);
        result.key_id
    }

    fn get_key_metadata(
        registry: &mut ServiceInterface,
        key_id: String,
        current_timestamp: u64,
    ) -> Key {
        let cp = CPWrapper::new("peer_id").add_timestamp_tetraplets(1);
        let result = registry.get_key_metadata_cp(key_id, current_timestamp, cp.get());
        assert!(result.success, "{}", result.error);
        result.key
    }

    fn get_signed_record_bytes(
        registry: &mut ServiceInterface,
        kp: &KeyPair,
        key_id: String,
        value: String,
        relay_id: Vec<String>,
        service_id: Vec<String>,
        timestamp_created: u64,
        solution: Vec<u8>,
    ) -> Vec<u8> {
        let issuer_peer_id = kp.get_peer_id().to_base58();
        let cp = CPWrapper::new(&issuer_peer_id);
        let record_bytes = registry.get_record_bytes_cp(
            key_id,
            value,
            relay_id,
            service_id,
            timestamp_created,
            solution,
            cp.get(),
        );

        kp.sign(&record_bytes).unwrap().to_vec().to_vec()
    }

    fn put_record(
        registry: &mut ServiceInterface,
        kp: &KeyPair,
        key_id: String,
        value: String,
        relay_id: Vec<String>,
        service_id: Vec<String>,
        timestamp_created: u64,
        current_timestamp: u64,
        weight: u32,
    ) -> RegistryResult {
        let issuer_peer_id = kp.get_peer_id().to_base58();
        let solution = vec![];

        let signature = get_signed_record_bytes(
            registry,
            kp,
            key_id.clone(),
            value.clone(),
            relay_id.clone(),
            service_id.clone(),
            timestamp_created,
            solution.clone(),
        );

        let cp = CPWrapper::new(&issuer_peer_id)
            .add_weight_tetraplets(7)
            .add_timestamp_tetraplets(8);
        let weight = get_weight(issuer_peer_id.clone(), weight);
        registry.put_record_cp(
            key_id,
            value,
            relay_id,
            service_id,
            timestamp_created,
            solution,
            signature,
            weight,
            current_timestamp,
            cp.get(),
        )
    }

    fn put_record_checked(
        registry: &mut ServiceInterface,
        kp: &KeyPair,
        key_id: String,
        value: String,
        relay_id: Vec<String>,
        service_id: Vec<String>,
        timestamp_created: u64,
        current_timestamp: u64,
        weight: u32,
    ) {
        let result = put_record(
            registry,
            kp,
            key_id,
            value,
            relay_id,
            service_id,
            timestamp_created,
            current_timestamp,
            weight,
        );
        assert!(result.success, "{}", result.error);
    }

    fn get_signed_host_record_bytes(
        registry: &mut ServiceInterface,
        kp: &KeyPair,
        key_id: String,
        value: String,
        relay_id: Vec<String>,
        service_id: Vec<String>,
        timestamp_created: u64,
        solution: Vec<u8>,
    ) -> Vec<u8> {
        let issuer_peer_id = kp.get_peer_id().to_base58();
        let cp = CPWrapper::new(&issuer_peer_id);
        let record_bytes = registry.get_host_record_bytes_cp(
            key_id,
            value,
            relay_id,
            service_id,
            timestamp_created,
            solution,
            cp.get(),
        );

        kp.sign(&record_bytes).unwrap().to_vec().to_vec()
    }

    fn put_host_record(
        registry: &mut ServiceInterface,
        kp: &KeyPair,
        key_id: String,
        value: String,
        relay_id: Vec<String>,
        service_id: Vec<String>,
        timestamp_created: u64,
        current_timestamp: u64,
        weight: u32,
    ) -> PutHostRecordResult {
        let issuer_peer_id = kp.get_peer_id().to_base58();
        let solution = vec![];

        let signature = get_signed_host_record_bytes(
            registry,
            kp,
            key_id.clone(),
            value.clone(),
            relay_id.clone(),
            service_id.clone(),
            timestamp_created,
            solution.clone(),
        );

        let cp = CPWrapper::new(&issuer_peer_id)
            .add_weight_tetraplets(7)
            .add_timestamp_tetraplets(8);
        let weight = get_weight(issuer_peer_id.clone(), weight);
        registry.put_host_record_cp(
            key_id,
            value,
            relay_id,
            service_id,
            timestamp_created,
            solution,
            signature,
            weight,
            current_timestamp,
            cp.get(),
        )
    }

    fn put_host_record_checked(
        registry: &mut ServiceInterface,
        kp: &KeyPair,
        key_id: String,
        value: String,
        relay_id: Vec<String>,
        service_id: Vec<String>,
        timestamp_created: u64,
        current_timestamp: u64,
        weight: u32,
    ) {
        let result = put_host_record(
            registry,
            kp,
            key_id,
            value,
            relay_id,
            service_id,
            timestamp_created,
            current_timestamp,
            weight,
        );
        assert!(result.success, "{}", result.error);
    }

    fn get_records(
        registry: &mut ServiceInterface,
        key_id: String,
        current_timestamp: u64,
    ) -> Vec<Record> {
        let cp = CPWrapper::new("some_peer_id").add_timestamp_tetraplets(1);

        let result = registry.get_records_cp(key_id, current_timestamp, cp.get());
        assert!(result.success, "{}", result.error);
        result.result
    }

    #[test]
    fn register_key_invalid_signature() {
        clear_env();
        let mut registry = ServiceInterface::new();
        let kp = KeyPair::generate_ed25519();
        let issuer_peer_id = kp.get_peer_id().to_base58();
        let mut cp = CPWrapper::new(&issuer_peer_id);
        let weight = get_weight(issuer_peer_id.clone(), 0);

        let invalid_signature = vec![];

        cp = cp.add_weight_tetraplets(5).add_timestamp_tetraplets(6);
        let reg_key_result = registry.register_key_cp(
            "some_key".to_string(),
            vec![],
            100u64,
            vec![],
            "".to_string(),
            invalid_signature,
            false,
            weight,
            10u64,
            cp.get(),
        );
        assert!(!reg_key_result.success);
    }

    #[test]
    fn register_key_invalid_weight_tetraplet() {
        clear_env();
        let mut registry = ServiceInterface::new();
        let kp = KeyPair::generate_ed25519();
        let issuer_peer_id = kp.get_peer_id().to_base58();
        let mut cp = CPWrapper::new(&issuer_peer_id);
        let label = "some_key".to_string();
        let timestamp_created = 0u64;
        let current_timestamp = 100u64;
        let challenge = vec![];
        let challenge_type = "".to_string();
        let weight = get_weight(issuer_peer_id.clone(), 0);

        let signature = get_signed_key_bytes(
            &mut registry,
            &kp,
            label.clone(),
            timestamp_created,
            challenge.clone(),
            challenge_type.clone(),
        );

        cp = cp.add_timestamp_tetraplets(8);
        let reg_key_result = registry.register_key_cp(
            label,
            vec![],
            timestamp_created,
            challenge,
            challenge_type,
            signature,
            false,
            weight,
            current_timestamp,
            cp.get(),
        );
        assert!(!reg_key_result.success);
    }

    #[test]
    fn register_key_missing_timestamp_tetraplet() {
        clear_env();
        let mut registry = ServiceInterface::new();
        let kp = KeyPair::generate_ed25519();
        let issuer_peer_id = kp.get_peer_id().to_base58();
        let label = "some_key".to_string();
        let timestamp_created = 0u64;
        let current_timestamp = 100u64;
        let weight = get_weight(issuer_peer_id.clone(), 0);
        let challenge = vec![1u8, 2u8, 3u8];
        let challenge_type = "type".to_string();

        let signature = get_signed_key_bytes(
            &mut registry,
            &kp,
            label.clone(),
            timestamp_created,
            challenge.clone(),
            challenge_type.clone(),
        );

        let cp = CPWrapper::new(&issuer_peer_id).add_weight_tetraplets(7);
        let reg_key_result = registry.register_key_cp(
            label,
            vec![],
            timestamp_created,
            challenge,
            challenge_type,
            signature,
            false,
            weight,
            current_timestamp,
            cp.get(),
        );
        assert!(!reg_key_result.success);
        assert_eq!(
            reg_key_result.error,
            InvalidTimestampTetraplet(format!("{:?}", cp.cp.tetraplets)).to_string()
        );
    }

    #[test]
    fn register_key_invalid_weight_peer_id() {
        clear_env();
        let mut registry = ServiceInterface::new();
        let kp = KeyPair::generate_ed25519();
        let issuer_peer_id = kp.get_peer_id().to_base58();
        let invalid_peer_id = "INVALID_PEER_ID".to_string();
        let mut cp = CPWrapper::new(&issuer_peer_id);
        let label = "some_key".to_string();
        let timestamp_created = 0u64;
        let current_timestamp = 100u64;
        let challenge = vec![1u8, 2u8, 3u8];
        let challenge_type = "type".to_string();
        let weight = get_weight(invalid_peer_id.clone(), 0);

        let signature = get_signed_key_bytes(
            &mut registry,
            &kp,
            label.clone(),
            timestamp_created,
            challenge.clone(),
            challenge_type.clone(),
        );

        cp = cp.add_weight_tetraplets(7).add_timestamp_tetraplets(8);
        let reg_key_result = registry.register_key_cp(
            label,
            vec![],
            timestamp_created,
            challenge,
            challenge_type,
            signature,
            false,
            weight,
            current_timestamp,
            cp.get(),
        );
        assert!(!reg_key_result.success);
        assert_eq!(
            reg_key_result.error,
            InvalidWeightPeerId(issuer_peer_id, invalid_peer_id).to_string()
        );
    }

    #[test]
    fn register_key_correct() {
        clear_env();
        let mut registry = ServiceInterface::new();
        let kp = KeyPair::generate_ed25519();
        let key = "some_key".to_string();
        let timestamp_created = 0u64;
        let current_timestamp = 100u64;
        let weight = 0;
        let pin = false;

        let result = register_key(
            &mut registry,
            &kp,
            key,
            timestamp_created,
            current_timestamp,
            pin,
            weight,
        );

        assert!(result.success, "{}", result.error);
    }

    #[test]
    fn register_key_older_timestamp() {
        clear_env();
        let mut registry = ServiceInterface::new();
        let kp = KeyPair::generate_ed25519();
        let key = "some_key".to_string();
        let timestamp_created_first = 100u64;
        let current_timestamp = 1000u64;
        let weight = 0;
        let pin = false;

        register_key_checked(
            &mut registry,
            &kp,
            key.clone(),
            timestamp_created_first,
            current_timestamp,
            pin,
            weight,
        );

        let timestamp_created_second = timestamp_created_first - 10u64;
        let result_second = register_key(
            &mut registry,
            &kp,
            key.clone(),
            timestamp_created_second,
            current_timestamp,
            pin,
            weight,
        );

        assert_eq!(
            result_second.error,
            KeyAlreadyExistsNewerTimestamp(key, kp.get_peer_id().to_base58()).to_string()
        );
    }

    #[test]
    fn register_key_in_the_future() {
        clear_env();
        let mut registry = ServiceInterface::new();
        let kp = KeyPair::generate_ed25519();
        let key = "some_key".to_string();
        let current_timestamp = 100u64;
        let timestamp_created = current_timestamp + 100u64;
        let weight = 0;
        let pin = false;

        let result = register_key(
            &mut registry,
            &kp,
            key,
            timestamp_created,
            current_timestamp,
            pin,
            weight,
        );

        assert_eq!(result.error, InvalidKeyTimestamp.to_string())
    }

    #[test]
    fn register_key_update_republish_old() {
        clear_env();
        let mut registry = ServiceInterface::new();
        let kp = KeyPair::generate_ed25519();
        let issuer_peer_id = kp.get_peer_id().to_base58();
        let key = "some_key".to_string();
        let timestamp_created_old = 0u64;
        let current_timestamp = 100u64;
        let weight = 0;
        let pin = false;

        let key_id = register_key_checked(
            &mut registry,
            &kp,
            key.clone(),
            timestamp_created_old,
            current_timestamp,
            pin,
            weight,
        );

        let old_key = get_key_metadata(&mut registry, key_id.clone(), current_timestamp);

        let timestamp_created_new = timestamp_created_old + 10u64;
        register_key_checked(
            &mut registry,
            &kp,
            key,
            timestamp_created_new,
            current_timestamp,
            pin,
            weight,
        );
        let new_key = get_key_metadata(&mut registry, key_id.clone(), current_timestamp);
        assert_ne!(old_key, new_key);

        let cp = CPWrapper::new(&issuer_peer_id)
            .add_weight_tetraplets(1)
            .add_timestamp_tetraplets(2);
        let weight = get_weight(issuer_peer_id.clone(), weight);
        let result =
            registry.republish_key_cp(old_key.clone(), weight, current_timestamp, cp.get());
        assert!(result.success, "{}", result.error);

        let result_key = get_key_metadata(&mut registry, key_id.clone(), current_timestamp);
        assert_eq!(new_key, result_key);
    }

    #[test]
    fn get_key_metadata_test() {
        clear_env();
        let mut registry = ServiceInterface::new();
        let kp = KeyPair::generate_ed25519();
        let label = "some_key".to_string();
        let timestamp_created = 0u64;
        let current_timestamp = 100u64;
        let weight = 0;
        let pin = false;
        let challenge = vec![];
        let challenge_type = "".to_string();
        let issuer_peer_id = kp.get_peer_id().to_base58();

        let key_bytes = registry.get_key_bytes(
            label.clone(),
            vec![issuer_peer_id.clone()],
            timestamp_created,
            challenge.clone(),
            challenge_type.clone(),
        );
        let signature = kp.sign(&key_bytes).unwrap().to_vec().to_vec();

        let key_id = register_key_checked(
            &mut registry,
            &kp,
            label.clone(),
            timestamp_created,
            current_timestamp,
            pin,
            weight,
        );

        let result_key = get_key_metadata(&mut registry, key_id.clone(), current_timestamp);
        let expected_key = Key {
            id: key_id,
            label,
            owner_peer_id: issuer_peer_id,
            timestamp_created,
            challenge,
            challenge_type,
            signature,
        };
        assert_eq!(result_key, expected_key);
    }

    #[test]
    fn republish_same_key_test() {
        clear_env();
        let mut registry = ServiceInterface::new();
        let kp = KeyPair::generate_ed25519();
        let issuer_peer_id = kp.get_peer_id().to_base58();
        let key = "some_key".to_string();
        let timestamp_created = 0u64;
        let current_timestamp = 100u64;
        let weight = 0;
        let pin = false;

        let key_id = register_key_checked(
            &mut registry,
            &kp,
            key.clone(),
            timestamp_created,
            current_timestamp,
            pin,
            weight,
        );

        let result_key = get_key_metadata(&mut registry, key_id.clone(), current_timestamp);
        let cp = CPWrapper::new(&issuer_peer_id)
            .add_weight_tetraplets(1)
            .add_timestamp_tetraplets(2);
        let weight = get_weight(issuer_peer_id.clone(), weight);
        let result =
            registry.republish_key_cp(result_key.clone(), weight, current_timestamp, cp.get());
        assert!(result.success, "{}", result.error);
    }

    #[test]
    fn test_put_get_record() {
        clear_env();
        let mut registry = ServiceInterface::new();
        let kp = KeyPair::generate_ed25519();
        let key = "some_key".to_string();
        let timestamp_created = 0u64;
        let current_timestamp = 100u64;
        let weight = 0;
        let pin = false;

        let key_id = register_key_checked(
            &mut registry,
            &kp,
            key,
            timestamp_created,
            current_timestamp,
            pin,
            weight,
        );
        let value = "some_value".to_string();
        let relay_id = vec!["some_relay".to_string()];
        let service_id = vec!["some_service_id".to_string()];
        let weight = 5u32;

        put_record_checked(
            &mut registry,
            &kp,
            key_id.clone(),
            value.clone(),
            relay_id.clone(),
            service_id.clone(),
            timestamp_created,
            current_timestamp,
            weight,
        );

        let records = get_records(&mut registry, key_id.clone(), current_timestamp);
        assert_eq!(records.len(), 1);
        let record = &records[0];
        assert_eq!(record.key_id, key_id);
        assert_eq!(record.relay_id, relay_id);
        assert_eq!(record.service_id, service_id);
        assert_eq!(record.peer_id, kp.get_peer_id().to_base58());
        assert_eq!(record.value, value);
        assert_eq!(record.set_by, kp.get_peer_id().to_base58());
    }

    #[test]
    fn test_put_get_host_record() {
        clear_env();
        let mut registry = ServiceInterface::new();
        let kp = KeyPair::generate_ed25519();
        let key = "some_key".to_string();
        let timestamp_created = 0u64;
        let current_timestamp = 100u64;
        let weight = 0;
        let pin = false;

        let key_id = register_key_checked(
            &mut registry,
            &kp,
            key,
            timestamp_created,
            current_timestamp,
            pin,
            weight,
        );
        let value = "some_value".to_string();
        let relay_id = vec!["some_relay".to_string()];
        let service_id = vec!["some_service_id".to_string()];
        let weight = 5u32;

        put_host_record_checked(
            &mut registry,
            &kp,
            key_id.clone(),
            value.clone(),
            relay_id.clone(),
            service_id.clone(),
            timestamp_created,
            current_timestamp,
            weight,
        );

        let records = get_records(&mut registry, key_id.clone(), current_timestamp);
        assert_eq!(records.len(), 1);
        let record = &records[0];
        assert_eq!(record.key_id, key_id);
        assert_eq!(record.relay_id, relay_id);
        assert_eq!(record.service_id, service_id);
        assert_eq!(record.set_by, kp.get_peer_id().to_base58());
        assert_eq!(record.peer_id, HOST_ID);
        assert_eq!(record.value, value);
        assert_eq!(record.set_by, kp.get_peer_id().to_base58());
    }
}

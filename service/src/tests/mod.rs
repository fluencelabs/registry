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
    use marine_test_env::registry::{DhtResult, Record, ServiceInterface};

    use crate::defaults::{
        CONFIG_FILE, DB_PATH, DEFAULT_STALE_VALUE_AGE, KEYS_TABLE_NAME, KEYS_TIMESTAMPS_TABLE_NAME,
        RECORDS_TABLE_NAME, TRUSTED_TIMESTAMP_FUNCTION_NAME, TRUSTED_TIMESTAMP_SERVICE_ID,
        TRUSTED_WEIGHT_FUNCTION_NAME, TRUSTED_WEIGHT_SERVICE_ID, VALUES_LIMIT,
    };
    use crate::error::ServiceError::{
        InvalidKeyTimestamp, InvalidTimestampTetraplet, InvalidWeightPeerId,
        KeyAlreadyExistsNewerTimestamp,
    };
    use crate::tests::tests::marine_test_env::registry::{Key, RegisterKeyResult, WeightResult};

    const HOST_ID: &str = "some_host_id";

    impl PartialEq for Key {
        fn eq(&self, other: &Self) -> bool {
            self.key_id == other.key_id
                && self.key == other.key
                && self.timestamp_created == other.timestamp_created
                && self.signature == other.signature
                && self.peer_id == other.peer_id
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

    fn register_key(
        registry: &mut ServiceInterface,
        kp: &KeyPair,
        key: String,
        timestamp_created: u64,
        current_timestamp: u64,
        pin: bool,
        weight: u32,
    ) -> RegisterKeyResult {
        let issuer_peer_id = kp.get_peer_id().to_base58();

        let key_bytes =
            registry.get_key_bytes(key.clone(), vec![issuer_peer_id.clone()], timestamp_created);
        let signature = kp.sign(&key_bytes).unwrap().to_vec().to_vec();
        let cp = CPWrapper::new(&issuer_peer_id)
            .add_weight_tetraplets(5)
            .add_timestamp_tetraplets(6);
        let weight = get_weight(issuer_peer_id.clone(), weight);
        registry.register_key_cp(
            key,
            vec![issuer_peer_id],
            timestamp_created,
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
    ) -> DhtResult {
        let issuer_peer_id = kp.get_peer_id().to_base58();
        let mut cp = CPWrapper::new(&issuer_peer_id);

        let record_bytes = registry.get_record_bytes_cp(
            key_id.clone(),
            value.clone(),
            relay_id.clone(),
            service_id.clone(),
            timestamp_created,
            cp.get(),
        );
        let signature = kp.sign(&record_bytes).unwrap().to_vec().to_vec();

        cp = cp.add_weight_tetraplets(6).add_timestamp_tetraplets(7);
        let weight = get_weight(issuer_peer_id.clone(), weight);
        registry.put_record_cp(
            key_id,
            value,
            relay_id,
            service_id,
            timestamp_created,
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
        let key = "some_key".to_string();
        let timestamp_created = 0u64;
        let current_timestamp = 100u64;
        let weight = get_weight(issuer_peer_id.clone(), 0);

        let key_bytes = registry.get_key_bytes_cp(key.clone(), vec![], timestamp_created, cp.get());
        let signature = key_bytes;

        cp = cp.add_weight_tetraplets(5).add_timestamp_tetraplets(6);
        let reg_key_result = registry.register_key_cp(
            key,
            vec![],
            timestamp_created,
            signature,
            false,
            weight,
            current_timestamp,
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
        let key = "some_key".to_string();
        let timestamp_created = 0u64;
        let current_timestamp = 100u64;
        let weight = get_weight(issuer_peer_id.clone(), 0);

        let key_bytes = registry.get_key_bytes_cp(key.clone(), vec![], timestamp_created, cp.get());
        let signature = kp.sign(&key_bytes).unwrap().to_vec().to_vec();

        cp = cp.add_timestamp_tetraplets(6);
        let reg_key_result = registry.register_key_cp(
            key,
            vec![],
            timestamp_created,
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
        let key = "some_key".to_string();
        let timestamp_created = 0u64;
        let current_timestamp = 100u64;
        let weight = get_weight(issuer_peer_id.clone(), 0);

        let key_bytes =
            registry.get_key_bytes(key.clone(), vec![issuer_peer_id.clone()], timestamp_created);
        let signature = kp.sign(&key_bytes).unwrap().to_vec().to_vec();

        let cp = CPWrapper::new(&issuer_peer_id).add_weight_tetraplets(5);
        let reg_key_result = registry.register_key_cp(
            key,
            vec![],
            timestamp_created,
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
        let key = "some_key".to_string();
        let timestamp_created = 0u64;
        let current_timestamp = 100u64;
        let weight = get_weight(invalid_peer_id.clone(), 0);

        let key_bytes = registry.get_key_bytes_cp(key.clone(), vec![], timestamp_created, cp.get());
        let signature = kp.sign(&key_bytes).unwrap().to_vec().to_vec();

        cp = cp.add_weight_tetraplets(5).add_timestamp_tetraplets(6);
        let reg_key_result = registry.register_key_cp(
            key,
            vec![],
            timestamp_created,
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
        let key = "some_key".to_string();
        let timestamp_created = 0u64;
        let current_timestamp = 100u64;
        let weight = 0;
        let pin = false;

        let issuer_peer_id = kp.get_peer_id().to_base58();

        let key_bytes =
            registry.get_key_bytes(key.clone(), vec![issuer_peer_id.clone()], timestamp_created);
        let signature = kp.sign(&key_bytes).unwrap().to_vec().to_vec();

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
        let expected_key = Key {
            key_id,
            key,
            peer_id: issuer_peer_id,
            timestamp_created,
            signature,
            pinned: pin,
            weight,
            timestamp_published: 0,
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
        assert_eq!(record.weight, weight);
        assert_eq!(record.value, value);
        assert_eq!(record.set_by, kp.get_peer_id().to_base58());
    }

    //
    // #[test]
    // fn register_key_empty_cp() {
    //     let mut registry = ServiceInterface::new();
    //     clear_env();
    //     let result = registry.register_key("some_key".to_string(), 123u64, false, 0u32);
    //     assert!(!result.success);
    //     assert_eq!(
    //         result.error,
    //         InvalidTimestampTetraplet("[]".to_string()).to_string()
    //     );
    // }
    //
    // #[test]
    // fn register_key_invalid_cp() {
    //     let mut registry = ServiceInterface::new();
    //     clear_env();
    //     let mut invalid_cp = CallParameters::default();
    //     invalid_cp.tetraplets.push(vec![]);
    //     invalid_cp.tetraplets.push(vec![SecurityTetraplet {
    //         peer_pk: "some peer_pk".to_string(),
    //         service_id: "INVALID SERVICE ID".to_string(),
    //         function_name: "INVALID FUNCTION NAME".to_string(),
    //         json_path: "some json path".to_string(),
    //     }]);
    //
    //     let result = registry.register_key_cp(
    //         "some_key".to_string(),
    //         123u64,
    //         false,
    //         8u32,
    //         invalid_cp.clone(),
    //     );
    //     assert!(!result.success);
    //     assert_eq!(
    //         result.error,
    //         InvalidTimestampTetraplet(format!("{:?}", invalid_cp.tetraplets[1][0])).to_string()
    //     );
    // }
    //
    // #[test]
    // fn register_key_twice_same_peer_id() {
    //     let mut registry = ServiceInterface::new();
    //     clear_env();
    //     let key = "some_key".to_string();
    //     let timestamp = 123u64;
    //     let weight = 8u32;
    //     let pin = false;
    //     let mut cp = get_correct_timestamp_cp(1);
    //     cp.init_peer_id = "some_peer_id".to_string();
    //
    //     register_key_and_check(&mut registry, &key, timestamp, pin, weight, &cp);
    //     register_key_and_check(&mut registry, &key, timestamp + 1, pin, weight, &cp);
    // }
    //
    // #[test]
    // fn register_key_twice_other_peer_id() {
    //     let mut registry = ServiceInterface::new();
    //     clear_env();
    //     let key = "some_key".to_string();
    //     let timestamp = 123u64;
    //     let weight = 8u32;
    //     let pin = false;
    //     let mut cp = get_correct_timestamp_cp(1);
    //     cp.init_peer_id = "some_peer_id".to_string();
    //     register_key_and_check(&mut registry, &key, timestamp, pin, weight, &cp);
    //
    //     cp.init_peer_id = "other_peer_id".to_string();
    //     let result = registry.register_key_cp(key.clone(), timestamp, pin, weight, cp);
    //     assert!(!result.success);
    //     assert_eq!(result.error, KeyAlreadyExists(key).to_string());
    // }
    //
    // #[test]
    // fn get_key_metadata_not_found() {
    //     let mut registry = ServiceInterface::new();
    //     clear_env();
    //     let key = "invalid_key".to_string();
    //     let result = registry.get_key_metadata_cp(key.clone(), 123u64, get_correct_timestamp_cp(1));
    //     assert!(!result.success);
    //     assert_eq!(result.error, f!("Requested key {key} does not exist"));
    // }
    //
    // #[test]
    // fn republish_key_not_exists() {
    //     let mut registry = ServiceInterface::new();
    //     clear_env();
    //
    //     let key = Key {
    //         key: "some_key".to_string(),
    //         peer_id: "some_peer".to_string(),
    //         timestamp_created: 0,
    //         pinned: false,
    //         weight: 8u32,
    //     };
    //
    //     republish_key_and_check(&mut registry, &key, 123u64, &get_correct_timestamp_cp(1));
    // }
    //
    // #[test]
    // fn republish_key_same_peer_id() {
    //     let mut registry = ServiceInterface::new();
    //     clear_env();
    //     let key_str = "some_key".to_string();
    //     let timestamp = 123u64;
    //     let weight = 8u32;
    //     let pin = false;
    //     let mut cp = get_correct_timestamp_cp(1);
    //     cp.init_peer_id = "some_peer_id".to_string();
    //     register_key_and_check(&mut registry, &key_str, timestamp, pin, weight, &cp);
    //
    //     let key = Key {
    //         key: key_str.clone(),
    //         peer_id: cp.init_peer_id,
    //         timestamp_created: timestamp + 1,
    //         pinned: false,
    //         weight: weight.clone(),
    //     };
    //
    //     republish_key_and_check(&mut registry, &key, 123123u64, &get_correct_timestamp_cp(1));
    // }
    //
    // #[test]
    // fn republish_key_other_peer_id() {
    //     let mut registry = ServiceInterface::new();
    //     clear_env();
    //     let key_str = "some_key".to_string();
    //     let timestamp = 123u64;
    //     let weight = 8u32;
    //     let pin = false;
    //     let mut cp = get_correct_timestamp_cp(1);
    //     cp.init_peer_id = "some_peer_id".to_string();
    //     register_key_and_check(&mut registry, &key_str, timestamp, pin, weight, &cp);
    //
    //     let key = Key {
    //         key: key_str.clone(),
    //         peer_id: "OTHER_PEER_ID".to_string(),
    //         timestamp_created: timestamp + 1,
    //         pinned: false,
    //         weight: weight.clone(),
    //     };
    //
    //     let result = registry.republish_key_cp(key, 123123u64, get_correct_timestamp_cp(1));
    //     assert!(!result.success);
    //     assert_eq!(result.error, KeyAlreadyExists(key_str).to_string());
    // }
    //
    // #[test]
    // fn put_value_empty_cp() {
    //     let mut registry = ServiceInterface::new();
    //     clear_env();
    //     let result = registry.put_value(
    //         "some_key".to_string(),
    //         "value".to_string(),
    //         123u64,
    //         vec![],
    //         vec![],
    //         8u32,
    //     );
    //     assert!(!result.success);
    //     assert_eq!(
    //         result.error,
    //         InvalidTimestampTetraplet("[]".to_string()).to_string()
    //     );
    // }
    //
    // #[test]
    // fn put_value_invalid_cp() {
    //     let mut registry = ServiceInterface::new();
    //     clear_env();
    //
    //     let mut invalid_cp = CallParameters::default();
    //     invalid_cp.tetraplets.push(vec![]);
    //     invalid_cp.tetraplets.push(vec![SecurityTetraplet {
    //         peer_pk: "some peer_pk".to_string(),
    //         service_id: "INVALID SERVICE ID".to_string(),
    //         function_name: "INVALID FUNCTION NAME".to_string(),
    //         json_path: "some json path".to_string(),
    //     }]);
    //
    //     let result = registry.put_value_cp(
    //         "some_key".to_string(),
    //         "value".to_string(),
    //         123u64,
    //         vec![],
    //         vec![],
    //         8u32,
    //         invalid_cp.clone(),
    //     );
    //     assert!(!result.success);
    //     assert_eq!(
    //         result.error,
    //         InvalidTimestampTetraplet(format!("{:?}", invalid_cp.tetraplets)).to_string()
    //     );
    // }
    //
    // #[test]
    // fn get_values_empty_cp() {
    //     let mut registry = ServiceInterface::new();
    //     clear_env();
    //     let result = registry.get_values("some_key".to_string(), 123u64);
    //     assert!(!result.success);
    //     assert_eq!(
    //         result.error,
    //         InvalidTimestampTetraplet("[]".to_string()).to_string()
    //     );
    // }
    //
    // #[test]
    // fn get_values_invalid_cp() {
    //     let mut registry = ServiceInterface::new();
    //     clear_env();
    //     let mut invalid_cp = CallParameters::default();
    //     invalid_cp.tetraplets.push(vec![]);
    //     invalid_cp.tetraplets.push(vec![SecurityTetraplet {
    //         peer_pk: "some peer_pk".to_string(),
    //         service_id: "INVALID SERVICE ID".to_string(),
    //         function_name: "INVALID FUNCTION NAME".to_string(),
    //         json_path: "some json path".to_string(),
    //     }]);
    //
    //     let result = registry.get_values_cp("some_key".to_string(), 123u64, invalid_cp.clone());
    //     assert!(!result.success);
    //     assert_eq!(
    //         result.error,
    //         InvalidTimestampTetraplet(format!("{:?}", invalid_cp.tetraplets[1][0])).to_string()
    //     );
    // }
    //
    // #[test]
    // fn get_values_empty() {
    //     let mut registry = ServiceInterface::new();
    //     clear_env();
    //
    //     let key = "some_key".to_string();
    //     register_key_and_check(
    //         &mut registry,
    //         &key,
    //         123u64,
    //         false,
    //         8u32,
    //         &get_correct_timestamp_cp(1),
    //     );
    //
    //     let result = registry.get_values_cp(key, 123u64, get_correct_timestamp_cp(1));
    //
    //     assert!(result.success, "{}", result.error);
    //
    //     assert_eq!(result.result.len(), 0);
    // }
    //
    // #[test]
    // fn get_values_key_not_exists() {
    //     let mut registry = ServiceInterface::new();
    //     clear_env();
    //
    //     let key = "invalid_key".to_string();
    //     let result = registry.get_values_cp(key.clone(), 123u64, get_correct_timestamp_cp(1));
    //
    //     assert!(!result.success);
    //     assert_eq!(result.error, f!("Requested key {key} does not exist"));
    //     assert_eq!(result.result.len(), 0);
    // }
    //
    // #[test]
    // fn put_value_key_not_exists() {
    //     let mut registry = ServiceInterface::new();
    //     clear_env();
    //     let key = "some_key".to_string();
    //     let result = registry.put_value_cp(
    //         key.clone(),
    //         "value".to_string(),
    //         123u64,
    //         vec![],
    //         vec![],
    //         8u32,
    //         get_correct_timestamp_cp(2),
    //     );
    //     assert!(!result.success);
    //     assert_eq!(result.error, f!("Requested key {key} does not exist"));
    // }
    //
    // #[test]
    // fn put_value() {
    //     let mut registry = ServiceInterface::new();
    //     clear_env();
    //
    //     let key = "some_key".to_string();
    //     let value = "some_value".to_string();
    //     let timestamp = 123u64;
    //     let weight = 8u32;
    //     let relay_id = "some_relay".to_string();
    //     let service_id = "some_service_id".to_string();
    //     let mut cp = get_correct_timestamp_cp(2);
    //     cp.init_peer_id = "some_peer_id".to_string();
    //
    //     register_key_and_check(
    //         &mut registry,
    //         &key,
    //         timestamp,
    //         false,
    //         8u32,
    //         &get_correct_timestamp_cp(1),
    //     );
    //     put_value_and_check(
    //         &mut registry,
    //         &key,
    //         &value,
    //         timestamp,
    //         &vec![relay_id.clone()],
    //         &vec![service_id.clone()],
    //         weight,
    //         &cp,
    //     );
    //
    //     let result = registry.get_values_cp(key, timestamp.clone(), get_correct_timestamp_cp(1));
    //
    //     assert!(result.success, "{}", result.error);
    //
    //     assert_eq!(result.result.len(), 1);
    //
    //     let record = &result.result[0];
    //     assert_eq!(record.value, value);
    //     assert_eq!(record.peer_id, cp.init_peer_id);
    //     assert_eq!(record.relay_id[0], relay_id);
    //     assert_eq!(record.service_id[0], service_id);
    //     assert_eq!(record.timestamp_created, timestamp);
    //     assert_eq!(record.weight, weight);
    // }
    //
    // #[test]
    // fn put_value_update() {
    //     let mut registry = ServiceInterface::new();
    //     clear_env();
    //     let key = "some_key".to_string();
    //     let value1 = "some_value".to_string();
    //     let timestamp = 123u64;
    //     let weight = 8u32;
    //     let relay_id = "some_relay".to_string();
    //     let service_id = "some_service_id".to_string();
    //
    //     let mut cp = get_correct_timestamp_cp(2);
    //     cp.init_peer_id = "some_peer_id".to_string();
    //
    //     register_key_and_check(
    //         &mut registry,
    //         &key,
    //         timestamp,
    //         false,
    //         8u32,
    //         &get_correct_timestamp_cp(1),
    //     );
    //     put_value_and_check(
    //         &mut registry,
    //         &key,
    //         &value1,
    //         timestamp,
    //         &vec![relay_id.clone()],
    //         &vec![service_id.clone()],
    //         weight,
    //         &cp,
    //     );
    //     let value2 = "other_value".to_string();
    //     put_value_and_check(
    //         &mut registry,
    //         &key,
    //         &value2,
    //         timestamp,
    //         &vec![relay_id.clone()],
    //         &vec![service_id.clone()],
    //         weight,
    //         &cp,
    //     );
    //
    //     let result = registry.get_values_cp(key, timestamp.clone(), get_correct_timestamp_cp(1));
    //
    //     assert!(result.success, "{}", result.error);
    //
    //     assert_eq!(result.result.len(), 1);
    //
    //     let record = &result.result[0];
    //     assert_eq!(record.value, value2);
    //     assert_eq!(record.peer_id, cp.init_peer_id);
    //     assert_eq!(record.relay_id[0], relay_id);
    //     assert_eq!(record.service_id[0], service_id);
    //     assert_eq!(record.timestamp_created, timestamp);
    //     assert_eq!(record.weight, weight);
    // }
    //
    // #[test]
    // fn put_value_limit() {
    //     let mut registry = ServiceInterface::new();
    //     clear_env();
    //     let key = "some_key".to_string();
    //     let value = "some_value".to_string();
    //     let timestamp = 123u64;
    //
    //     let put_value = |registry: &mut ServiceInterface, peer_id: &str, weight: u32| {
    //         let mut cp = get_correct_timestamp_cp(2);
    //         cp.init_peer_id = peer_id.to_string();
    //         put_value_and_check(
    //             registry,
    //             &key,
    //             &value,
    //             timestamp,
    //             &vec![],
    //             &vec![],
    //             weight,
    //             &cp,
    //         );
    //     };
    //
    //     register_key_and_check(
    //         &mut registry,
    //         &key,
    //         timestamp,
    //         false,
    //         8u32,
    //         &get_correct_timestamp_cp(1),
    //     );
    //
    //     let min_weight = 10u32;
    //     for i in 0..VALUES_LIMIT {
    //         put_value(&mut registry, &i.to_string(), min_weight + i as u32);
    //     }
    //
    //     // try to put value with smaller weight
    //     let smaller_weight = min_weight - 1;
    //     let mut cp = get_correct_timestamp_cp(2);
    //     cp.init_peer_id = "unique_peer_id1".to_string();
    //     let result = registry.put_value_cp(
    //         key.clone(),
    //         value.clone(),
    //         timestamp,
    //         vec![],
    //         vec![],
    //         smaller_weight,
    //         cp,
    //     );
    //     assert!(!result.success);
    //     assert_eq!(result.error, ValuesLimitExceeded(key.clone()).to_string());
    //
    //     // try to put value with bigger weight
    //     let bigger_weight = min_weight + 99999;
    //     let mut cp = get_correct_timestamp_cp(2);
    //     cp.init_peer_id = "unique_peer_id2".to_string();
    //     put_value_and_check(
    //         &mut registry,
    //         &key,
    //         &value,
    //         timestamp,
    //         &vec![],
    //         &vec![],
    //         bigger_weight,
    //         &cp,
    //     );
    //
    //     // try to put host value
    //     put_host_value_and_check(
    //         &mut registry,
    //         &key,
    //         &value,
    //         timestamp,
    //         &vec![],
    //         &vec![],
    //         0u32,
    //         &get_correct_timestamp_cp(2),
    //     );
    // }
    //
    // #[test]
    // fn put_multiple_values_for_key() {
    //     let mut registry = ServiceInterface::new();
    //     clear_env();
    //     let key = "some_key".to_string();
    //     let value = "some_value".to_string();
    //     let timestamp = 123u64;
    //     let weight = 8u32;
    //     let relay_id = vec!["some_relay".to_string()];
    //     let service_id = vec!["some_service_id".to_string()];
    //     let mut cp = get_correct_timestamp_cp(2);
    //     let peer1_id = "some_peer_id".to_string();
    //     let peer2_id = "other_peer_id".to_string();
    //
    //     register_key_and_check(
    //         &mut registry,
    //         &key,
    //         timestamp,
    //         false,
    //         8u32,
    //         &get_correct_timestamp_cp(1),
    //     );
    //
    //     cp.init_peer_id = peer1_id.clone();
    //     put_value_and_check(
    //         &mut registry,
    //         &key,
    //         &value,
    //         timestamp,
    //         &relay_id,
    //         &service_id,
    //         weight,
    //         &cp,
    //     );
    //
    //     cp.init_peer_id = peer2_id.clone();
    //     put_value_and_check(
    //         &mut registry,
    //         &key,
    //         &value,
    //         timestamp,
    //         &relay_id,
    //         &service_id,
    //         weight,
    //         &cp,
    //     );
    //
    //     let result = registry.get_values_cp(key, timestamp.clone(), get_correct_timestamp_cp(1));
    //
    //     assert!(result.success, "{}", result.error);
    //
    //     assert_eq!(result.result.len(), 2);
    //
    //     let record = &result.result[0];
    //     assert_eq!(record.value, value);
    //     assert_eq!(record.peer_id, peer2_id);
    //     assert_eq!(record.relay_id, relay_id);
    //     assert_eq!(record.service_id, service_id);
    //     assert_eq!(record.timestamp_created, timestamp);
    //     assert_eq!(record.weight, weight);
    //
    //     let record = &result.result[1];
    //     assert_eq!(record.value, value);
    //     assert_eq!(record.peer_id, peer1_id);
    //     assert_eq!(record.relay_id, relay_id);
    //     assert_eq!(record.service_id, service_id);
    //     assert_eq!(record.timestamp_created, timestamp);
    //     assert_eq!(record.weight, weight);
    // }
    //
    // #[test]
    // fn clear_expired_empty_cp() {
    //     let mut registry = ServiceInterface::new();
    //     clear_env();
    //
    //     let result = registry.clear_expired(124u64);
    //     assert!(!result.success);
    //     assert_eq!(
    //         result.error,
    //         InvalidTimestampTetraplet("[]".to_string()).to_string()
    //     );
    // }
    //
    // #[test]
    // fn clear_expired_invalid_cp() {
    //     let mut registry = ServiceInterface::new();
    //     clear_env();
    //
    //     let mut invalid_cp = CallParameters::default();
    //     invalid_cp.tetraplets.push(vec![]);
    //     invalid_cp.tetraplets.push(vec![SecurityTetraplet {
    //         peer_pk: "some peer_pk".to_string(),
    //         service_id: "INVALID SERVICE ID".to_string(),
    //         function_name: "INVALID FUNCTION NAME".to_string(),
    //         json_path: "some json path".to_string(),
    //     }]);
    //
    //     let result = registry.clear_expired_cp(124u64, invalid_cp.clone());
    //     assert!(!result.success);
    //     assert_eq!(
    //         result.error,
    //         InvalidTimestampTetraplet(format!("{:?}", invalid_cp.tetraplets)).to_string()
    //     );
    // }
    //
    // #[test]
    // fn clear_expired_empty() {
    //     let mut registry = ServiceInterface::new();
    //     clear_env();
    //     let result = registry.clear_expired_cp(124u64, get_correct_timestamp_cp(0));
    //
    //     assert!(result.success, "{}", result.error);
    //     assert_eq!(result.count_keys + result.count_values, 0);
    // }
    //
    // #[test]
    // fn clear_expired_key_without_values() {
    //     let mut registry = ServiceInterface::new();
    //     clear_env();
    //     let key = "some_key".to_string();
    //     let expired_timestamp = SystemTime::now()
    //         .duration_since(SystemTime::UNIX_EPOCH)
    //         .unwrap()
    //         .as_secs();
    //     register_key_and_check(
    //         &mut registry,
    //         &key,
    //         expired_timestamp,
    //         false,
    //         8u32,
    //         &get_correct_timestamp_cp(1),
    //     );
    //
    //     let result = registry.clear_expired_cp(
    //         expired_timestamp + DEFAULT_EXPIRED_VALUE_AGE,
    //         get_correct_timestamp_cp(0),
    //     );
    //
    //     assert!(result.success, "{}", result.error);
    //
    //     assert_eq!(result.count_keys, 1);
    //     assert_eq!(result.count_values, 0);
    //
    //     let result = registry.get_key_metadata_cp(key.clone(), 123u64, get_correct_timestamp_cp(1));
    //     assert!(!result.success);
    //     assert_eq!(result.error, f!("Requested key {key} does not exist"));
    // }
    //
    // #[test]
    // fn clear_expired_host_key() {
    //     let mut registry = ServiceInterface::new();
    //     clear_env();
    //     let key = "some_key".to_string();
    //     let expired_timestamp = SystemTime::now()
    //         .duration_since(SystemTime::UNIX_EPOCH)
    //         .unwrap()
    //         .as_secs();
    //     register_key_and_check(
    //         &mut registry,
    //         &key,
    //         expired_timestamp,
    //         true,
    //         8u32,
    //         &get_correct_timestamp_cp(1),
    //     );
    //     put_value_and_check(
    //         &mut registry,
    //         &key,
    //         &"some_value".to_string(),
    //         expired_timestamp,
    //         &vec![],
    //         &vec![],
    //         8u32,
    //         &get_correct_timestamp_cp(2),
    //     );
    //
    //     let result = registry.clear_expired_cp(
    //         expired_timestamp + DEFAULT_EXPIRED_VALUE_AGE,
    //         get_correct_timestamp_cp(0),
    //     );
    //
    //     assert!(result.success, "{}", result.error);
    //
    //     assert_eq!(result.count_keys, 0);
    //     assert_eq!(result.count_values, 1);
    // }
    //
    // #[test]
    // fn clear_expired_host_value() {
    //     let mut registry = ServiceInterface::new();
    //     clear_env();
    //     let key = "some_key".to_string();
    //     let expired_timestamp = SystemTime::now()
    //         .duration_since(SystemTime::UNIX_EPOCH)
    //         .unwrap()
    //         .as_secs();
    //     register_key_and_check(
    //         &mut registry,
    //         &key,
    //         expired_timestamp,
    //         false,
    //         8u32,
    //         &get_correct_timestamp_cp(1),
    //     );
    //     put_host_value_and_check(
    //         &mut registry,
    //         &key,
    //         &"some_value".to_string(),
    //         expired_timestamp,
    //         &vec![],
    //         &vec![],
    //         8u32,
    //         &get_correct_timestamp_cp(2),
    //     );
    //
    //     let result = registry.clear_expired_cp(
    //         expired_timestamp + DEFAULT_EXPIRED_VALUE_AGE,
    //         get_correct_timestamp_cp(0),
    //     );
    //
    //     assert!(result.success, "{}", result.error);
    //
    //     assert_eq!(result.count_keys, 0);
    //     assert_eq!(result.count_values, 0);
    // }
    //
    // #[test]
    // fn clear_expired_key_with_values() {
    //     let mut registry = ServiceInterface::new();
    //     clear_env();
    //     let key = "some_key".to_string();
    //     let expired_timestamp = SystemTime::now()
    //         .duration_since(SystemTime::UNIX_EPOCH)
    //         .unwrap()
    //         .as_secs();
    //     register_key_and_check(
    //         &mut registry,
    //         &key,
    //         expired_timestamp,
    //         false,
    //         8u32,
    //         &get_correct_timestamp_cp(1),
    //     );
    //     put_value_and_check(
    //         &mut registry,
    //         &key,
    //         &"some_value".to_string(),
    //         expired_timestamp,
    //         &vec![],
    //         &vec![],
    //         8u32,
    //         &get_correct_timestamp_cp(2),
    //     );
    //
    //     let result = registry.clear_expired_cp(
    //         expired_timestamp + DEFAULT_EXPIRED_VALUE_AGE,
    //         get_correct_timestamp_cp(0),
    //     );
    //
    //     assert!(result.success, "{}", result.error);
    //
    //     assert_eq!(result.count_keys, 1);
    //     assert_eq!(result.count_values, 1);
    //
    //     let result = registry.get_key_metadata_cp(key.clone(), 123u64, get_correct_timestamp_cp(1));
    //     assert!(!result.success);
    //     assert_eq!(result.error, f!("Requested key {key} does not exist"));
    //
    //     let result = registry.get_values_cp(key.clone(), 123u64, get_correct_timestamp_cp(1));
    //
    //     assert!(!result.success);
    //     assert_eq!(result.error, f!("Requested key {key} does not exist"));
    // }
    //
    // #[test]
    // fn clear_expired_change_timeout() {
    //     let mut registry = ServiceInterface::new();
    //     clear_env();
    //     let key = "some_key".to_string();
    //     let expired_timestamp = SystemTime::now()
    //         .duration_since(SystemTime::UNIX_EPOCH)
    //         .unwrap()
    //         .as_secs();
    //
    //     register_key_and_check(
    //         &mut registry,
    //         &key,
    //         expired_timestamp,
    //         false,
    //         8u32,
    //         &get_correct_timestamp_cp(1),
    //     );
    //     put_value_and_check(
    //         &mut registry,
    //         &key,
    //         &"some_value".to_string(),
    //         expired_timestamp,
    //         &vec![],
    //         &vec![],
    //         8u32,
    //         &get_correct_timestamp_cp(2),
    //     );
    //
    //     let new_expired_timeout = DEFAULT_EXPIRED_VALUE_AGE - 100u64;
    //     registry.set_expired_timeout(new_expired_timeout.clone());
    //     let result = registry.clear_expired_cp(
    //         expired_timestamp + new_expired_timeout,
    //         get_correct_timestamp_cp(0),
    //     );
    //
    //     assert!(result.success, "{}", result.error);
    //
    //     assert_eq!(result.count_keys, 1);
    //     assert_eq!(result.count_values, 1);
    //
    //     let result = registry.get_key_metadata_cp(key.clone(), 123u64, get_correct_timestamp_cp(1));
    //     assert!(!result.success);
    //     assert_eq!(result.error, f!("Requested key {key} does not exist"));
    //
    //     let result = registry.get_values_cp(key.clone(), 123u64, get_correct_timestamp_cp(1));
    //
    //     assert!(!result.success);
    //     assert_eq!(result.error, f!("Requested key {key} does not exist"));
    // }
    //
    // #[test]
    // fn evict_stale_empty_cp() {
    //     let mut registry = ServiceInterface::new();
    //     clear_env();
    //
    //     let result = registry.evict_stale(124u64);
    //     assert!(!result.success);
    //     assert_eq!(
    //         result.error,
    //         InvalidTimestampTetraplet("[]".to_string()).to_string()
    //     );
    // }
    //
    // #[test]
    // fn evict_stale_invalid_cp() {
    //     let mut registry = ServiceInterface::new();
    //     clear_env();
    //
    //     let mut invalid_cp = CallParameters::default();
    //     invalid_cp.tetraplets.push(vec![]);
    //     invalid_cp.tetraplets.push(vec![SecurityTetraplet {
    //         peer_pk: "some peer_pk".to_string(),
    //         service_id: "INVALID SERVICE ID".to_string(),
    //         function_name: "INVALID FUNCTION NAME".to_string(),
    //         json_path: "some json path".to_string(),
    //     }]);
    //
    //     let result = registry.evict_stale_cp(124u64, invalid_cp.clone());
    //     assert!(!result.success);
    //     assert_eq!(
    //         result.error,
    //         InvalidTimestampTetraplet(format!("{:?}", invalid_cp.tetraplets)).to_string()
    //     );
    // }
    //
    // #[test]
    // fn evict_stale_empty() {
    //     let mut registry = ServiceInterface::new();
    //     clear_env();
    //     let result = registry.evict_stale_cp(124u64, get_correct_timestamp_cp(0));
    //     assert!(result.success, "{}", result.error);
    //
    //     assert_eq!(result.results.len(), 0);
    // }
    //
    // #[test]
    // fn evict_stale_key_without_values() {
    //     let mut registry = ServiceInterface::new();
    //     clear_env();
    //     let key = "some_key".to_string();
    //     let stale_timestamp = 0u64;
    //     register_key_and_check(
    //         &mut registry,
    //         &key,
    //         stale_timestamp,
    //         false,
    //         8u32,
    //         &get_correct_timestamp_cp(1),
    //     );
    //
    //     let result = registry.evict_stale_cp(
    //         stale_timestamp + DEFAULT_STALE_VALUE_AGE,
    //         get_correct_timestamp_cp(0),
    //     );
    //
    //     assert!(result.success, "{}", result.error);
    //
    //     assert_eq!(result.results.len(), 1);
    //     let item = &result.results[0];
    //     assert_eq!(item.key.key, key);
    //     assert_eq!(item.records.len(), 0);
    //
    //     let result = registry.get_key_metadata_cp(key.clone(), 123u64, get_correct_timestamp_cp(1));
    //     assert!(!result.success);
    //     assert_eq!(result.error, f!("Requested key {key} does not exist"));
    // }
    //
    // #[test]
    // fn evict_stale_key_with_values() {
    //     let mut registry = ServiceInterface::new();
    //     clear_env();
    //     let key = "some_key".to_string();
    //     let value = "some_value".to_string();
    //     let stale_timestamp = 0u64;
    //     register_key_and_check(
    //         &mut registry,
    //         &key,
    //         stale_timestamp,
    //         false,
    //         8u32,
    //         &get_correct_timestamp_cp(1),
    //     );
    //     put_value_and_check(
    //         &mut registry,
    //         &key,
    //         &value,
    //         stale_timestamp,
    //         &vec![],
    //         &vec![],
    //         8u32,
    //         &get_correct_timestamp_cp(2),
    //     );
    //
    //     let result = registry.evict_stale_cp(
    //         stale_timestamp + DEFAULT_STALE_VALUE_AGE,
    //         get_correct_timestamp_cp(0),
    //     );
    //
    //     assert!(result.success, "{}", result.error);
    //
    //     assert_eq!(result.results.len(), 1);
    //
    //     let item = &result.results[0];
    //     assert_eq!(item.key.key, key);
    //     assert_eq!(item.records.len(), 1);
    //
    //     let record = &item.records[0];
    //     assert_eq!(record.value, value);
    //     assert_eq!(record.timestamp_created, stale_timestamp);
    //
    //     let result = registry.get_key_metadata_cp(key.clone(), 123u64, get_correct_timestamp_cp(1));
    //     assert!(!result.success);
    //     assert_eq!(result.error, f!("Requested key {key} does not exist"));
    //
    //     let result = registry.get_values_cp(key.clone(), 123u64, get_correct_timestamp_cp(1));
    //
    //     assert!(!result.success);
    //     assert_eq!(result.error, f!("Requested key {key} does not exist"));
    // }
    //
    // #[test]
    // fn merge_test() {
    //     let mut registry = ServiceInterface::new();
    //
    //     let peer_id = "some_peer_id".to_string();
    //     let stale_record = Record {
    //         value: "stale".to_string(),
    //         peer_id: peer_id.clone(),
    //         relay_id: vec![],
    //         service_id: vec![],
    //         timestamp_created: 123u64,
    //         set_by: peer_id.clone(),
    //         weight: 8u32,
    //     };
    //
    //     let new_record = Record {
    //         value: "new".to_string(),
    //         peer_id: peer_id.clone(),
    //         relay_id: vec![],
    //         service_id: vec![],
    //         timestamp_created: stale_record.timestamp_created + 9999u64,
    //         set_by: peer_id.clone(),
    //         weight: 8u32,
    //     };
    //
    //     let result = registry.merge(vec![vec![stale_record.clone()], vec![new_record.clone()]]);
    //
    //     assert_eq!(result.result.len(), 1);
    //     let record = &result.result[0];
    //     assert_eq!(record.value, new_record.value);
    //     assert_eq!(record.timestamp_created, new_record.timestamp_created);
    //
    //     let result = registry.merge_two(vec![stale_record.clone()], vec![new_record.clone()]);
    //
    //     assert_eq!(result.result.len(), 1);
    //     let record = &result.result[0];
    //     assert_eq!(record.value, new_record.value);
    //     assert_eq!(record.timestamp_created, new_record.timestamp_created);
    // }
    //
    // #[test]
    // fn merge_test_different_peer_ids() {
    //     let mut registry = ServiceInterface::new();
    //
    //     let peer_id1 = "some_peer_id1".to_string();
    //     let peer_id2 = "some_peer_id2".to_string();
    //     let record1 = Record {
    //         value: "value1".to_string(),
    //         peer_id: peer_id1.clone(),
    //         relay_id: vec![],
    //         service_id: vec![],
    //         timestamp_created: 123u64,
    //         set_by: peer_id1.clone(),
    //         weight: 8u32,
    //     };
    //
    //     let record2 = Record {
    //         value: "value2".to_string(),
    //         peer_id: peer_id2.clone(),
    //         relay_id: vec![],
    //         service_id: vec![],
    //         timestamp_created: record1.timestamp_created + 9999u64,
    //         set_by: peer_id2.clone(),
    //         weight: 8u32,
    //     };
    //
    //     let result = registry.merge_two(vec![record1], vec![record2]);
    //
    //     assert_eq!(result.result.len(), 2);
    // }
    //
    // // test repeats initTopicAndSubscribeNode method from pubsub api
    // #[test]
    // fn init_topic_and_subscribe_node_test() {
    //     let mut registry = ServiceInterface::new();
    //     clear_env();
    //     let topic = "some_key".to_string();
    //     let timestamp = 123u64;
    //     let weight = 8u32;
    //     let pin = false;
    //     let value = "some_value".to_string();
    //     let subscriber_peer_id = "some_peer_id".to_string();
    //     let mut cp = get_correct_timestamp_cp(1);
    //     cp.init_peer_id = subscriber_peer_id.clone();
    //
    //     // === init topic and subscribe to it
    //
    //     // register topic
    //     register_key_and_check(&mut registry, &topic, timestamp, pin, weight, &cp);
    //
    //     let mut cp = get_correct_timestamp_cp(2);
    //     cp.init_peer_id = subscriber_peer_id.clone();
    //
    //     // make a subscription
    //     let result = put_host_value_and_check(
    //         &mut registry,
    //         &topic,
    //         &value,
    //         timestamp,
    //         &vec![],
    //         &vec![],
    //         0u32,
    //         &cp,
    //     );
    //     assert!(result.success, "{}", result.error);
    //
    //     // clear db to imitate switching to neighbor
    //     let mut registry = ServiceInterface::new();
    //     clear_env();
    //
    //     // === notify neighbor about subscription
    //
    //     let mut cp = get_correct_timestamp_cp(1);
    //     cp.init_peer_id = subscriber_peer_id.clone();
    //
    //     // register topic on neighbor
    //     register_key_and_check(&mut registry, &topic, timestamp, pin, weight, &cp);
    //     let mut cp = get_correct_timestamp_cp(1);
    //     cp.tetraplets[0] = vec![SecurityTetraplet {
    //         peer_pk: HOST_ID.to_string(),
    //         service_id: "aqua-dht".to_string(),
    //         function_name: "put_host_value".to_string(),
    //         json_path: "".to_string(),
    //     }];
    //     cp.init_peer_id = subscriber_peer_id.clone();
    //
    //     // leave record about subscription
    //     let result = registry.propagate_host_value_cp(result, timestamp, weight.clone(), cp);
    //     assert!(result.success, "{}", result.error);
    //
    //     // check subscription (mimics findSubscribers but for one node without merging)
    //     let result = registry.get_values_cp(topic, 123u64, get_correct_timestamp_cp(1));
    //
    //     assert!(result.success, "{}", result.error);
    //     assert_eq!(result.result.len(), 1);
    //     let record = result.result[0].clone();
    //     assert_eq!(record.value, value);
    //     assert_eq!(record.peer_id, HOST_ID);
    //     assert_eq!(record.set_by, subscriber_peer_id);
    // }
    //

    //
    // #[test]
    // pub fn sql_injection_test() {
    //     let mut registry = ServiceInterface::new();
    //     clear_env();
    //     let key = "blabla".to_string();
    //     let injection_key =
    //         f!("{key}', '123', '123', 'abc', '0', '0'); DELETE FROM TABLE {KEYS_TABLE_NAME};");
    //
    //     let result = registry.register_key_cp(
    //         injection_key.clone(),
    //         123u64,
    //         false,
    //         0u32,
    //         get_correct_timestamp_cp(1),
    //     );
    //     assert!(result.success, "{}", result.error);
    //
    //     let result = registry.get_key_metadata_cp(
    //         injection_key.clone(),
    //         123u64,
    //         get_correct_timestamp_cp(1),
    //     );
    //     assert!(result.success, "{}", result.error);
    //     assert_eq!(result.key.key, injection_key);
    // }
}

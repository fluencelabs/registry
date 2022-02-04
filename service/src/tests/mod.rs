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
#![allow(module_inception)]
#[cfg(test)]
mod tests {
    use fluence_keypair::KeyPair;
    use std::fs;
    use std::time::SystemTime;

    use marine_rs_sdk::{CallParameters, SecurityTetraplet};
    use rusqlite::Connection;
    marine_rs_sdk_test::include_test_env!("/marine_test_env.rs");
    use marine_test_env::aqua_dht::{DhtResult, Key, Record, ServiceInterface};

    use crate::defaults::{
        CONFIG_FILE, DB_PATH, DEFAULT_EXPIRED_VALUE_AGE, DEFAULT_STALE_VALUE_AGE, KEYS_TABLE_NAME,
        KEYS_TIMESTAMPS_TABLE_NAME, RECORDS_TABLE_NAME, TRUSTED_TIMESTAMP_FUNCTION_NAME,
        TRUSTED_TIMESTAMP_SERVICE_ID, TRUSTED_WEIGHT_FUNCTION_NAME, TRUSTED_WEIGHT_SERVICE_ID,
        VALUES_LIMIT,
    };
    use crate::error::ServiceError::{
        InvalidKeyTimestamp, InvalidTimestampTetraplet, InvalidWeightPeerId,
        KeyAlreadyExistsNewerTimestamp, ValuesLimitExceeded,
    };
    use crate::tests::tests::marine_test_env::aqua_dht::WeightResult;

    const HOST_ID: &str = "some_host_id";

    fn clear_env() {
        let connection = Connection::open(DB_PATH).unwrap();

        connection
            .execute(f!("DELETE FROM {KEYS_TABLE_NAME}").as_str(), [])
            .unwrap();
        connection
            .execute(f!("DELETE FROM {KEYS_TIMESTAMPS_TABLE_NAME}").as_str(), [])
            .unwrap();
        connection
            .execute(f!("DELETE FROM {RECORDS_TABLE_NAME}").as_str(), [])
            .unwrap();

        if fs::metadata(CONFIG_FILE).is_ok() {
            fs::remove_file(CONFIG_FILE).unwrap();
        }
    }

    fn get_default_cp(init_peer_id: String) -> CallParameters {
        CallParameters {
            init_peer_id,
            service_id: "".to_string(),
            service_creator_peer_id: "".to_string(),
            host_id: HOST_ID.to_string(),
            particle_id: "".to_string(),
            tetraplets: vec![],
        }
    }

    fn add_timestamp_tetraplets(cp: &mut CallParameters, arg_number: usize) {
        if cp.tetraplets.len() <= arg_number {
            cp.tetraplets.resize(arg_number + 1, vec![]);
        }

        cp.tetraplets[arg_number] = vec![SecurityTetraplet {
            peer_pk: HOST_ID.to_string(),
            service_id: TRUSTED_TIMESTAMP_SERVICE_ID.to_string(),
            function_name: TRUSTED_TIMESTAMP_FUNCTION_NAME.to_string(),
            json_path: "".to_string(),
        }];
    }

    fn add_weight_tetraplets(cp: &mut CallParameters, arg_number: usize) {
        if cp.tetraplets.len() < arg_number {
            cp.tetraplets.resize(arg_number + 1, vec![]);
        }

        cp.tetraplets[arg_number] = vec![SecurityTetraplet {
            peer_pk: HOST_ID.to_string(),
            service_id: TRUSTED_WEIGHT_SERVICE_ID.to_string(),
            function_name: TRUSTED_WEIGHT_FUNCTION_NAME.to_string(),
            json_path: "".to_string(),
        }];
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
    ) -> DhtResult {
        let issuer_peer_id = kp.get_peer_id().to_base58();

        let key_bytes =
            registry.get_key_bytes(key.clone(), vec![issuer_peer_id.clone()], timestamp_created);
        let signature = kp.sign(&key_bytes).unwrap().to_vec().to_vec();

        let mut cp = get_default_cp(issuer_peer_id.clone());
        add_weight_tetraplets(&mut cp, 5);
        add_timestamp_tetraplets(&mut cp, 6);
        let weight = get_weight(issuer_peer_id.clone(), weight);
        registry.register_key_cp(
            key,
            vec![issuer_peer_id],
            timestamp_created,
            signature,
            pin,
            weight,
            current_timestamp,
            cp,
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
        assert!(result.success, result.error);
        // result.key_id
        "".to_string()
    }

    // fn get_key_metadata(
    //     registry: &mut ServiceInterface,
    //     key: String,
    //     peer_id: String,
    //     current_timestamp: String,
    // ) {
    //     let mut cp = get_default_cp("peer_id".to_string());
    //     add_timestamp_tetraplets(&mut cp)
    // }

    #[test]
    fn register_key_invalid_signature() {
        let mut registry = ServiceInterface::new();
        let kp = KeyPair::generate_ed25519();
        let issuer_peer_id = kp.get_peer_id().to_base58();
        let mut cp = get_default_cp(issuer_peer_id.clone());
        let key = "some_key".to_string();
        let timestamp_created = 0u64;
        let current_timestamp = 100u64;
        let weight = get_weight(issuer_peer_id.clone(), 0);

        let key_bytes =
            registry.get_key_bytes_cp(key.clone(), vec![], timestamp_created, cp.clone());
        let signature = key_bytes;

        add_weight_tetraplets(&mut cp, 5);
        add_timestamp_tetraplets(&mut cp, 6);
        let reg_key_result = registry.register_key_cp(
            key,
            vec![],
            timestamp_created,
            signature,
            false,
            weight,
            current_timestamp,
            cp,
        );
        assert!(!reg_key_result.success);
    }

    #[test]
    fn register_key_invalid_weight_tetraplet() {
        let mut registry = ServiceInterface::new();
        let kp = KeyPair::generate_ed25519();
        let issuer_peer_id = kp.get_peer_id().to_base58();
        let mut cp = get_default_cp(issuer_peer_id.clone());
        let key = "some_key".to_string();
        let timestamp_created = 0u64;
        let current_timestamp = 100u64;
        let weight = get_weight(issuer_peer_id.clone(), 0);

        let key_bytes =
            registry.get_key_bytes_cp(key.clone(), vec![], timestamp_created, cp.clone());
        let signature = kp.sign(&key_bytes).unwrap().to_vec().to_vec();

        add_timestamp_tetraplets(&mut cp, 6);
        let reg_key_result = registry.register_key_cp(
            key,
            vec![],
            timestamp_created,
            signature,
            false,
            weight,
            current_timestamp,
            cp,
        );
        assert!(!reg_key_result.success);
    }

    #[test]
    fn register_key_invalid_weight_peer_id() {
        let mut registry = ServiceInterface::new();
        let kp = KeyPair::generate_ed25519();
        let issuer_peer_id = kp.get_peer_id().to_base58();
        let invalid_peer_id = "INVALID_PEER_ID".to_string();
        let mut cp = get_default_cp(issuer_peer_id.clone());
        let key = "some_key".to_string();
        let timestamp_created = 0u64;
        let current_timestamp = 100u64;
        let weight = get_weight(invalid_peer_id.clone(), 0);

        let key_bytes =
            registry.get_key_bytes_cp(key.clone(), vec![], timestamp_created, cp.clone());
        let signature = kp.sign(&key_bytes).unwrap().to_vec().to_vec();

        add_weight_tetraplets(&mut cp, 5);
        add_timestamp_tetraplets(&mut cp, 6);
        let reg_key_result = registry.register_key_cp(
            key,
            vec![],
            timestamp_created,
            signature,
            false,
            weight,
            current_timestamp,
            cp,
        );
        assert!(!reg_key_result.success);
        assert_eq!(
            reg_key_result.error,
            InvalidWeightPeerId(issuer_peer_id, invalid_peer_id).to_string()
        );
    }

    #[test]
    fn register_key_correct() {
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

        assert!(result.success, result.error)
    }

    #[test]
    fn register_key_older_timestamp() {
        let mut registry = ServiceInterface::new();
        let kp = KeyPair::generate_ed25519();
        let key = "some_key".to_string();
        let timestamp_created_first = 100u64;
        let timestamp_created_second = timestamp_created_first - 10u64;
        let current_timestamp = 1000u64;
        let weight = 0;
        let pin = false;

        let result_first = register_key(
            &mut registry,
            &kp,
            key.clone(),
            timestamp_created_first,
            current_timestamp,
            pin,
            weight,
        );
        assert!(result_first.success, result_first.error);

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
    fn get_key_metadata() {
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

        let mut cp = get_default_cp(issuer_peer_id.clone());
        add_timestamp_tetraplets(&mut cp, 1);
        let result = registry.get_key_metadata_cp(key_id.clone(), current_timestamp, cp);
        assert!(result.success, result.error);
        let key = &result.key;
    }
    //
    // fn put_host_value_and_check(
    //     aqua_dht: &mut ServiceInterface,
    //     key: &str,
    //     value: &str,
    //     timestamp: u64,
    //     relay_id: &Vec<String>,
    //     service_id: &Vec<String>,
    //     weight: u32,
    //     cp: &CallParameters,
    // ) -> PutHostValueResult {
    //     let result = aqua_dht.put_host_value_cp(
    //         key.to_string(),
    //         value.to_string(),
    //         timestamp.clone(),
    //         relay_id.clone(),
    //         service_id.clone(),
    //         weight.clone(),
    //         cp.clone(),
    //     );
    //
    //     assert!(result.success, "{}", result.error);
    //     result
    // }
    //
    // fn put_value_and_check(
    //     aqua_dht: &mut ServiceInterface,
    //     key: &str,
    //     value: &str,
    //     timestamp: u64,
    //     relay_id: &Vec<String>,
    //     service_id: &Vec<String>,
    //     weight: u32,
    //     cp: &CallParameters,
    // ) -> DhtResult {
    //     let result = aqua_dht.put_value_cp(
    //         key.to_string(),
    //         value.to_string(),
    //         timestamp.clone(),
    //         relay_id.clone(),
    //         service_id.clone(),
    //         weight.clone(),
    //         cp.clone(),
    //     );
    //
    //     assert!(result.success, "{}", result.error);
    //     result
    // }
    //
    // fn check_key_metadata(
    //     aqua_dht: &mut ServiceInterface,
    //     key: &str,
    //     timestamp: u64,
    //     peer_id: &str,
    //     current_timestamp: u64,
    //     pinned: bool,
    //     weight: u32,
    //     cp: &CallParameters,
    // ) {
    //     let result =
    //         aqua_dht.get_key_metadata_cp(key.to_string(), current_timestamp.clone(), cp.clone());
    //     assert!(result.success, "{}", result.error);
    //     assert_eq!(result.key.key, key.clone());
    //     assert_eq!(result.key.peer_id, peer_id.clone());
    //     assert_eq!(result.key.timestamp_created, timestamp);
    //     assert_eq!(result.key.pinned, pinned);
    //     assert_eq!(result.key.weight, weight);
    // }
    //
    // fn register_key_and_check(
    //     aqua_dht: &mut ServiceInterface,
    //     key: &str,
    //     timestamp: u64,
    //     pin: bool,
    //     weight: u32,
    //     cp: &CallParameters,
    // ) {
    //     let result = aqua_dht.register_key_cp(
    //         key.to_string(),
    //         timestamp.clone(),
    //         pin.clone(),
    //         weight.clone(),
    //         cp.clone(),
    //     );
    //
    //     assert!(result.success, "{}", result.error);
    //
    //     check_key_metadata(
    //         aqua_dht,
    //         key,
    //         timestamp,
    //         &cp.init_peer_id,
    //         timestamp,
    //         pin,
    //         weight,
    //         cp,
    //     );
    // }
    //
    // fn republish_key_and_check(
    //     aqua_dht: &mut ServiceInterface,
    //     key: &Key,
    //     timestamp: u64,
    //     cp: &CallParameters,
    // ) {
    //     let result = aqua_dht.republish_key_cp(key.clone(), timestamp, cp.clone());
    //
    //     assert!(result.success, "{}", result.error);
    //
    //     check_key_metadata(
    //         aqua_dht,
    //         &key.key,
    //         key.timestamp_created,
    //         &key.peer_id,
    //         timestamp,
    //         false,
    //         key.weight,
    //         cp,
    //     );
    // }

    // #[test]
    // fn register_key() {
    //     let mut aqua_dht = ServiceInterface::new();
    //     clear_env();
    //     register_key_and_check(
    //         &mut aqua_dht,
    //         &"some_key".to_string(),
    //         123u64,
    //         false,
    //         0u32,
    //         &get_correct_timestamp_cp(1),
    //     );
    // }
    //
    // #[test]
    // fn register_key_empty_cp() {
    //     let mut aqua_dht = ServiceInterface::new();
    //     clear_env();
    //     let result = aqua_dht.register_key("some_key".to_string(), 123u64, false, 0u32);
    //     assert!(!result.success);
    //     assert_eq!(
    //         result.error,
    //         InvalidTimestampTetraplet("[]".to_string()).to_string()
    //     );
    // }
    //
    // #[test]
    // fn register_key_invalid_cp() {
    //     let mut aqua_dht = ServiceInterface::new();
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
    //     let result = aqua_dht.register_key_cp(
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
    //     let mut aqua_dht = ServiceInterface::new();
    //     clear_env();
    //     let key = "some_key".to_string();
    //     let timestamp = 123u64;
    //     let weight = 8u32;
    //     let pin = false;
    //     let mut cp = get_correct_timestamp_cp(1);
    //     cp.init_peer_id = "some_peer_id".to_string();
    //
    //     register_key_and_check(&mut aqua_dht, &key, timestamp, pin, weight, &cp);
    //     register_key_and_check(&mut aqua_dht, &key, timestamp + 1, pin, weight, &cp);
    // }
    //
    // #[test]
    // fn register_key_twice_other_peer_id() {
    //     let mut aqua_dht = ServiceInterface::new();
    //     clear_env();
    //     let key = "some_key".to_string();
    //     let timestamp = 123u64;
    //     let weight = 8u32;
    //     let pin = false;
    //     let mut cp = get_correct_timestamp_cp(1);
    //     cp.init_peer_id = "some_peer_id".to_string();
    //     register_key_and_check(&mut aqua_dht, &key, timestamp, pin, weight, &cp);
    //
    //     cp.init_peer_id = "other_peer_id".to_string();
    //     let result = aqua_dht.register_key_cp(key.clone(), timestamp, pin, weight, cp);
    //     assert!(!result.success);
    //     assert_eq!(result.error, KeyAlreadyExists(key).to_string());
    // }
    //
    // #[test]
    // fn get_key_metadata_not_found() {
    //     let mut aqua_dht = ServiceInterface::new();
    //     clear_env();
    //     let key = "invalid_key".to_string();
    //     let result = aqua_dht.get_key_metadata_cp(key.clone(), 123u64, get_correct_timestamp_cp(1));
    //     assert!(!result.success);
    //     assert_eq!(result.error, f!("Requested key {key} does not exist"));
    // }
    //
    // #[test]
    // fn republish_key_not_exists() {
    //     let mut aqua_dht = ServiceInterface::new();
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
    //     republish_key_and_check(&mut aqua_dht, &key, 123u64, &get_correct_timestamp_cp(1));
    // }
    //
    // #[test]
    // fn republish_key_same_peer_id() {
    //     let mut aqua_dht = ServiceInterface::new();
    //     clear_env();
    //     let key_str = "some_key".to_string();
    //     let timestamp = 123u64;
    //     let weight = 8u32;
    //     let pin = false;
    //     let mut cp = get_correct_timestamp_cp(1);
    //     cp.init_peer_id = "some_peer_id".to_string();
    //     register_key_and_check(&mut aqua_dht, &key_str, timestamp, pin, weight, &cp);
    //
    //     let key = Key {
    //         key: key_str.clone(),
    //         peer_id: cp.init_peer_id,
    //         timestamp_created: timestamp + 1,
    //         pinned: false,
    //         weight: weight.clone(),
    //     };
    //
    //     republish_key_and_check(&mut aqua_dht, &key, 123123u64, &get_correct_timestamp_cp(1));
    // }
    //
    // #[test]
    // fn republish_key_other_peer_id() {
    //     let mut aqua_dht = ServiceInterface::new();
    //     clear_env();
    //     let key_str = "some_key".to_string();
    //     let timestamp = 123u64;
    //     let weight = 8u32;
    //     let pin = false;
    //     let mut cp = get_correct_timestamp_cp(1);
    //     cp.init_peer_id = "some_peer_id".to_string();
    //     register_key_and_check(&mut aqua_dht, &key_str, timestamp, pin, weight, &cp);
    //
    //     let key = Key {
    //         key: key_str.clone(),
    //         peer_id: "OTHER_PEER_ID".to_string(),
    //         timestamp_created: timestamp + 1,
    //         pinned: false,
    //         weight: weight.clone(),
    //     };
    //
    //     let result = aqua_dht.republish_key_cp(key, 123123u64, get_correct_timestamp_cp(1));
    //     assert!(!result.success);
    //     assert_eq!(result.error, KeyAlreadyExists(key_str).to_string());
    // }
    //
    // #[test]
    // fn put_value_empty_cp() {
    //     let mut aqua_dht = ServiceInterface::new();
    //     clear_env();
    //     let result = aqua_dht.put_value(
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
    //     let mut aqua_dht = ServiceInterface::new();
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
    //     let result = aqua_dht.put_value_cp(
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
    //     let mut aqua_dht = ServiceInterface::new();
    //     clear_env();
    //     let result = aqua_dht.get_values("some_key".to_string(), 123u64);
    //     assert!(!result.success);
    //     assert_eq!(
    //         result.error,
    //         InvalidTimestampTetraplet("[]".to_string()).to_string()
    //     );
    // }
    //
    // #[test]
    // fn get_values_invalid_cp() {
    //     let mut aqua_dht = ServiceInterface::new();
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
    //     let result = aqua_dht.get_values_cp("some_key".to_string(), 123u64, invalid_cp.clone());
    //     assert!(!result.success);
    //     assert_eq!(
    //         result.error,
    //         InvalidTimestampTetraplet(format!("{:?}", invalid_cp.tetraplets[1][0])).to_string()
    //     );
    // }
    //
    // #[test]
    // fn get_values_empty() {
    //     let mut aqua_dht = ServiceInterface::new();
    //     clear_env();
    //
    //     let key = "some_key".to_string();
    //     register_key_and_check(
    //         &mut aqua_dht,
    //         &key,
    //         123u64,
    //         false,
    //         8u32,
    //         &get_correct_timestamp_cp(1),
    //     );
    //
    //     let result = aqua_dht.get_values_cp(key, 123u64, get_correct_timestamp_cp(1));
    //
    //     assert!(result.success, "{}", result.error);
    //
    //     assert_eq!(result.result.len(), 0);
    // }
    //
    // #[test]
    // fn get_values_key_not_exists() {
    //     let mut aqua_dht = ServiceInterface::new();
    //     clear_env();
    //
    //     let key = "invalid_key".to_string();
    //     let result = aqua_dht.get_values_cp(key.clone(), 123u64, get_correct_timestamp_cp(1));
    //
    //     assert!(!result.success);
    //     assert_eq!(result.error, f!("Requested key {key} does not exist"));
    //     assert_eq!(result.result.len(), 0);
    // }
    //
    // #[test]
    // fn put_value_key_not_exists() {
    //     let mut aqua_dht = ServiceInterface::new();
    //     clear_env();
    //     let key = "some_key".to_string();
    //     let result = aqua_dht.put_value_cp(
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
    //     let mut aqua_dht = ServiceInterface::new();
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
    //         &mut aqua_dht,
    //         &key,
    //         timestamp,
    //         false,
    //         8u32,
    //         &get_correct_timestamp_cp(1),
    //     );
    //     put_value_and_check(
    //         &mut aqua_dht,
    //         &key,
    //         &value,
    //         timestamp,
    //         &vec![relay_id.clone()],
    //         &vec![service_id.clone()],
    //         weight,
    //         &cp,
    //     );
    //
    //     let result = aqua_dht.get_values_cp(key, timestamp.clone(), get_correct_timestamp_cp(1));
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
    //     let mut aqua_dht = ServiceInterface::new();
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
    //         &mut aqua_dht,
    //         &key,
    //         timestamp,
    //         false,
    //         8u32,
    //         &get_correct_timestamp_cp(1),
    //     );
    //     put_value_and_check(
    //         &mut aqua_dht,
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
    //         &mut aqua_dht,
    //         &key,
    //         &value2,
    //         timestamp,
    //         &vec![relay_id.clone()],
    //         &vec![service_id.clone()],
    //         weight,
    //         &cp,
    //     );
    //
    //     let result = aqua_dht.get_values_cp(key, timestamp.clone(), get_correct_timestamp_cp(1));
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
    //     let mut aqua_dht = ServiceInterface::new();
    //     clear_env();
    //     let key = "some_key".to_string();
    //     let value = "some_value".to_string();
    //     let timestamp = 123u64;
    //
    //     let put_value = |aqua_dht: &mut ServiceInterface, peer_id: &str, weight: u32| {
    //         let mut cp = get_correct_timestamp_cp(2);
    //         cp.init_peer_id = peer_id.to_string();
    //         put_value_and_check(
    //             aqua_dht,
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
    //         &mut aqua_dht,
    //         &key,
    //         timestamp,
    //         false,
    //         8u32,
    //         &get_correct_timestamp_cp(1),
    //     );
    //
    //     let min_weight = 10u32;
    //     for i in 0..VALUES_LIMIT {
    //         put_value(&mut aqua_dht, &i.to_string(), min_weight + i as u32);
    //     }
    //
    //     // try to put value with smaller weight
    //     let smaller_weight = min_weight - 1;
    //     let mut cp = get_correct_timestamp_cp(2);
    //     cp.init_peer_id = "unique_peer_id1".to_string();
    //     let result = aqua_dht.put_value_cp(
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
    //         &mut aqua_dht,
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
    //         &mut aqua_dht,
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
    //     let mut aqua_dht = ServiceInterface::new();
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
    //         &mut aqua_dht,
    //         &key,
    //         timestamp,
    //         false,
    //         8u32,
    //         &get_correct_timestamp_cp(1),
    //     );
    //
    //     cp.init_peer_id = peer1_id.clone();
    //     put_value_and_check(
    //         &mut aqua_dht,
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
    //         &mut aqua_dht,
    //         &key,
    //         &value,
    //         timestamp,
    //         &relay_id,
    //         &service_id,
    //         weight,
    //         &cp,
    //     );
    //
    //     let result = aqua_dht.get_values_cp(key, timestamp.clone(), get_correct_timestamp_cp(1));
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
    //     let mut aqua_dht = ServiceInterface::new();
    //     clear_env();
    //
    //     let result = aqua_dht.clear_expired(124u64);
    //     assert!(!result.success);
    //     assert_eq!(
    //         result.error,
    //         InvalidTimestampTetraplet("[]".to_string()).to_string()
    //     );
    // }
    //
    // #[test]
    // fn clear_expired_invalid_cp() {
    //     let mut aqua_dht = ServiceInterface::new();
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
    //     let result = aqua_dht.clear_expired_cp(124u64, invalid_cp.clone());
    //     assert!(!result.success);
    //     assert_eq!(
    //         result.error,
    //         InvalidTimestampTetraplet(format!("{:?}", invalid_cp.tetraplets)).to_string()
    //     );
    // }
    //
    // #[test]
    // fn clear_expired_empty() {
    //     let mut aqua_dht = ServiceInterface::new();
    //     clear_env();
    //     let result = aqua_dht.clear_expired_cp(124u64, get_correct_timestamp_cp(0));
    //
    //     assert!(result.success, "{}", result.error);
    //     assert_eq!(result.count_keys + result.count_values, 0);
    // }
    //
    // #[test]
    // fn clear_expired_key_without_values() {
    //     let mut aqua_dht = ServiceInterface::new();
    //     clear_env();
    //     let key = "some_key".to_string();
    //     let expired_timestamp = SystemTime::now()
    //         .duration_since(SystemTime::UNIX_EPOCH)
    //         .unwrap()
    //         .as_secs();
    //     register_key_and_check(
    //         &mut aqua_dht,
    //         &key,
    //         expired_timestamp,
    //         false,
    //         8u32,
    //         &get_correct_timestamp_cp(1),
    //     );
    //
    //     let result = aqua_dht.clear_expired_cp(
    //         expired_timestamp + DEFAULT_EXPIRED_VALUE_AGE,
    //         get_correct_timestamp_cp(0),
    //     );
    //
    //     assert!(result.success, "{}", result.error);
    //
    //     assert_eq!(result.count_keys, 1);
    //     assert_eq!(result.count_values, 0);
    //
    //     let result = aqua_dht.get_key_metadata_cp(key.clone(), 123u64, get_correct_timestamp_cp(1));
    //     assert!(!result.success);
    //     assert_eq!(result.error, f!("Requested key {key} does not exist"));
    // }
    //
    // #[test]
    // fn clear_expired_host_key() {
    //     let mut aqua_dht = ServiceInterface::new();
    //     clear_env();
    //     let key = "some_key".to_string();
    //     let expired_timestamp = SystemTime::now()
    //         .duration_since(SystemTime::UNIX_EPOCH)
    //         .unwrap()
    //         .as_secs();
    //     register_key_and_check(
    //         &mut aqua_dht,
    //         &key,
    //         expired_timestamp,
    //         true,
    //         8u32,
    //         &get_correct_timestamp_cp(1),
    //     );
    //     put_value_and_check(
    //         &mut aqua_dht,
    //         &key,
    //         &"some_value".to_string(),
    //         expired_timestamp,
    //         &vec![],
    //         &vec![],
    //         8u32,
    //         &get_correct_timestamp_cp(2),
    //     );
    //
    //     let result = aqua_dht.clear_expired_cp(
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
    //     let mut aqua_dht = ServiceInterface::new();
    //     clear_env();
    //     let key = "some_key".to_string();
    //     let expired_timestamp = SystemTime::now()
    //         .duration_since(SystemTime::UNIX_EPOCH)
    //         .unwrap()
    //         .as_secs();
    //     register_key_and_check(
    //         &mut aqua_dht,
    //         &key,
    //         expired_timestamp,
    //         false,
    //         8u32,
    //         &get_correct_timestamp_cp(1),
    //     );
    //     put_host_value_and_check(
    //         &mut aqua_dht,
    //         &key,
    //         &"some_value".to_string(),
    //         expired_timestamp,
    //         &vec![],
    //         &vec![],
    //         8u32,
    //         &get_correct_timestamp_cp(2),
    //     );
    //
    //     let result = aqua_dht.clear_expired_cp(
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
    //     let mut aqua_dht = ServiceInterface::new();
    //     clear_env();
    //     let key = "some_key".to_string();
    //     let expired_timestamp = SystemTime::now()
    //         .duration_since(SystemTime::UNIX_EPOCH)
    //         .unwrap()
    //         .as_secs();
    //     register_key_and_check(
    //         &mut aqua_dht,
    //         &key,
    //         expired_timestamp,
    //         false,
    //         8u32,
    //         &get_correct_timestamp_cp(1),
    //     );
    //     put_value_and_check(
    //         &mut aqua_dht,
    //         &key,
    //         &"some_value".to_string(),
    //         expired_timestamp,
    //         &vec![],
    //         &vec![],
    //         8u32,
    //         &get_correct_timestamp_cp(2),
    //     );
    //
    //     let result = aqua_dht.clear_expired_cp(
    //         expired_timestamp + DEFAULT_EXPIRED_VALUE_AGE,
    //         get_correct_timestamp_cp(0),
    //     );
    //
    //     assert!(result.success, "{}", result.error);
    //
    //     assert_eq!(result.count_keys, 1);
    //     assert_eq!(result.count_values, 1);
    //
    //     let result = aqua_dht.get_key_metadata_cp(key.clone(), 123u64, get_correct_timestamp_cp(1));
    //     assert!(!result.success);
    //     assert_eq!(result.error, f!("Requested key {key} does not exist"));
    //
    //     let result = aqua_dht.get_values_cp(key.clone(), 123u64, get_correct_timestamp_cp(1));
    //
    //     assert!(!result.success);
    //     assert_eq!(result.error, f!("Requested key {key} does not exist"));
    // }
    //
    // #[test]
    // fn clear_expired_change_timeout() {
    //     let mut aqua_dht = ServiceInterface::new();
    //     clear_env();
    //     let key = "some_key".to_string();
    //     let expired_timestamp = SystemTime::now()
    //         .duration_since(SystemTime::UNIX_EPOCH)
    //         .unwrap()
    //         .as_secs();
    //
    //     register_key_and_check(
    //         &mut aqua_dht,
    //         &key,
    //         expired_timestamp,
    //         false,
    //         8u32,
    //         &get_correct_timestamp_cp(1),
    //     );
    //     put_value_and_check(
    //         &mut aqua_dht,
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
    //     aqua_dht.set_expired_timeout(new_expired_timeout.clone());
    //     let result = aqua_dht.clear_expired_cp(
    //         expired_timestamp + new_expired_timeout,
    //         get_correct_timestamp_cp(0),
    //     );
    //
    //     assert!(result.success, "{}", result.error);
    //
    //     assert_eq!(result.count_keys, 1);
    //     assert_eq!(result.count_values, 1);
    //
    //     let result = aqua_dht.get_key_metadata_cp(key.clone(), 123u64, get_correct_timestamp_cp(1));
    //     assert!(!result.success);
    //     assert_eq!(result.error, f!("Requested key {key} does not exist"));
    //
    //     let result = aqua_dht.get_values_cp(key.clone(), 123u64, get_correct_timestamp_cp(1));
    //
    //     assert!(!result.success);
    //     assert_eq!(result.error, f!("Requested key {key} does not exist"));
    // }
    //
    // #[test]
    // fn evict_stale_empty_cp() {
    //     let mut aqua_dht = ServiceInterface::new();
    //     clear_env();
    //
    //     let result = aqua_dht.evict_stale(124u64);
    //     assert!(!result.success);
    //     assert_eq!(
    //         result.error,
    //         InvalidTimestampTetraplet("[]".to_string()).to_string()
    //     );
    // }
    //
    // #[test]
    // fn evict_stale_invalid_cp() {
    //     let mut aqua_dht = ServiceInterface::new();
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
    //     let result = aqua_dht.evict_stale_cp(124u64, invalid_cp.clone());
    //     assert!(!result.success);
    //     assert_eq!(
    //         result.error,
    //         InvalidTimestampTetraplet(format!("{:?}", invalid_cp.tetraplets)).to_string()
    //     );
    // }
    //
    // #[test]
    // fn evict_stale_empty() {
    //     let mut aqua_dht = ServiceInterface::new();
    //     clear_env();
    //     let result = aqua_dht.evict_stale_cp(124u64, get_correct_timestamp_cp(0));
    //     assert!(result.success, "{}", result.error);
    //
    //     assert_eq!(result.results.len(), 0);
    // }
    //
    // #[test]
    // fn evict_stale_key_without_values() {
    //     let mut aqua_dht = ServiceInterface::new();
    //     clear_env();
    //     let key = "some_key".to_string();
    //     let stale_timestamp = 0u64;
    //     register_key_and_check(
    //         &mut aqua_dht,
    //         &key,
    //         stale_timestamp,
    //         false,
    //         8u32,
    //         &get_correct_timestamp_cp(1),
    //     );
    //
    //     let result = aqua_dht.evict_stale_cp(
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
    //     let result = aqua_dht.get_key_metadata_cp(key.clone(), 123u64, get_correct_timestamp_cp(1));
    //     assert!(!result.success);
    //     assert_eq!(result.error, f!("Requested key {key} does not exist"));
    // }
    //
    // #[test]
    // fn evict_stale_key_with_values() {
    //     let mut aqua_dht = ServiceInterface::new();
    //     clear_env();
    //     let key = "some_key".to_string();
    //     let value = "some_value".to_string();
    //     let stale_timestamp = 0u64;
    //     register_key_and_check(
    //         &mut aqua_dht,
    //         &key,
    //         stale_timestamp,
    //         false,
    //         8u32,
    //         &get_correct_timestamp_cp(1),
    //     );
    //     put_value_and_check(
    //         &mut aqua_dht,
    //         &key,
    //         &value,
    //         stale_timestamp,
    //         &vec![],
    //         &vec![],
    //         8u32,
    //         &get_correct_timestamp_cp(2),
    //     );
    //
    //     let result = aqua_dht.evict_stale_cp(
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
    //     let result = aqua_dht.get_key_metadata_cp(key.clone(), 123u64, get_correct_timestamp_cp(1));
    //     assert!(!result.success);
    //     assert_eq!(result.error, f!("Requested key {key} does not exist"));
    //
    //     let result = aqua_dht.get_values_cp(key.clone(), 123u64, get_correct_timestamp_cp(1));
    //
    //     assert!(!result.success);
    //     assert_eq!(result.error, f!("Requested key {key} does not exist"));
    // }
    //
    // #[test]
    // fn merge_test() {
    //     let mut aqua_dht = ServiceInterface::new();
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
    //     let result = aqua_dht.merge(vec![vec![stale_record.clone()], vec![new_record.clone()]]);
    //
    //     assert_eq!(result.result.len(), 1);
    //     let record = &result.result[0];
    //     assert_eq!(record.value, new_record.value);
    //     assert_eq!(record.timestamp_created, new_record.timestamp_created);
    //
    //     let result = aqua_dht.merge_two(vec![stale_record.clone()], vec![new_record.clone()]);
    //
    //     assert_eq!(result.result.len(), 1);
    //     let record = &result.result[0];
    //     assert_eq!(record.value, new_record.value);
    //     assert_eq!(record.timestamp_created, new_record.timestamp_created);
    // }
    //
    // #[test]
    // fn merge_test_different_peer_ids() {
    //     let mut aqua_dht = ServiceInterface::new();
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
    //     let result = aqua_dht.merge_two(vec![record1], vec![record2]);
    //
    //     assert_eq!(result.result.len(), 2);
    // }
    //
    // // test repeats initTopicAndSubscribeNode method from pubsub api
    // #[test]
    // fn init_topic_and_subscribe_node_test() {
    //     let mut aqua_dht = ServiceInterface::new();
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
    //     register_key_and_check(&mut aqua_dht, &topic, timestamp, pin, weight, &cp);
    //
    //     let mut cp = get_correct_timestamp_cp(2);
    //     cp.init_peer_id = subscriber_peer_id.clone();
    //
    //     // make a subscription
    //     let result = put_host_value_and_check(
    //         &mut aqua_dht,
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
    //     let mut aqua_dht = ServiceInterface::new();
    //     clear_env();
    //
    //     // === notify neighbor about subscription
    //
    //     let mut cp = get_correct_timestamp_cp(1);
    //     cp.init_peer_id = subscriber_peer_id.clone();
    //
    //     // register topic on neighbor
    //     register_key_and_check(&mut aqua_dht, &topic, timestamp, pin, weight, &cp);
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
    //     let result = aqua_dht.propagate_host_value_cp(result, timestamp, weight.clone(), cp);
    //     assert!(result.success, "{}", result.error);
    //
    //     // check subscription (mimics findSubscribers but for one node without merging)
    //     let result = aqua_dht.get_values_cp(topic, 123u64, get_correct_timestamp_cp(1));
    //
    //     assert!(result.success, "{}", result.error);
    //     assert_eq!(result.result.len(), 1);
    //     let record = result.result[0].clone();
    //     assert_eq!(record.value, value);
    //     assert_eq!(record.peer_id, HOST_ID);
    //     assert_eq!(record.set_by, subscriber_peer_id);
    // }
    //
    // // checks evict_stale -> republish_key[values] -> clear_expired lifecycle
    // #[test]
    // fn evict_republish_clear_expired() {
    //     let mut aqua_dht = ServiceInterface::new();
    //     clear_env();
    //     let key = "some_key".to_string();
    //     let value = "some_value".to_string();
    //     let stale_timestamp = 0u64;
    //
    //     // register key and put some value
    //     register_key_and_check(
    //         &mut aqua_dht,
    //         &key,
    //         stale_timestamp,
    //         false,
    //         8u32,
    //         &get_correct_timestamp_cp(1),
    //     );
    //     put_value_and_check(
    //         &mut aqua_dht,
    //         &key,
    //         &value,
    //         stale_timestamp,
    //         &vec![],
    //         &vec![],
    //         8u32,
    //         &get_correct_timestamp_cp(2),
    //     );
    //
    //     // increase timestamp to make value and key stale
    //     let current_timestamp = stale_timestamp + DEFAULT_STALE_VALUE_AGE;
    //
    //     // retrieve values and keys to republish
    //     let result = aqua_dht.evict_stale_cp(current_timestamp, get_correct_timestamp_cp(0));
    //     assert!(result.success, "{}", result.error);
    //
    //     assert_eq!(result.results.len(), 1);
    //
    //     let item = &result.results[0];
    //     assert_eq!(item.key.key, key);
    //     assert_eq!(item.records.len(), 1);
    //
    //     let key_to_republish = item.key.clone();
    //     let records_to_republish = item.records.clone();
    //
    //     let record = &item.records[0];
    //     assert_eq!(record.value, value);
    //     assert_eq!(record.timestamp_created, stale_timestamp);
    //
    //     // check that key not exists and values are empty (because node is neighbor to itself and should republish values to itself)
    //     // get_values checks key existence
    //     let result =
    //         aqua_dht.get_values_cp(key.clone(), current_timestamp, get_correct_timestamp_cp(1));
    //     assert!(!result.success);
    //     assert_eq!(result.error, f!("Requested key {key} does not exist"));
    //
    //     // republish key and values
    //     let result = aqua_dht.republish_key_cp(
    //         key_to_republish,
    //         current_timestamp,
    //         get_correct_timestamp_cp(1),
    //     );
    //     assert!(result.success, "{}", result.error);
    //
    //     let result = aqua_dht.republish_values_cp(
    //         key.clone(),
    //         records_to_republish.clone(),
    //         current_timestamp,
    //         get_correct_timestamp_cp(2),
    //     );
    //     assert!(result.success, "{}", result.error);
    //
    //     // check values' existence
    //     let result =
    //         aqua_dht.get_values_cp(key.clone(), current_timestamp, get_correct_timestamp_cp(1));
    //     assert!(result.success, "{}", result.error);
    //
    //     assert_eq!(result.result.len(), 1);
    //
    //     // increase timestamp to make value and key expired
    //     let expired_timestamp = current_timestamp + DEFAULT_EXPIRED_VALUE_AGE;
    //
    //     // clear expired values and keys
    //     let result = aqua_dht.clear_expired_cp(expired_timestamp, get_correct_timestamp_cp(0));
    //     assert!(result.success, "{}", result.error);
    //     assert_eq!(result.count_keys, 1);
    //     assert_eq!(result.count_values, 1);
    //
    //     // check that values and keys not exists anymore (get_values checks key existence)
    //     let result =
    //         aqua_dht.get_values_cp(key.clone(), expired_timestamp, get_correct_timestamp_cp(1));
    //
    //     assert!(!result.success);
    //     assert_eq!(result.error, f!("Requested key {key} does not exist"));
    // }
    //
    // #[test]
    // pub fn sql_injection_test() {
    //     let mut aqua_dht = ServiceInterface::new();
    //     clear_env();
    //     let key = "blabla".to_string();
    //     let injection_key =
    //         f!("{key}', '123', '123', 'abc', '0', '0'); DELETE FROM TABLE {KEYS_TABLE_NAME};");
    //
    //     let result = aqua_dht.register_key_cp(
    //         injection_key.clone(),
    //         123u64,
    //         false,
    //         0u32,
    //         get_correct_timestamp_cp(1),
    //     );
    //     assert!(result.success, "{}", result.error);
    //
    //     let result = aqua_dht.get_key_metadata_cp(
    //         injection_key.clone(),
    //         123u64,
    //         get_correct_timestamp_cp(1),
    //     );
    //     assert!(result.success, "{}", result.error);
    //     assert_eq!(result.key.key, injection_key);
    // }
}

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
    use fluence_test::marine_test;
    use rusqlite::{Connection};
    use crate::{KEYS_TABLE_NAME, VALUES_TABLE_NAME, DB_PATH, TRUSTED_TIMESTAMP_FUNCTION_NAME, TRUSTED_TIMESTAMP_SERVICE_ID, EXPIRED_VALUE_AGE, STALE_VALUE_AGE};
    use fluence::{CallParameters, SecurityTetraplet};

    fn clear_db() {
        let connection = Connection::open(DB_PATH).unwrap();

        connection.execute(f!("DELETE FROM {KEYS_TABLE_NAME}").as_str(), []).unwrap();
        connection.execute(f!("DELETE FROM {VALUES_TABLE_NAME}").as_str(), []).unwrap();
    }

    fn get_correct_timestamp_cp(arg_number: usize) -> CallParameters {
        let mut cp = CallParameters::default();

        for _ in 0..arg_number {
            cp.tetraplets.push(vec![]);
        }

        cp.tetraplets.push(vec![SecurityTetraplet {
            peer_pk: "".to_string(),
            service_id: TRUSTED_TIMESTAMP_SERVICE_ID.to_string(),
            function_name: TRUSTED_TIMESTAMP_FUNCTION_NAME.to_string(),
            json_path: "".to_string(),
        }]);

        cp
    }

    macro_rules! put_value_and_check {
        ($aqua_dht:expr, $key:expr, $value:expr, $timestamp:expr, $relay_id:expr, $cp:expr) => {
            {
                let result = $aqua_dht.put_value_cp($key.clone(), $value.clone(), $timestamp.clone(), $relay_id.clone(), $cp.clone());

                assert_eq!(result.error, "");
                assert!(result.success);
            }
        }
    }

    macro_rules! check_key_metadata {
        ($aqua_dht:expr, $key:expr, $timestamp:expr, $peer_id:expr, $current_timestamp:expr, $cp:expr) => {
            {
                let result = $aqua_dht.get_key_metadata_cp($key.clone(), $current_timestamp.clone(), $cp.clone());
                assert!(result.success);
                assert_eq!(result.error, "");
                assert_eq!(result.key.key, $key);
                assert_eq!(result.key.peer_id, $peer_id);
                assert_eq!(result.key.timestamp_created, $timestamp);
            }
        }
    }

    macro_rules! register_key_and_check {
        ($aqua_dht:expr, $key:expr, $timestamp:expr, $cp:expr) => {
            {
                let result = $aqua_dht.register_key_cp($key.clone(), $timestamp, $cp.clone());
                assert_eq!(result.error, "");
                assert!(result.success);

                check_key_metadata!($aqua_dht, $key, $timestamp, $cp.init_peer_id, $timestamp, $cp);
            }

        }
    }

    macro_rules! republish_key_and_check {
        ($aqua_dht:expr, $key:expr, $timestamp:expr, $cp:expr) => {
            {
                let result = $aqua_dht.republish_key_cp($key.clone(), $timestamp, $cp.clone());
                assert_eq!(result.error, "");
                assert!(result.success);

                check_key_metadata!($aqua_dht, $key.key, $key.timestamp_created, $key.peer_id, $timestamp, $cp);
            }
        }
    }

    #[marine_test(config_path = "../Config.toml", modules_dir = "../artifacts/")]
    fn register_key() {
        clear_db();
        register_key_and_check!(aqua_dht, "some_key".to_string(), 123u64, get_correct_timestamp_cp(1));
    }

    #[marine_test(config_path = "../Config.toml", modules_dir = "../artifacts/")]
    fn register_key_empty_cp() {
        clear_db();
        let result = aqua_dht.register_key("some_key".to_string(), 123u64);
        assert!(!result.success);
        assert_eq!(result.error, "you should use peer.timestamp_ms to pass timestamp");
    }

    #[marine_test(config_path = "../Config.toml", modules_dir = "../artifacts/")]
    fn register_key_invalid_cp() {
        clear_db();
        let mut invalid_cp = CallParameters::default();
        invalid_cp.tetraplets.push(vec![]);
        invalid_cp.tetraplets.push(vec![SecurityTetraplet {
            peer_pk: "some peer_pk".to_string(),
            service_id: "INVALID SERVICE ID".to_string(),
            function_name: "INVALID FUNCTION NAME".to_string(),
            json_path: "some json path".to_string(),
        }]);

        let result = aqua_dht.register_key_cp("some_key".to_string(), 123u64, invalid_cp);
        assert!(!result.success);
        assert_eq!(result.error, "you should use peer.timestamp_ms to pass timestamp");
    }

    #[marine_test(config_path = "../Config.toml", modules_dir = "../artifacts/")]
    fn register_key_twice_same_peer_id() {
        clear_db();
        let key = "some_key".to_string();
        let timestamp = 123u64;
        let mut cp = get_correct_timestamp_cp(1);
        cp.init_peer_id = "some_peer_id".to_string();

        register_key_and_check!(aqua_dht, key, timestamp, cp);
        register_key_and_check!(aqua_dht, key, timestamp, cp);
    }

    #[marine_test(config_path = "../Config.toml", modules_dir = "../artifacts/")]
    fn register_key_twice_other_peer_id() {
        clear_db();
        let key = "some_key".to_string();
        let timestamp = 123u64;
        let mut cp = get_correct_timestamp_cp(1);
        cp.init_peer_id = "some_peer_id".to_string();
        register_key_and_check!(aqua_dht, key, timestamp, cp);

        cp.init_peer_id = "other_peer_id".to_string();
        let result = aqua_dht.register_key_cp(key.clone(), timestamp, cp);
        assert!(!result.success);
        assert_eq!(result.error, "key already exists with different peer_id");
    }

    #[marine_test(config_path = "../Config.toml", modules_dir = "../artifacts/")]
    fn get_key_metadata_not_found() {
        clear_db();
        let result = aqua_dht.get_key_metadata_cp("invalid_key".to_string(), 123u64, get_correct_timestamp_cp(1));
        assert!(!result.success);
        assert_eq!(result.error, "not found");
    }

    #[marine_test(config_path = "../Config.toml", modules_dir = "../artifacts/")]
    fn republish_key_not_exists() {
        clear_db();
        let key = aqua_dht_structs::Key {
            key: "some_key".to_string(),
            peer_id: "some_peer".to_string(),
            timestamp_created: 0,
        };

        republish_key_and_check!(aqua_dht, key, 123u64, get_correct_timestamp_cp(1));
    }

    #[marine_test(config_path = "../Config.toml", modules_dir = "../artifacts/")]
    fn republish_key_same_peer_id() {
        clear_db();
        let key_str = "some_key".to_string();
        let timestamp = 123u64;
        let mut cp = get_correct_timestamp_cp(1);
        cp.init_peer_id = "some_peer_id".to_string();
        register_key_and_check!(aqua_dht, key_str, timestamp, cp);

        let key = aqua_dht_structs::Key {
            key: key_str.clone(),
            peer_id: cp.init_peer_id,
            timestamp_created: timestamp + 1,
        };

        republish_key_and_check!(aqua_dht, key, 123123u64, get_correct_timestamp_cp(1));
    }

    #[marine_test(config_path = "../Config.toml", modules_dir = "../artifacts/")]
    fn republish_key_other_peer_id() {
        clear_db();
        let key_str = "some_key".to_string();
        let timestamp = 123u64;
        let mut cp = get_correct_timestamp_cp(1);
        cp.init_peer_id = "some_peer_id".to_string();
        register_key_and_check!(aqua_dht, key_str, timestamp, cp);

        let key = aqua_dht_structs::Key {
            key: key_str.clone(),
            peer_id: "OTHER_PEER_ID".to_string(),
            timestamp_created: timestamp + 1,
        };

        let result = aqua_dht.republish_key_cp(key, 123123u64, get_correct_timestamp_cp(1));
        assert!(!result.success);
        assert_eq!(result.error, "key already exists with different peer_id");
    }

    #[marine_test(config_path = "../Config.toml", modules_dir = "../artifacts/")]
    fn put_value_empty_cp() {
        clear_db();
        let result = aqua_dht.put_value("some_key".to_string(), "value".to_string(), 123u64, vec![]);
        assert!(!result.success);
        assert_eq!(result.error, "you should use peer.timestamp_ms to pass timestamp");
    }

    #[marine_test(config_path = "../Config.toml", modules_dir = "../artifacts/")]
    fn put_value_invalid_cp() {
        clear_db();

        let mut invalid_cp = CallParameters::default();
        invalid_cp.tetraplets.push(vec![]);
        invalid_cp.tetraplets.push(vec![SecurityTetraplet {
            peer_pk: "some peer_pk".to_string(),
            service_id: "INVALID SERVICE ID".to_string(),
            function_name: "INVALID FUNCTION NAME".to_string(),
            json_path: "some json path".to_string(),
        }]);

        let result = aqua_dht.put_value_cp("some_key".to_string(), "value".to_string(), 123u64, vec![], invalid_cp);
        assert!(!result.success);
        assert_eq!(result.error, "you should use peer.timestamp_ms to pass timestamp");
    }

    #[marine_test(config_path = "../Config.toml", modules_dir = "../artifacts/")]
    fn get_values_empty_cp() {
        clear_db();
        let result = aqua_dht.get_values("some_key".to_string(), 123u64);
        assert!(!result.success);
        assert_eq!(result.error, "you should use peer.timestamp_ms to pass timestamp");
    }

    #[marine_test(config_path = "../Config.toml", modules_dir = "../artifacts/")]
    fn get_values_invalid_cp() {
        clear_db();
        let mut invalid_cp = CallParameters::default();
        invalid_cp.tetraplets.push(vec![]);
        invalid_cp.tetraplets.push(vec![SecurityTetraplet {
            peer_pk: "some peer_pk".to_string(),
            service_id: "INVALID SERVICE ID".to_string(),
            function_name: "INVALID FUNCTION NAME".to_string(),
            json_path: "some json path".to_string(),
        }]);

        let result = aqua_dht.get_values_cp("some_key".to_string(), 123u64, invalid_cp);
        assert!(!result.success);
        assert_eq!(result.error, "you should use peer.timestamp_ms to pass timestamp");
    }

    #[marine_test(config_path = "../Config.toml", modules_dir = "../artifacts/")]
    fn get_values_empty() {
        clear_db();

        let key = "some_key".to_string();
        register_key_and_check!(aqua_dht, key, 123u64, get_correct_timestamp_cp(1));

        let result = aqua_dht.get_values_cp(key, 123u64, get_correct_timestamp_cp(1));

        assert!(result.success);
        assert_eq!(result.error, "");
        assert_eq!(result.result.len(), 0);
    }

    #[marine_test(config_path = "../Config.toml", modules_dir = "../artifacts/")]
    fn put_value_key_not_exists() {
        clear_db();
        let result = aqua_dht.put_value_cp("some_key".to_string(), "value".to_string(), 123u64, vec![], get_correct_timestamp_cp(2));
        assert!(!result.success);
        assert_eq!(result.error, "not found");
    }

    #[marine_test(config_path = "../Config.toml", modules_dir = "../artifacts/")]
    fn put_value() {
        clear_db();

        let key = "some_key".to_string();
        let value = "some_value".to_string();
        let timestamp = 123u64;
        let relay_id = "some_relay".to_string();
        let mut cp = get_correct_timestamp_cp(2);
        cp.init_peer_id = "some_peer_id".to_string();
        cp.service_id = "some_service_id".to_string();

        register_key_and_check!(aqua_dht, key, timestamp, get_correct_timestamp_cp(1));
        put_value_and_check!(aqua_dht, key, value, timestamp, vec![relay_id.clone()], cp);

        let result = aqua_dht.get_values_cp(key, timestamp.clone(), get_correct_timestamp_cp(1));

        assert_eq!(result.error, "");
        assert!(result.success);

        assert_eq!(result.result.len(), 1);

        let record = &result.result[0];
        assert_eq!(record.value, value);
        assert_eq!(record.peer_id, cp.init_peer_id);
        assert_eq!(record.relay_id, relay_id);
        assert_eq!(record.service_id, cp.service_id);
        assert_eq!(record.timestamp_created, timestamp);
    }

    #[marine_test(config_path = "../Config.toml", modules_dir = "../artifacts/")]
    fn put_value_update() {
        clear_db();
        let key = "some_key".to_string();
        let value1 = "some_value".to_string();
        let timestamp = 123u64;
        let relay_id = "some_relay".to_string();
        let mut cp = get_correct_timestamp_cp(2);
        cp.init_peer_id = "some_peer_id".to_string();
        cp.service_id = "some_service_id".to_string();

        register_key_and_check!(aqua_dht, key, timestamp, get_correct_timestamp_cp(1));
        put_value_and_check!(aqua_dht, key, value1, timestamp, vec![relay_id.clone()], cp);
        let value2 = "other_value".to_string();
        put_value_and_check!(aqua_dht, key, value2, timestamp, vec![relay_id.clone()], cp);

        let result = aqua_dht.get_values_cp(key, timestamp.clone(), get_correct_timestamp_cp(1));

        assert_eq!(result.error, "");
        assert!(result.success);

        assert_eq!(result.result.len(), 1);

        let record = &result.result[0];
        assert_eq!(record.value, value2);
        assert_eq!(record.peer_id, cp.init_peer_id);
        assert_eq!(record.relay_id, relay_id);
        assert_eq!(record.service_id, cp.service_id);
        assert_eq!(record.timestamp_created, timestamp);
    }

    #[marine_test(config_path = "../Config.toml", modules_dir = "../artifacts/")]
    fn put_multiple_values_for_key() {
        clear_db();
        let key = "some_key".to_string();
        let value = "some_value".to_string();
        let timestamp = 123u64;
        let relay_id = "some_relay".to_string();
        let mut cp = get_correct_timestamp_cp(2);
        let peer1_id = "some_peer_id".to_string();
        let peer2_id = "other_peer_id".to_string();
        cp.service_id = "some_service_id".to_string();

        register_key_and_check!(aqua_dht, key, timestamp, get_correct_timestamp_cp(1));

        cp.init_peer_id = peer1_id.clone();
        put_value_and_check!(aqua_dht, key, value, timestamp, vec![relay_id.clone()], cp);

        cp.init_peer_id = peer2_id.clone();
        put_value_and_check!(aqua_dht, key, value, timestamp, vec![relay_id.clone()], cp);

        let result = aqua_dht.get_values_cp(key, timestamp.clone(), get_correct_timestamp_cp(1));

        assert_eq!(result.error, "");
        assert!(result.success);

        assert_eq!(result.result.len(), 2);

        let record = &result.result[0];
        assert_eq!(record.value, value);
        assert_eq!(record.peer_id, peer2_id);
        assert_eq!(record.relay_id, relay_id);
        assert_eq!(record.service_id, cp.service_id);
        assert_eq!(record.timestamp_created, timestamp);

        let record = &result.result[1];
        assert_eq!(record.value, value);
        assert_eq!(record.peer_id, peer1_id);
        assert_eq!(record.relay_id, relay_id);
        assert_eq!(record.service_id, cp.service_id);
        assert_eq!(record.timestamp_created, timestamp);
    }

    #[marine_test(config_path = "../Config.toml", modules_dir = "../artifacts/")]
    fn clear_expired_empty_cp() {
        clear_db();

        let result = aqua_dht.clear_expired(124u64);
        assert!(!result.success);
        assert_eq!(result.error, "you should use peer.timestamp_ms to pass timestamp");
    }

    #[marine_test(config_path = "../Config.toml", modules_dir = "../artifacts/")]
    fn clear_expired_invalid_cp() {
        clear_db();

        let mut invalid_cp = CallParameters::default();
        invalid_cp.tetraplets.push(vec![]);
        invalid_cp.tetraplets.push(vec![SecurityTetraplet {
            peer_pk: "some peer_pk".to_string(),
            service_id: "INVALID SERVICE ID".to_string(),
            function_name: "INVALID FUNCTION NAME".to_string(),
            json_path: "some json path".to_string(),
        }]);

        let result = aqua_dht.clear_expired_cp(124u64, invalid_cp);
        assert!(!result.success);
        assert_eq!(result.error, "you should use peer.timestamp_ms to pass timestamp");
    }

    #[marine_test(config_path = "../Config.toml", modules_dir = "../artifacts/")]
    fn clear_expired_empty() {
        clear_db();
        let result = aqua_dht.clear_expired_cp(124u64, get_correct_timestamp_cp(0));
        assert!(result.success);
        assert_eq!(result.error, "");
        assert_eq!(result.count_keys + result.count_values, 0);
    }

    #[marine_test(config_path = "../Config.toml", modules_dir = "../artifacts/")]
    fn clear_expired_key_without_values() {
        clear_db();
        let key = "some_key".to_string();
        let expired_timestamp = 0u64;
        register_key_and_check!(aqua_dht, key.clone(), expired_timestamp.clone(), get_correct_timestamp_cp(1));

        let result = aqua_dht.clear_expired_cp(expired_timestamp + EXPIRED_VALUE_AGE, get_correct_timestamp_cp(0));

        assert!(result.success);
        assert_eq!(result.error, "");
        assert_eq!(result.count_keys, 1);
        assert_eq!(result.count_values, 0);

        let result = aqua_dht.get_key_metadata_cp(key, 123u64, get_correct_timestamp_cp(1));
        assert!(!result.success);
        assert_eq!(result.error, "not found");
    }

    #[marine_test(config_path = "../Config.toml", modules_dir = "../artifacts/")]
    fn clear_expired_key_with_values() {
        clear_db();
        let key = "some_key".to_string();
        let expired_timestamp = 0u64;
        register_key_and_check!(aqua_dht, key.clone(), expired_timestamp.clone(), get_correct_timestamp_cp(1));
        put_value_and_check!(aqua_dht, key.clone(), "some_value".to_string(), expired_timestamp.clone(), vec![], get_correct_timestamp_cp(2));

        let result = aqua_dht.clear_expired_cp(expired_timestamp + EXPIRED_VALUE_AGE, get_correct_timestamp_cp(0));

        assert!(result.success);
        assert_eq!(result.error, "");
        assert_eq!(result.count_keys, 1);
        assert_eq!(result.count_values, 1);

        let result = aqua_dht.get_key_metadata_cp(key.clone(), 123u64, get_correct_timestamp_cp(1));
        assert!(!result.success);
        assert_eq!(result.error, "not found");

        let result = aqua_dht.get_values_cp(key, 123u64, get_correct_timestamp_cp(1));

        assert!(result.success);
        assert_eq!(result.error, "");
        assert_eq!(result.result.len(), 0);
    }

    #[marine_test(config_path = "../Config.toml", modules_dir = "../artifacts/")]
    fn evict_stale_empty_cp() {
        clear_db();

        let result = aqua_dht.evict_stale(124u64);
        assert!(!result.success);
        assert_eq!(result.error, "you should use peer.timestamp_ms to pass timestamp");
    }

    #[marine_test(config_path = "../Config.toml", modules_dir = "../artifacts/")]
    fn evict_stale_invalid_cp() {
        clear_db();

        let mut invalid_cp = CallParameters::default();
        invalid_cp.tetraplets.push(vec![]);
        invalid_cp.tetraplets.push(vec![SecurityTetraplet {
            peer_pk: "some peer_pk".to_string(),
            service_id: "INVALID SERVICE ID".to_string(),
            function_name: "INVALID FUNCTION NAME".to_string(),
            json_path: "some json path".to_string(),
        }]);

        let result = aqua_dht.evict_stale_cp(124u64, invalid_cp);
        assert!(!result.success);
        assert_eq!(result.error, "you should use peer.timestamp_ms to pass timestamp");
    }

    #[marine_test(config_path = "../Config.toml", modules_dir = "../artifacts/")]
    fn evict_stale_empty() {
        clear_db();
        let result = aqua_dht.evict_stale_cp(124u64, get_correct_timestamp_cp(0));
        assert!(result.success);
        assert_eq!(result.error, "");
        assert_eq!(result.results.len(), 0);
    }

    #[marine_test(config_path = "../Config.toml", modules_dir = "../artifacts/")]
    fn evict_stale_key_without_values() {
        clear_db();
        let key = "some_key".to_string();
        let stale_timestamp = 0u64;
        register_key_and_check!(aqua_dht, key.clone(), stale_timestamp.clone(), get_correct_timestamp_cp(1));

        let result = aqua_dht.evict_stale_cp(stale_timestamp + STALE_VALUE_AGE, get_correct_timestamp_cp(0));

        assert!(result.success);
        assert_eq!(result.error, "");
        assert_eq!(result.results.len(), 1);
        let item = &result.results[0];
        assert_eq!(item.key.key, key);
        assert_eq!(item.records.len(), 0);

        let result = aqua_dht.get_key_metadata_cp(key, 123u64, get_correct_timestamp_cp(1));
        assert!(!result.success);
        assert_eq!(result.error, "not found");
    }

    #[marine_test(config_path = "../Config.toml", modules_dir = "../artifacts/")]
    fn evict_stale_key_with_values() {
        clear_db();
        let key = "some_key".to_string();
        let value = "some_value".to_string();
        let stale_timestamp = 0u64;
        register_key_and_check!(aqua_dht, key.clone(), stale_timestamp.clone(), get_correct_timestamp_cp(1));
        put_value_and_check!(aqua_dht, key.clone(), value.clone(), stale_timestamp.clone(), vec![], get_correct_timestamp_cp(2));

        let result = aqua_dht.evict_stale_cp(stale_timestamp + STALE_VALUE_AGE, get_correct_timestamp_cp(0));

        assert!(result.success);
        assert_eq!(result.error, "");
        assert_eq!(result.results.len(), 1);

        let item = &result.results[0];
        assert_eq!(item.key.key, key);
        assert_eq!(item.records.len(), 1);

        let record = &item.records[0];
        assert_eq!(record.value, value);
        assert_eq!(record.timestamp_created, stale_timestamp);

        let result = aqua_dht.get_key_metadata_cp(key.clone(), 123u64, get_correct_timestamp_cp(1));
        assert!(!result.success);
        assert_eq!(result.error, "not found");

        let result = aqua_dht.get_values_cp(key, 123u64, get_correct_timestamp_cp(1));

        assert!(result.success);
        assert_eq!(result.error, "");
        assert_eq!(result.result.len(), 0);
    }
}

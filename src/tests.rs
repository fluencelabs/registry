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
    use crate::{KEYS_TABLE_NAME, VALUES_TABLE_NAME, DB_PATH, TRUSTED_TIMESTAMP_FUNCTION_NAME, TRUSTED_TIMESTAMP_SERVICE_ID};
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
        ($aqua_dht:expr, $key:expr, $value:expr, $timestamp:expr) => {
            {
                let put_result = $aqua_dht.put_value($key.clone(), $value.clone(), $timestamp.clone());

                assert!(put_result.success);
                assert_eq!(put_result.error, "");

                let get_result = $aqua_dht.get_value($key);

                assert!(get_result.success);
                assert_eq!(get_result.result, $value);
            }
        }
    }

    macro_rules! check_key_metadata {
        ($aqua_dht:expr, $key:expr, $timestamp:expr, $peer_id:expr) => {
            {
                let result = $aqua_dht.get_key_metadata($key.clone());
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

                check_key_metadata!($aqua_dht, $key, $timestamp, $cp.init_peer_id);
            }
        }
    }

    macro_rules! republish_key_and_check {
        ($aqua_dht:expr, $key:expr, $timestamp:expr, $cp:expr) => {
            {
                let result = $aqua_dht.republish_key_cp($key.clone(), $timestamp, $cp.clone());
                assert_eq!(result.error, "");
                assert!(result.success);

                check_key_metadata!($aqua_dht, $key.key, $key.timestamp_created, $key.peer_id);
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
        let key = "some_key".to_string();
        let timestamp = 123u64;

        let result = aqua_dht.register_key(key.clone(), timestamp);
        assert!(!result.success);
        assert_eq!(result.error, "you should use peer.timestamp_ms to pass timestamp");
    }

    #[marine_test(config_path = "../Config.toml", modules_dir = "../artifacts/")]
    fn register_key_invalid_cp() {
        clear_db();
        let key = "some_key".to_string();
        let timestamp = 123u64;

        let mut invalid_cp = CallParameters::default();
        invalid_cp.tetraplets.push(vec![]);
        invalid_cp.tetraplets.push(vec![SecurityTetraplet {
            peer_pk: "some peer_pk".to_string(),
            service_id: "INVALID SERVICE ID".to_string(),
            function_name: "INVALID FUNCTION NAME".to_string(),
            json_path: "some json path".to_string(),
        }]);

        let result = aqua_dht.register_key_cp(key.clone(), timestamp, invalid_cp);
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
        let result = aqua_dht.get_key_metadata("invalid_key".to_string());
        assert!(!result.success);
        assert_eq!(result.error, "not found");
    }

    #[marine_test(config_path = "../Config.toml", modules_dir = "../artifacts/")]
    fn republish_key_not_exists() {
        clear_db();
        let key = __m_generated_aqua_dht::Key {
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

        let key = __m_generated_aqua_dht::Key {
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

        let key = __m_generated_aqua_dht::Key {
            key: key_str.clone(),
            peer_id: "OTHER_PEER_ID".to_string(),
            timestamp_created: timestamp + 1,
        };

        let result = aqua_dht.republish_key_cp(key, 123123u64, get_correct_timestamp_cp(1));
        assert!(!result.success);
        assert_eq!(result.error, "key already exists with different peer_id");
    }
}

use fluence::fce;
use fluence::module_manifest;

use fce_sqlite_connector;
use fce_sqlite_connector::{Value, Connection};

#[macro_use]
extern crate fstrings;

module_manifest!();

pub static TABLE_NAME: &str = "dht";
pub static DB_PATH: &str = "/tmp/dht.db";
pub static STALE_VALUE_AGE: u64 = 60 * 60 * 1000;
pub static EXPIRED_VALUE_AGE: u64 = 24 * 60 * 60 * 1000;

#[inline]
fn get_connection() -> Connection{
    fce_sqlite_connector::open(DB_PATH).unwrap()
}

#[fce]
pub fn create_table(table_name: String) -> bool {
    let connection = get_connection();

    connection
        .execute(f!("
            CREATE TABLE IF NOT EXISTS {table_name} (
                key TEXT PRIMARY KEY,
                value TEXT,
                timestamp INTEGER,
                peer_id TEXT);
        "),
        ).is_ok()
}

fn main() {
    create_table(TABLE_NAME.to_string());
}

#[fce]
pub fn put_value(key: String, value: String, timestamp: u64) -> bool {
    let connection = get_connection();
    let peer_id = fluence::get_call_parameters().init_peer_id;

    let statement = f!("
            INSERT OR REPLACE INTO {TABLE_NAME} VALUES ('{key}', '{value}', {timestamp}, '{peer_id}');
        ");
    println!("{}", statement);
    connection.execute(statement).is_ok()
}

#[fce]
#[derive(Debug)]
pub struct GetValueResult {
    pub success: bool,
    pub result: String,
}

#[fce]
pub fn get_value(key: String) -> GetValueResult {
    let connection = get_connection();

    let mut cursor = connection
        .prepare(f!("SELECT value FROM {TABLE_NAME} WHERE key = ?"))
        .unwrap()
        .cursor();

    cursor.bind(&[Value::String(key)]).unwrap();
    if let Some(row) = cursor.next().unwrap() {
        GetValueResult { success: true, result: row[0].as_string().expect("error on row[0] parsing").to_string() }
    } else {
        GetValueResult { success: false, result: "not found".to_string() }
    }
}

#[fce]
pub fn clear_expired(current_timestamp: u64) -> u64 {
    let connection = get_connection();

    let expired_timestamp = current_timestamp - EXPIRED_VALUE_AGE;
    connection
        .execute(f!("DELETE FROM {TABLE_NAME} WHERE timestamp <= {expired_timestamp}"))
        .unwrap();

    connection.changes() as u64
}

#[fce]
#[derive(Debug)]
pub struct Record {
    pub key: String,
    pub value: String,
    pub peer_id: String,
}

#[fce]
pub fn get_stale_records(current_timestamp: u64) -> Vec<Record> {
    let connection = get_connection();

    let expired_timestamp = current_timestamp.saturating_sub(EXPIRED_VALUE_AGE);
    let stale_timestamp = current_timestamp.saturating_sub(STALE_VALUE_AGE);

    let statement = f!("SELECT key, value, peer_id FROM {TABLE_NAME} WHERE timestamp BETWEEN {expired_timestamp} AND {stale_timestamp}");
    println!("{}", statement);
    let mut cursor = connection
        .prepare(statement)
        .unwrap()
        .cursor();

    let mut result: Vec<Record> = Vec::new();
    while let Some(row) = cursor.next().unwrap() {
        let key = row[0].as_string().expect("error on key parsing").to_string();
        let value = row[1].as_string().expect("error on value parsing").to_string();
        let peer_id = row[2].as_string().expect("error on peer_id parsing").to_string();

        result.push(Record { key, value, peer_id });
    }

    result
}
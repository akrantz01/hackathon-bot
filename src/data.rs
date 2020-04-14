use crate::util::random_string;
use redis::{Client, Commands, Connection, RedisResult};
use serenity::{
    prelude::{RwLock, ShareMap, TypeMapKey},
    Client as Serenity,
};
use std::sync::Arc;

struct RedisConnection;

impl TypeMapKey for RedisConnection {
    type Value = Client;
}

// Initialize Redis connection and add to Serenity
pub fn init(serenity: &Serenity) {
    // Connect to redis
    let redis = match Client::open(crate::util::REDIS_URL.clone()) {
        Ok(c) => c,
        Err(e) => crate::util::fail(&format!("Failed to connect to redis: {}", e)),
    };

    // Attach to discord client
    let mut data = serenity.data.write();
    data.insert::<RedisConnection>(redis);
}

// Get a connection to Redis
pub fn get_connection(data: &Arc<RwLock<ShareMap>>) -> RedisResult<Connection> {
    data.read()
        .get::<RedisConnection>()
        .expect("Expected RedisConnection in ShareMap.")
        .get_connection()
}

// Persist a help request in redis
pub fn add_help_request(
    client: &mut Connection,
    description: String,
    link: String,
    table: String,
    at: i64,
) -> RedisResult<usize> {
    let request_key = format!("help_request:{}", random_string(8));

    client.lpush(&request_key, at)?;
    client.lpush(&request_key, table)?;
    client.lpush(&request_key, link)?;
    client.lpush(&request_key, description)
}

pub fn get_help_request(
    client: &mut Connection,
    key: &String,
) -> RedisResult<(String, String, String, i64)> {
    let data: (String, String, String, i64) = client.lrange(key, 0, 4)?;
    Ok(data)
}

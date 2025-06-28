// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub garble: GarbleConfig,
    pub performance: PerformanceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GarbleConfig {
    pub min_body_size: usize,
    pub max_body_size: usize,
    pub min_wait_duration_ms: u64,
    pub max_wait_duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub chunk_pool_max_memory_mb: usize,
    pub chunk_pool_min_chunks_per_size: usize,
    pub chunk_pool_max_chunks_per_size: usize,
    pub streaming_threshold_bytes: usize,
    pub fast_response_threshold_bytes: usize,
    pub background_generation_interval_ms: u64,
    pub memory_check_interval_ms: u64,
    pub enable_parallel_generation: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 3000,
            },
            garble: GarbleConfig {
                min_body_size: 100,
                max_body_size: 10000,
                min_wait_duration_ms: 0,
                max_wait_duration_ms: 1000,
            },
            performance: PerformanceConfig {
                chunk_pool_max_memory_mb: 8,
                chunk_pool_min_chunks_per_size: 5,
                chunk_pool_max_chunks_per_size: 50,
                streaming_threshold_bytes: 1_000_000,  // 1MB
                fast_response_threshold_bytes: 10_000, // 10KB
                background_generation_interval_ms: 1000,
                memory_check_interval_ms: 5000,
                enable_parallel_generation: true,
            },
        }
    }
}

impl Config {
    pub fn load_from_file(path: &str) -> Result<Self> {
        match fs::read_to_string(path) {
            Ok(content) => {
                let config: Config = serde_json::from_str(&content)?;
                Ok(config)
            }
            Err(_) => {
                tracing::warn!("Config file not found at {}, using defaults", path);
                Ok(Config::default())
            }
        }
    }
}

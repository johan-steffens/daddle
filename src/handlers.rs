// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use rand::prelude::*;
use serde::Deserialize;
use serde_json::Value;

use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

use crate::config::Config;
use crate::streaming::create_optimal_response;

#[derive(Debug, Deserialize)]
pub struct GarbleParams {
    #[serde(rename = "maxBodySize")]
    max_body_size: Option<usize>,
    #[serde(rename = "minBodySize")]
    min_body_size: Option<usize>,
    #[serde(rename = "maxWaitDuration")]
    max_wait_duration: Option<u64>,
    #[serde(rename = "minWaitDuration")]
    min_wait_duration: Option<u64>,
}

// No fixed response structure - everything is garbled!

#[axum::debug_handler]
pub async fn garble_handler(
    Query(garble_params): Query<GarbleParams>,
    State(config): State<Arc<Config>>,
) -> Result<impl IntoResponse, StatusCode> {
    // Determine effective configuration (query params override config file)
    let min_body_size = garble_params
        .min_body_size
        .unwrap_or(config.garble.min_body_size);
    let max_body_size = garble_params
        .max_body_size
        .unwrap_or(config.garble.max_body_size);
    let min_wait_duration_ms = garble_params
        .min_wait_duration
        .unwrap_or(config.garble.min_wait_duration_ms);
    let max_wait_duration_ms = garble_params
        .max_wait_duration
        .unwrap_or(config.garble.max_wait_duration_ms);

    // Validate parameters
    if min_body_size > max_body_size {
        tracing::warn!(
            "min_body_size ({}) > max_body_size ({}), swapping values",
            min_body_size,
            max_body_size
        );
    }
    if min_wait_duration_ms > max_wait_duration_ms {
        tracing::warn!(
            "min_wait_duration_ms ({}) > max_wait_duration_ms ({}), swapping values",
            min_wait_duration_ms,
            max_wait_duration_ms
        );
    }

    let effective_min_body = min_body_size.min(max_body_size);
    let effective_max_body = min_body_size.max(max_body_size);
    let effective_min_wait = min_wait_duration_ms.min(max_wait_duration_ms);
    let effective_max_wait = min_wait_duration_ms.max(max_wait_duration_ms);

    // Generate random values within the specified ranges
    let (target_size, wait_duration_ms) = {
        let mut rng = thread_rng();
        let target_size = if effective_min_body == effective_max_body {
            effective_min_body
        } else {
            rng.gen_range(effective_min_body..=effective_max_body)
        };

        let wait_duration_ms = if effective_min_wait == effective_max_wait {
            effective_min_wait
        } else {
            rng.gen_range(effective_min_wait..=effective_max_wait)
        };

        (target_size, wait_duration_ms)
    };

    // Wait for the specified duration
    if wait_duration_ms > 0 {
        sleep(Duration::from_millis(wait_duration_ms)).await;
    }

    // Use optimal response strategy based on size and configuration
    let response = create_optimal_response(target_size);

    // Log the response strategy used
    let strategy = if target_size < config.performance.fast_response_threshold_bytes {
        "direct"
    } else if target_size < config.performance.streaming_threshold_bytes {
        "fast_pool"
    } else {
        "streaming"
    };

    tracing::info!(
        "Generated GARBLED response: strategy={}, target_size={}B, wait={}ms",
        strategy,
        target_size,
        wait_duration_ms
    );

    Ok(response)
}

pub async fn health_handler() -> Json<Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "daddle",
        "version": "0.1.0",
        "timestamp": chrono::Utc::now()
    }))
}

pub async fn stats_handler() -> Json<Value> {
    use crate::chunk_pool::CHUNK_POOL;

    let stats = CHUNK_POOL.get_stats();

    Json(serde_json::json!({
        "chunk_pool": {
            "total_chunks": stats.total_chunks,
            "memory_usage_bytes": stats.memory_usage_bytes,
            "memory_usage_mb": stats.memory_usage_bytes / (1024 * 1024),
            "cache_hits": stats.cache_hits,
            "cache_misses": stats.cache_misses,
            "cache_hit_rate": if stats.cache_hits + stats.cache_misses > 0 {
                stats.cache_hits as f64 / (stats.cache_hits + stats.cache_misses) as f64
            } else {
                0.0
            },
            "background_generations": stats.background_generations
        },
        "service": "daddle",
        "version": "0.1.0",
        "timestamp": chrono::Utc::now()
    }))
}

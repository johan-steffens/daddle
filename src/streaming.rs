// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use async_stream::stream;
use axum::{
    body::Body,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use futures::{Stream, StreamExt};
use std::pin::Pin;

use crate::chunk_pool::{ChunkSize, CHUNK_POOL};
use crate::generator::RandomDataGenerator;

/// Streaming response for large JSON payloads
pub struct StreamingGarbleResponse {
    target_size: usize,
    chunk_size: usize,
}

impl StreamingGarbleResponse {
    pub fn new(target_size: usize) -> Self {
        // Use adaptive chunk size based on target size
        let chunk_size = if target_size > 10_000_000 {
            ChunkSize::XLarge.target_bytes() // 1MB chunks for very large responses
        } else if target_size > 1_000_000 {
            ChunkSize::Large.target_bytes() // 100KB chunks for large responses
        } else {
            ChunkSize::Medium.target_bytes() // 10KB chunks for medium responses
        };

        Self {
            target_size,
            chunk_size,
        }
    }

    /// Create a stream of JSON chunks
    pub fn into_stream(self) -> Pin<Box<dyn Stream<Item = Result<String, std::io::Error>> + Send>> {
        let stream = stream! {
            let mut remaining = self.target_size;
            let mut chunk_count = 0;
            let total_chunks = self.target_size.div_ceil(self.chunk_size);

            // Start JSON structure - use same format as chunk pool
            yield Ok(r#"{"garbled_chunks":["#.to_string());

            while remaining > 500 && chunk_count < total_chunks {
                if chunk_count > 0 {
                    yield Ok(",".to_string());
                }

                // Determine chunk size for this iteration
                let current_chunk_size = remaining.min(self.chunk_size);

                // Try to get from chunk pool first
                let chunk_data = if let Some(pooled_chunk) = self.get_pooled_chunk(current_chunk_size) {
                    pooled_chunk
                } else {
                    // Generate on-demand if pool is empty
                    self.generate_chunk(current_chunk_size)
                };

                // Update remaining based on actual chunk size, not target size
                let actual_chunk_size = chunk_data.len();
                remaining = remaining.saturating_sub(actual_chunk_size);

                yield Ok(chunk_data);
                chunk_count += 1;

                // Yield control to allow other tasks to run
                tokio::task::yield_now().await;
            }

            // Close JSON structure - use same format as chunk pool
            yield Ok(format!(
                r#"],"metadata":{{"generated_by":"streaming","target_size":{},"actual_size":{},"chunk_count":{},"streaming":true}}}}"#,
                self.target_size, self.target_size, chunk_count
            ));
        };

        Box::pin(stream)
    }

    fn get_pooled_chunk(&self, target_size: usize) -> Option<String> {
        // Determine best chunk size from pool
        let chunk_size = if target_size >= ChunkSize::XLarge.target_bytes() {
            ChunkSize::XLarge
        } else if target_size >= ChunkSize::Large.target_bytes() {
            ChunkSize::Large
        } else if target_size >= ChunkSize::Medium.target_bytes() {
            ChunkSize::Medium
        } else {
            ChunkSize::Small
        };

        // Get chunk from pool - these are already JSON array elements
        CHUNK_POOL.get_chunk(chunk_size)
    }

    fn generate_chunk(&self, size: usize) -> String {
        let mut generator = RandomDataGenerator::new();
        let payload = generator.generate_array_element(size);
        serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string())
    }
}

impl IntoResponse for StreamingGarbleResponse {
    fn into_response(self) -> Response {
        let stream = self.into_stream();

        // Convert string stream to bytes stream
        let byte_stream = stream.map(|result| {
            result
                .map(|s| axum::body::Bytes::from(s.into_bytes()))
                .map_err(std::io::Error::other)
        });

        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::TRANSFER_ENCODING, "chunked")
            .header("X-Garble-Mode", "streaming")
            .body(Body::from_stream(byte_stream))
            .unwrap()
    }
}

/// Fast response builder for medium-sized responses using chunk pool
pub struct FastGarbleResponse {
    target_size: usize,
}

impl FastGarbleResponse {
    pub fn new(target_size: usize) -> Self {
        Self { target_size }
    }

    /// Build response using parallel chunk assembly
    pub fn build(self) -> String {
        if self.target_size < 100_000 {
            // For small responses, use the chunk pool's build method
            CHUNK_POOL.build_response(self.target_size)
        } else {
            // For larger responses, use parallel assembly
            self.build_parallel()
        }
    }

    fn build_parallel(self) -> String {
        use rayon::prelude::*;

        // Calculate how many chunks we need
        let chunk_size = ChunkSize::Large.target_bytes(); // 100KB chunks
        let num_chunks = self.target_size.div_ceil(chunk_size);

        // Generate chunks in parallel
        let chunks: Vec<String> = (0..num_chunks)
            .into_par_iter()
            .map(|i| {
                let remaining = self.target_size - (i * chunk_size);
                let current_size = remaining.min(chunk_size);

                // Try pool first, then generate
                if let Some(chunk) = CHUNK_POOL.get_chunk(ChunkSize::Large) {
                    chunk
                } else {
                    let mut generator = RandomDataGenerator::new();
                    let payload = generator.generate_array_element(current_size);
                    serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string())
                }
            })
            .collect();

        // Assemble final response - use same format as chunk pool
        let mut result = String::with_capacity(self.target_size + 1024);
        result.push_str(r#"{"garbled_chunks":["#);

        for (i, chunk) in chunks.iter().enumerate() {
            if i > 0 {
                result.push(',');
            }
            result.push_str(chunk);
        }

        result.push_str(r#"],"metadata":{"generated_by":"parallel","target_size":"#);
        result.push_str(&self.target_size.to_string());
        result.push_str(r#","chunk_count":"#);
        result.push_str(&chunks.len().to_string());
        result.push_str(r#","actual_size":"#);
        result.push_str(&result.len().to_string());
        result.push_str(r#"}}"#);

        result
    }
}

/// Determine the best response strategy based on size
pub enum ResponseStrategy {
    Direct,    // < 10KB - generate directly
    Fast,      // 10KB - 1MB - use chunk pool + parallel
    Streaming, // > 1MB - use streaming
}

impl ResponseStrategy {
    pub fn for_size(size: usize) -> Self {
        if size < 10_000 {
            ResponseStrategy::Direct
        } else if size < 1_000_000 {
            ResponseStrategy::Fast
        } else {
            ResponseStrategy::Streaming
        }
    }
}

/// Response type that can be either regular JSON or streaming
pub enum GarbleResponse {
    Json(String),
    Streaming(StreamingGarbleResponse),
}

impl IntoResponse for GarbleResponse {
    fn into_response(self) -> Response {
        match self {
            GarbleResponse::Json(json) => Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/json")
                .header("X-Garble-Mode", "fast")
                .body(Body::from(json))
                .unwrap(),
            GarbleResponse::Streaming(streaming) => streaming.into_response(),
        }
    }
}

/// Create the optimal response for the given target size
pub fn create_optimal_response(target_size: usize) -> GarbleResponse {
    match ResponseStrategy::for_size(target_size) {
        ResponseStrategy::Direct => {
            let mut generator = RandomDataGenerator::new();
            let payload = generator.generate_payload(target_size);
            let json = serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string());
            GarbleResponse::Json(json)
        }
        ResponseStrategy::Fast => {
            let response = FastGarbleResponse::new(target_size).build();
            GarbleResponse::Json(response)
        }
        ResponseStrategy::Streaming => {
            let streaming = StreamingGarbleResponse::new(target_size);
            GarbleResponse::Streaming(streaming)
        }
    }
}

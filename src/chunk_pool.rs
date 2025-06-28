// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use once_cell::sync::Lazy;
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

use crate::generator::RandomDataGenerator;

/// Different chunk sizes we pre-generate
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChunkSize {
    Small,  // ~1KB
    Medium, // ~10KB
    Large,  // ~100KB
    XLarge, // ~1MB
}

impl ChunkSize {
    pub fn target_bytes(&self) -> usize {
        match self {
            ChunkSize::Small => 1_024,
            ChunkSize::Medium => 10_240,
            ChunkSize::Large => 102_400,
            ChunkSize::XLarge => 1_048_576,
        }
    }

    pub fn all() -> &'static [ChunkSize] {
        &[
            ChunkSize::Small,
            ChunkSize::Medium,
            ChunkSize::Large,
            ChunkSize::XLarge,
        ]
    }
}

/// Configuration for the chunk pool
#[derive(Debug, Clone)]
pub struct ChunkPoolConfig {
    pub max_memory_mb: usize,
    pub min_chunks_per_size: usize,
    #[allow(dead_code)] // Reserved for future use
    pub max_chunks_per_size: usize,
    pub background_generation_interval_ms: u64,
    #[allow(dead_code)] // Reserved for future use
    pub memory_check_interval_ms: u64,
}

impl Default for ChunkPoolConfig {
    fn default() -> Self {
        Self {
            max_memory_mb: 128, // 128MB max for chunk pool
            min_chunks_per_size: 5,
            max_chunks_per_size: 50,
            background_generation_interval_ms: 1000,
            memory_check_interval_ms: 5000,
        }
    }
}

/// A pool of pre-generated chunks for fast response assembly
pub struct ChunkPool {
    chunks: RwLock<HashMap<ChunkSize, Vec<String>>>,
    config: ChunkPoolConfig,
    stats: Mutex<ChunkPoolStats>,
    #[allow(dead_code)] // Reserved for future use
    last_generation: Mutex<Instant>,
}

#[derive(Debug, Default, Clone)]
pub struct ChunkPoolStats {
    pub total_chunks: usize,
    pub memory_usage_bytes: usize,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub background_generations: u64,
}

impl ChunkPool {
    pub fn new(config: ChunkPoolConfig) -> Self {
        let pool = Self {
            chunks: RwLock::new(HashMap::new()),
            config,
            stats: Mutex::new(ChunkPoolStats::default()),
            last_generation: Mutex::new(Instant::now()),
        };

        // Don't initialize anything here - just create the empty structure
        tracing::info!("ChunkPool struct created (no initialization yet)");
        pool
    }

    pub fn lazy_initialize(&self) {
        // Initialize empty vectors only when first needed
        let mut chunks = self.chunks.write().unwrap();

        if chunks.is_empty() {
            for &size in ChunkSize::all() {
                chunks.insert(size, Vec::new());
            }
            tracing::info!("Chunk pool lazy-initialized with empty vectors");
        }
    }

    /// Generate chunks in parallel for better performance
    /// These are JSON values that can be inserted into arrays
    fn generate_chunks_parallel(&self, size: ChunkSize, count: usize) -> Vec<String> {
        (0..count)
            .into_par_iter()
            .map(|_| {
                let mut generator = RandomDataGenerator::new();
                let chunk = generator.generate_array_element(size.target_bytes());
                // Generate as a JSON value that can be inserted into an array
                serde_json::to_string(&chunk)
                    .unwrap_or_else(|_| r#"{"error":"generation_failed"}"#.to_string())
            })
            .collect()
    }

    /// Get a chunk of the specified size
    pub fn get_chunk(&self, size: ChunkSize) -> Option<String> {
        // Ensure pool is initialized
        self.lazy_initialize();

        let mut chunks = self.chunks.write().unwrap();
        let chunk_vec = chunks.get_mut(&size)?;

        if let Some(chunk) = chunk_vec.pop() {
            // Update stats
            if let Ok(mut stats) = self.stats.lock() {
                stats.cache_hits += 1;
                stats.total_chunks = stats.total_chunks.saturating_sub(1);
            }
            Some(chunk)
        } else {
            // Cache miss - generate on demand
            if let Ok(mut stats) = self.stats.lock() {
                stats.cache_misses += 1;
            }
            None
        }
    }

    /// Get multiple chunks efficiently
    #[allow(dead_code)] // Reserved for future batch operations
    pub fn get_chunks(&self, size: ChunkSize, count: usize) -> Vec<String> {
        let mut chunks = self.chunks.write().unwrap();
        let mut default_vec = Vec::new();
        let chunk_vec = chunks.get_mut(&size).unwrap_or(&mut default_vec);

        let available = chunk_vec.len().min(count);
        let mut result = Vec::with_capacity(count);

        // Take available chunks from pool
        for _ in 0..available {
            if let Some(chunk) = chunk_vec.pop() {
                result.push(chunk);
            }
        }

        // Generate remaining chunks if needed
        let remaining = count - available;
        if remaining > 0 {
            let new_chunks = self.generate_chunks_parallel(size, remaining);
            result.extend(new_chunks);

            if let Ok(mut stats) = self.stats.lock() {
                stats.cache_misses += remaining as u64;
                stats.cache_hits += available as u64;
            }
        } else if let Ok(mut stats) = self.stats.lock() {
            stats.cache_hits += available as u64;
        }

        result
    }

    /// Build a response by combining chunks to reach target size
    pub fn build_response(&self, target_size: usize) -> String {
        // Ensure pool is initialized
        self.lazy_initialize();

        if target_size < ChunkSize::Small.target_bytes() {
            // For very small responses, generate directly
            let mut generator = RandomDataGenerator::new();
            let payload = generator.generate_payload(target_size);
            return serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string());
        }

        let mut result = String::with_capacity(target_size + 1024);
        let mut remaining = target_size;
        let mut chunk_count = 0;

        result.push_str(r#"{"garbled_chunks":["#);

        let mut first = true;
        while remaining > 500 && chunk_count < 1000 {
            // Safety limits
            if !first {
                result.push(',');
            }
            first = false;

            // Choose appropriate chunk size
            let chunk_size = if remaining >= ChunkSize::XLarge.target_bytes() {
                ChunkSize::XLarge
            } else if remaining >= ChunkSize::Large.target_bytes() {
                ChunkSize::Large
            } else if remaining >= ChunkSize::Medium.target_bytes() {
                ChunkSize::Medium
            } else {
                ChunkSize::Small
            };

            if let Some(chunk) = self.get_chunk(chunk_size) {
                result.push_str(&chunk);
                remaining = remaining.saturating_sub(chunk.len());
            } else {
                // Fallback: generate a small chunk
                let mut generator = RandomDataGenerator::new();
                let size = remaining.min(ChunkSize::Small.target_bytes());
                let payload = generator.generate_array_element(size);
                let chunk = serde_json::to_string(&payload)
                    .unwrap_or_else(|_| r#"{"fallback":true}"#.to_string());
                result.push_str(&chunk);
                remaining = remaining.saturating_sub(chunk.len());
            }

            chunk_count += 1;

            // Safety check to prevent infinite loops
            if result.len() > target_size * 2 {
                break;
            }
        }

        result.push_str(r#"],"metadata":{"generated_by":"chunk_pool","target_size":"#);
        result.push_str(&target_size.to_string());
        result.push_str(r#","actual_size":"#);
        result.push_str(&result.len().to_string());
        result.push_str(r#","chunk_count":"#);
        result.push_str(&chunk_count.to_string());
        result.push_str(r#"}}"#);

        result
    }

    /// Background task to maintain chunk pool
    pub async fn background_maintenance(&self) {
        // First, ensure the pool is initialized
        self.lazy_initialize();

        // Start with faster generation to populate the pool quickly
        let mut fast_startup = true;
        let mut startup_rounds = 0;

        tracing::info!("Background maintenance task starting...");

        loop {
            // Use shorter intervals during startup
            let interval_ms = if fast_startup {
                100 // 100ms during startup
            } else {
                self.config.background_generation_interval_ms
            };

            tokio::time::sleep(Duration::from_millis(interval_ms)).await;

            // Check if we should generate more chunks
            if self.should_generate_chunks() {
                tracing::debug!("Generating background chunks (startup: {})", fast_startup);
                self.generate_background_chunks().await;
                startup_rounds += 1;
            } else if fast_startup {
                // Pool is sufficiently populated, switch to normal mode
                fast_startup = false;
                tracing::info!(
                    "Chunk pool startup complete after {} rounds, switching to maintenance mode",
                    startup_rounds
                );
            }
        }
    }

    fn should_generate_chunks(&self) -> bool {
        // Check memory usage
        if !self.has_memory_available() {
            return false;
        }

        // Check if any chunk type is running low
        let chunks = self.chunks.read().unwrap();
        for &size in ChunkSize::all() {
            let count = chunks.get(&size).map(|v| v.len()).unwrap_or(0);
            if count < self.config.min_chunks_per_size {
                return true;
            }
        }

        false
    }

    async fn generate_background_chunks(&self) {
        let chunks_to_generate = {
            let chunks = self.chunks.read().unwrap();
            let mut needed = Vec::new();

            for &size in ChunkSize::all() {
                let current_count = chunks.get(&size).map(|v| v.len()).unwrap_or(0);
                if current_count < self.config.min_chunks_per_size {
                    // Generate only a few chunks at a time to avoid blocking
                    let needed_count = (self.config.min_chunks_per_size - current_count).min(3);
                    needed.push((size, needed_count));
                }
            }
            needed
        };

        if !chunks_to_generate.is_empty() {
            // Generate chunks one size at a time to avoid overwhelming the system
            for (size, count) in chunks_to_generate.into_iter().take(1) {
                // Only process one size per round
                tracing::debug!("Generating {} chunks of size {:?}", count, size);
                let new_chunks = self.generate_chunks_parallel(size, count);

                if let Ok(mut chunks) = self.chunks.write() {
                    chunks
                        .entry(size)
                        .or_insert_with(Vec::new)
                        .extend(new_chunks);
                }

                // Yield to allow other tasks to run
                tokio::task::yield_now().await;
            }

            self.update_stats();

            if let Ok(mut stats) = self.stats.lock() {
                stats.background_generations += 1;
            }
        }
    }

    fn has_memory_available(&self) -> bool {
        let current_usage = self.estimate_memory_usage();
        let max_bytes = self.config.max_memory_mb * 1024 * 1024;
        current_usage < max_bytes
    }

    fn estimate_memory_usage(&self) -> usize {
        let chunks = self.chunks.read().unwrap();
        chunks
            .values()
            .flat_map(|chunk_vec| chunk_vec.iter())
            .map(|chunk| chunk.len())
            .sum()
    }

    fn update_stats(&self) {
        if let Ok(mut stats) = self.stats.lock() {
            let chunks = self.chunks.read().unwrap();
            stats.total_chunks = chunks.values().map(|v| v.len()).sum();
            stats.memory_usage_bytes = self.estimate_memory_usage();
        }
    }

    pub fn get_stats(&self) -> ChunkPoolStats {
        self.stats.lock().unwrap().clone()
    }
}

// Global chunk pool instance
pub static CHUNK_POOL: Lazy<Arc<ChunkPool>> =
    Lazy::new(|| Arc::new(ChunkPool::new(ChunkPoolConfig::default())));

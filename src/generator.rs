// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use chrono::Utc;
use rand::prelude::*;
use serde_json::{Map, Value};
use uuid::Uuid;

pub struct RandomDataGenerator {
    rng: ThreadRng,
}

impl RandomDataGenerator {
    pub fn new() -> Self {
        Self { rng: thread_rng() }
    }

    pub fn generate_payload(&mut self, target_size: usize) -> Value {
        // Start with completely random structure - no fixed fields
        let mut payload = self.generate_random_object(3); // Start with depth 3

        // Keep adding random data until we reach target size
        let mut current_size = serde_json::to_string(&payload).unwrap().len();
        let mut iterations = 0;

        while current_size < target_size && iterations < 1000 {
            // Randomly decide what to add
            match self.rng.gen_range(0..4) {
                0 => {
                    // Add a completely random field to root
                    let key = self.generate_random_key();
                    let depth = self.rng.gen_range(1..6);
                    let value = self.generate_random_value(depth);
                    if let Value::Object(ref mut map) = payload {
                        map.insert(key, value);
                    }
                }
                1 => {
                    // Add random array
                    let key = self.generate_random_key();
                    let length = self.rng.gen_range(1..20);
                    let array = self.generate_random_array(length);
                    if let Value::Object(ref mut map) = payload {
                        map.insert(key, array);
                    }
                }
                2 => {
                    // Add nested random object
                    let key = self.generate_random_key();
                    let depth = self.rng.gen_range(1..5);
                    let obj = self.generate_random_object(depth);
                    if let Value::Object(ref mut map) = payload {
                        map.insert(key, obj);
                    }
                }
                _ => {
                    // Add random garbled data
                    let key = self.generate_random_key();
                    let garbled = self.generate_garbled_data();
                    if let Value::Object(ref mut map) = payload {
                        map.insert(key, garbled);
                    }
                }
            }

            current_size = serde_json::to_string(&payload).unwrap().len();
            iterations += 1;

            // Safety check to prevent infinite loops
            if current_size > target_size * 3 {
                break;
            }
        }

        payload
    }

    /// Generate a payload that's designed to be an array element (not a complete JSON object)
    pub fn generate_array_element(&mut self, target_size: usize) -> Value {
        // Generate various types of values that can go in an array
        let choice = self.rng.gen_range(0..6);
        match choice {
            0 => {
                let depth = self.rng.gen_range(1..4);
                self.generate_random_object(depth)
            }
            1 => {
                let length = self.rng.gen_range(1..10);
                self.generate_random_array(length)
            }
            2 => Value::String(self.generate_massive_garbled_string()),
            3 => self.generate_garbled_data(),
            4 => Value::Number(serde_json::Number::from(self.rng.gen::<i64>())),
            _ => {
                // Generate a payload and return it as-is
                self.generate_payload(target_size)
            }
        }
    }

    fn generate_random_object(&mut self, max_depth: usize) -> Value {
        let mut obj = Map::new();
        let field_count = self.rng.gen_range(1..15);

        for _ in 0..field_count {
            let key = self.generate_random_key();
            let value = if max_depth > 0 && self.rng.gen_bool(0.3) {
                // 30% chance of nested object
                self.generate_random_object(max_depth - 1)
            } else {
                self.generate_random_value(max_depth)
            };
            obj.insert(key, value);
        }

        Value::Object(obj)
    }

    fn generate_random_array(&mut self, max_length: usize) -> Value {
        let length = self.rng.gen_range(0..max_length);
        let mut array = Vec::new();

        for _ in 0..length {
            let depth = self.rng.gen_range(1..4);
            array.push(self.generate_random_value(depth));
        }

        Value::Array(array)
    }

    fn generate_random_value(&mut self, max_depth: usize) -> Value {
        match self.rng.gen_range(0..12) {
            0 => {
                let length = self.rng.gen_range(1..50);
                Value::String(self.generate_random_string(length))
            }
            1 => Value::Number(serde_json::Number::from(self.rng.gen::<i64>())),
            2 => Value::Number(
                serde_json::Number::from_f64(self.rng.gen::<f64>())
                    .unwrap_or_else(|| serde_json::Number::from(0)),
            ),
            3 => Value::Bool(self.rng.gen_bool(0.5)),
            4 => Value::Null,
            5 => Value::String(Uuid::new_v4().to_string()),
            6 => Value::String(self.generate_garbled_string()),
            7 => Value::String(format!("{}", Utc::now())),
            8 => {
                let length = self.rng.gen_range(1..10);
                self.generate_random_array(length)
            }
            9 => {
                if max_depth > 0 {
                    self.generate_random_object(max_depth - 1)
                } else {
                    Value::Null
                }
            }
            10 => Value::String(self.generate_hex_string()),
            _ => Value::String(self.generate_base64_like_string()),
        }
    }

    fn generate_random_key(&mut self) -> String {
        match self.rng.gen_range(0..8) {
            0 => {
                let length = self.rng.gen_range(3..20);
                self.generate_random_string(length)
            }
            1 => self.generate_garbled_string(),
            2 => format!(
                "{}_{}",
                self.generate_random_string(5),
                self.rng.gen::<u32>()
            ),
            3 => format!("field_{}", self.rng.gen::<u64>()),
            4 => self.generate_hex_string(),
            5 => format!(
                "{}_{}",
                self.generate_garbled_string(),
                self.generate_random_string(3)
            ),
            6 => Uuid::new_v4().to_string().replace("-", "_"),
            _ => format!("garbled_{}", self.generate_random_string(8)),
        }
    }

    fn generate_garbled_data(&mut self) -> Value {
        match self.rng.gen_range(0..6) {
            0 => {
                // Garbled object with random structure
                let mut obj = Map::new();
                for _ in 0..self.rng.gen_range(1..20) {
                    obj.insert(
                        self.generate_garbled_string(),
                        self.generate_random_value(2),
                    );
                }
                Value::Object(obj)
            }
            1 => {
                // Array of completely random stuff
                let mut array = Vec::new();
                for _ in 0..self.rng.gen_range(1..30) {
                    array.push(self.generate_random_value(1));
                }
                Value::Array(array)
            }
            2 => Value::String(self.generate_massive_garbled_string()),
            3 => {
                // Nested chaos
                let mut chaos = Map::new();
                chaos.insert("chaos".to_string(), self.generate_random_array(20));
                chaos.insert(
                    "mayhem".to_string(),
                    Value::String(self.generate_garbled_string()),
                );
                chaos.insert(
                    "disorder".to_string(),
                    Value::Number(serde_json::Number::from(self.rng.gen::<i64>())),
                );
                Value::Object(chaos)
            }
            4 => {
                // Mixed type array
                Value::Array(vec![
                    Value::String(self.generate_garbled_string()),
                    Value::Number(
                        serde_json::Number::from_f64(self.rng.gen::<f64>())
                            .unwrap_or_else(|| serde_json::Number::from(0)),
                    ),
                    Value::Bool(self.rng.gen_bool(0.5)),
                    Value::Null,
                    self.generate_random_object(1),
                ])
            }
            _ => Value::String(format!(
                "GARBLED_{}_{}_{}",
                self.rng.gen::<u64>(),
                self.generate_hex_string(),
                self.generate_random_string(10)
            )),
        }
    }

    fn generate_random_string(&mut self, length: usize) -> String {
        const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-";
        (0..length)
            .map(|_| {
                let idx = self.rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    fn generate_garbled_string(&mut self) -> String {
        // Truly garbled - mix of everything
        const GARBLED_CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()_+-=[]{}|;:,.<>?~`";
        let length = self.rng.gen_range(3..40);
        (0..length)
            .map(|_| {
                let idx = self.rng.gen_range(0..GARBLED_CHARS.len());
                GARBLED_CHARS[idx] as char
            })
            .collect()
    }

    fn generate_hex_string(&mut self) -> String {
        const HEX_CHARS: &[u8] = b"0123456789abcdef";
        let length = self.rng.gen_range(8..32);
        (0..length)
            .map(|_| {
                let idx = self.rng.gen_range(0..HEX_CHARS.len());
                HEX_CHARS[idx] as char
            })
            .collect()
    }

    fn generate_base64_like_string(&mut self) -> String {
        const BASE64_CHARS: &[u8] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=";
        let length = self.rng.gen_range(10..60);
        (0..length)
            .map(|_| {
                let idx = self.rng.gen_range(0..BASE64_CHARS.len());
                BASE64_CHARS[idx] as char
            })
            .collect()
    }

    fn generate_massive_garbled_string(&mut self) -> String {
        // For when we need to fill space quickly
        let segments = self.rng.gen_range(3..15);
        let mut result = String::new();

        for i in 0..segments {
            if i > 0 {
                result.push_str(&format!("_{}_", self.rng.gen::<u32>()));
            }
            result.push_str(&self.generate_garbled_string());

            // Sometimes add random data
            if self.rng.gen_bool(0.4) {
                result.push_str(&format!("_UUID_{}_", Uuid::new_v4()));
            }
            if self.rng.gen_bool(0.3) {
                result.push_str(&format!("_HEX_{}_", self.generate_hex_string()));
            }
        }

        result
    }
}

# Daddle

[![CI-Docker](https://img.shields.io/github/actions/workflow/status/johan-steffens/daddle/docker.yml?label=docker-build)](https://github.com/johan-steffens/daddle/actions/workflows/docker.yml)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-blue)](https://blog.rust-lang.org/2023/06/01/Rust-1.70.0.html)
[![Docker Version](https://img.shields.io/docker/v/johansteffens/daddle?label=docker%20tag)](https://hub.docker.com/r/johansteffens/daddle)
[![Docker Pulls](https://img.shields.io/docker/pulls/johansteffens/daddle)](https://hub.docker.com/r/johansteffens/daddle)
[![License](https://img.shields.io/github/license/johan-steffens/daddle)](https://github.com/johan-steffens/daddle/blob/main/LICENSE.md)

Daddle is a blazing-fast Rust-based HTTP service that generates completely random, chaotic, and truly "arbled JSON payloads of varying sizes with configurable wait durations. 

## 🚀 Key Features

### ⚡ **Extreme Performance**
- Optimized for high-throughput scenarios
- No blocking initialization, ready in <1 second
- Handles massive payloads without memory spikes
- Configurable memory limits and intelligent resource management

### 🎯 **Smart Performance Strategies**
- **< 10KB**: Direct generation (fastest for small payloads)
- **10KB - 1MB**: Chunk pool + parallel generation (optimized for medium payloads)
- **> 1MB**: Streaming generation (memory-efficient for large payloads)
- **Background generation**: Idle-time chunk creation with memory monitoring

### 🌪️ **Truly Garbled Data**
Generates completely random, unpredictable JSON structures including:
- Random nested objects with arbitrary depth
- Arrays of mixed data types (strings, numbers, booleans, nulls, objects)
- Completely random field names and values
- Garbled strings with special characters and symbols
- UUIDs, hex strings, base64-like data
- **No fixed structure** - every response is unique chaos

### ⚙️ **Ready to Go**
- **Configurable Response Sizes**: Generate JSON payloads between specified minimum and maximum sizes
- **Variable Wait Durations**: Simulate processing time with configurable sleep durations
- **Flexible Configuration**: Configure defaults via JSON file, override with query parameters
- **Health Monitoring**: Built-in health check and performance statistics endpoints
- **CORS Support**: Ready for cross-origin requests
- **Valid JSON**: All responses are syntactically correct JSON despite being completely garbled

## Quick Start

### Run with Docker

> **Prerequisites:** Docker 20.10+ installed  

Pull the multi-arch image:

```bash
docker pull johansteffens/daddle:latest
```

Run daddle, exposing port **3000**:

```bash
docker run --rm -p 8080:8080 johansteffens/daddle:latest
```

### Run from Source

> **Prerequisites:** Rust 1.70+ installed with Cargo

1. **Clone or navigate to the project directory**
   ```bash
   cd daddle
   ```

2. **Build the project**
   ```bash
   cargo build --release
   ```

3. **Run the service**
   ```bash
   cargo run
   ```

The service will start on `http://0.0.0.0:3000` by default.

## API Endpoints

### `/garble` - Generate Random Payload

Generates a random JSON payload with configurable size and wait duration.

**Method**: `GET`

**Query Parameters** (all optional):
- `minBodySize` - Minimum response body size in bytes
- `maxBodySize` - Maximum response body size in bytes  
- `minWaitDuration` - Minimum wait duration in milliseconds
- `maxWaitDuration` - Maximum wait duration in milliseconds

**Example Requests**:
```bash
# Basic request with default configuration
curl http://localhost:3000/garble

# Request with custom size range
curl "http://localhost:3000/garble?minBodySize=500&maxBodySize=2000"

# Request with custom wait duration
curl "http://localhost:3000/garble?minWaitDuration=100&maxWaitDuration=500"

# Request with all parameters
curl "http://localhost:3000/garble?minBodySize=1000&maxBodySize=5000&minWaitDuration=200&maxWaitDuration=800"
```

**Response Format**:
The response is completely garbled JSON with no fixed structure. Every response is unique and chaotic. Examples of what you might get:

```json
{
  "OFP%j[0qf2;TTI!*j0hO": {
    "2d5fe81f_fb7f_4704_b606_57b5a7aebfa4": {
      "05985b4d69617b3f38": {
        "0d00c17891b": "2025-06-29 07:42:46.242503 UTC",
        "2zdwG_230488105": [],
        "8c8df1c6_4098_43bf_b243_37a315d27f20": "IUIMXu2Q2sqS39Bva4g9mM/oolW4g5MlftyJoC/siju8",
        "FEAxE00-0oCUdKkEP": null,
        "SHNjzbpJeLcjwztE": [
          "wEU,AvI3{cRM6+z7%ii:D",
          -2542683836428308362,
          "j@pQ!l,j-~JjzX^VE4jpUB$*{}ra@Ni$O"
        ],
        "YUZuwuF-": 4319109914484698971,
        "ZMxO2iWNDj5bC-9brz0": 0.6490290830980666,
        "field_17737776738356680243": "UtXk0ehNKBcSAgc-m4AHzYNvKLXntwQuR7aL6LiyVHz",
        "field_4688877596922098470": null,
        "garbled_9gr7XW7i": "2025-06-29 07:42:46.242650 UTC",
        "garbled_j76dLRiw": "43fc0e5c578ab8a1c559c0fdc3"
      },
      "field_15692735315835169438": ">T~{MNo^_&l}FFQu:3N.HaWv8s:=9UM~&OI`{8%",
      "kGNSR_1701937921": {
        "-p1Tj_865286472": -4660943544106295363,
        "02a7707cfc90104c812cb0a1779e7": [
          null,
          8400274521474645370,
          0.315398570792483,
          7314428524892887461,
          9087646353759153137
        ],
        "33b334aa93da": "283c221ffc88fd2b7de12",
        "8562e042ab2618bed": "6198d4b9-0f94-4813-b6af-2745a4a15c4e",
        "8d22ec7d_1f37_4bf6_a5f6_4f3c82840717": -5964232166637025800,
        ">5<@e-ACHjm_TUjr`TBsK,GVOnFNf_RC~g#gN_IhW": null,
        "field_13617532672358591833": "2025-06-29 07:42:46.241118 UTC",
        "field_9782506042809497189": "91ca5aafb7e2",
        "garbled_oKjObcPI": 0.7278492899847163,
        "hIuxw_3434006643": true
      },
      "u7NT0Y9L-5Lj": "bOko5q/n6LjOu9Q2Hu6px+"
    },
    "kayl-qJQZXWJ": null
  },
  "bfa525d8_9f33_4b6c_9a03_2df7db40a517": "2025-06-29 07:42:46.240465 UTC",
  "f#,sV4P`OrVZ_Tg6": "2025-06-29 07:42:46.240432 UTC",
  "field_1494827627947597647": 3259602512930859468,
  "kPuO8_1114626528": "qaJ~8}*"
}
```

**Note**: The structure above is just an example. Every response will have completely different field names, nesting levels, data types, and values. No two responses will ever be the same!

### `/health` - Health Check

Returns service health status.

**Method**: `GET`

**Example**:
```bash
curl http://localhost:3000/health
```

### `/stats` - Performance Statistics

Returns chunk pool and performance statistics.

**Method**: `GET`

**Example**:
```bash
curl http://localhost:3000/stats
```

**Response includes**:
- Chunk pool memory usage and hit rates
- Background generation statistics
- Cache performance metrics

## Configuration

The service can be configured via the `config.json` file in the project root:

```json
{
  "server": {
    "host": "0.0.0.0",
    "port": 3000
  },
  "garble": {
    "min_body_size": 100,
    "max_body_size": 10000,
    "min_wait_duration_ms": 0,
    "max_wait_duration_ms": 1000
  }
}
```

### Configuration Options

#### Server Configuration
- **server.host**: IP address to bind to (default: "0.0.0.0")
- **server.port**: Port to listen on (default: 3000)

#### Garble Configuration
- **garble.min_body_size**: Default minimum response size in bytes (default: 100)
- **garble.max_body_size**: Default maximum response size in bytes (default: 10000)
- **garble.min_wait_duration_ms**: Default minimum wait time in milliseconds (default: 0)
- **garble.max_wait_duration_ms**: Default maximum wait time in milliseconds (default: 1000)

#### Performance Configuration
- **performance.chunk_pool_max_memory_mb**: Maximum memory for chunk pool in MB (default: 128)
- **performance.chunk_pool_min_chunks_per_size**: Minimum chunks per size category (default: 5)
- **performance.streaming_threshold_bytes**: Size threshold for streaming responses (default: 1MB)
- **performance.fast_response_threshold_bytes**: Size threshold for chunk pool usage (default: 10KB)
- **performance.background_generation_interval_ms**: Background generation interval (default: 1000ms)
- **performance.enable_parallel_generation**: Enable parallel chunk generation (default: true)

Query parameters override configuration file values for individual requests.

## Generated Data Types

Daddle generates completely random, garbled data including:

- **Random Objects**: Nested structures with arbitrary depth and random field names
- **Mixed Arrays**: Arrays containing random combinations of strings, numbers, booleans, nulls, and nested objects
- **Garbled Strings**: Random character combinations including special characters, symbols, and mixed case
- **Random Numbers**: Integers and floating-point numbers of all sizes
- **UUIDs and Hex**: Random identifiers and hexadecimal strings
- **Base64-like Data**: Random encoded-looking strings
- **Chaos Structures**: Completely unpredictable nested combinations of all the above

**Important**: There is NO fixed structure. Every field name, every value, every nesting level is completely random. The service truly lives up to its name - it's pure garbled chaos!

## Development

### Building
```bash
cargo build --release
```

### Running in Development
```bash
cargo run
```

### Running Tests
```bash
cargo test
```

### Code Quality
```bash
cargo fmt
cargo clippy
```

## License

This project is open source. See the [LICENSE](LICENSE.md) file for details.

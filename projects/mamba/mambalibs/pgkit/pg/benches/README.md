# data-bridge-postgres Benchmarks

Criterion-based benchmark suite for PostgreSQL operations.

## Benchmarks

### 1. Bulk Insert (1k rows)
- **Target**: <25ms
- **Description**: Inserts 1000 rows into the orders table
- **Measures**: Raw insertion throughput and connection pooling efficiency

### 2. Complex Query (JOIN + filter)
- **Target**: <20ms
- **Description**: Executes INNER JOIN between customers and orders tables with filtering and ordering
- **Measures**: Query builder overhead and JOIN performance

### 3. Serialization Overhead (10k rows)
- **Target**: <5ms
- **Description**: Fetches 10,000 rows and measures serialization to ExtractedValue
- **Measures**: Row-to-Rust conversion overhead

### 4. Query Builder Construction
- **No target** (pure CPU benchmark)
- **Description**: Measures query builder API overhead for simple and complex queries
- **Measures**: Builder pattern performance

## Prerequisites

1. PostgreSQL 12+ running and accessible
2. Database with appropriate permissions (CREATE/DROP tables)
3. Set environment variable:
   ```bash
   export POSTGRES_URL="postgresql://user:password@localhost:5432/bench_db"
   ```

## Running Benchmarks

```bash
# Run all benchmarks
POSTGRES_URL="postgresql://localhost/bench_db" cargo bench -p data-bridge-postgres

# Run specific benchmark
POSTGRES_URL="postgresql://localhost/bench_db" cargo bench -p data-bridge-postgres bulk_insert

# Save baseline for comparison
POSTGRES_URL="postgresql://localhost/bench_db" cargo bench -p data-bridge-postgres --save-baseline main

# Compare against baseline
POSTGRES_URL="postgresql://localhost/bench_db" cargo bench -p data-bridge-postgres --baseline main
```

## Configuration

Benchmarks use the following Criterion configuration:
- **Sample size**: 10 iterations
- **Measurement time**: 10 seconds per benchmark
- **Warm-up time**: 3 seconds

These settings balance accuracy with execution time for database-heavy operations.

## Test Database Setup

The benchmarks automatically create and destroy test tables:
- `customers`: Basic user information (id, name, email)
- `products`: Product catalog (id, name, price, stock)
- `orders`: Transaction records with foreign keys to customers and products

All tables are dropped after each benchmark completes.

## Performance Targets

| Benchmark | Target | Rationale |
|-----------|--------|-----------|
| Bulk Insert (1k) | <25ms | Should achieve ~40k inserts/sec |
| Complex Query | <20ms | JOIN + filter should be sub-20ms |
| Serialization (10k) | <5ms | ~2M rows/sec conversion rate |
| Query Builder | N/A | CPU-only, no database interaction |

## Notes

- Benchmarks require **network access** to PostgreSQL
- Performance varies with PostgreSQL version, hardware, and network latency
- Connection pooling (10 max connections) is used for realistic scenarios
- Measurements include network roundtrip time

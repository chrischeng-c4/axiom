# PostgreSQL Benchmarks

Comprehensive performance benchmarks comparing cclab-titan against other popular Python PostgreSQL libraries.

## Libraries Compared

1. **cclab-titan** - Our Rust-backed ORM with zero Python byte handling
2. **asyncpg** - High-performance async PostgreSQL driver
3. **psycopg2** - Traditional synchronous PostgreSQL driver
4. **SQLAlchemy** - Popular ORM with async support

## Benchmark Operations

- **Insert One**: Single row insert latency
- **Insert Bulk**: Bulk inserts (10, 100, 1000, 10000 rows)
- **Select One**: Find by primary key
- **Select Many**: Find with filter, limit 1000
- **Update One**: Update single row
- **Update Many**: Update multiple rows
- **Delete Many**: Delete multiple rows
- **Count**: Count with filter
- **Transaction**: Multiple operations in transaction

## Prerequisites

1. **PostgreSQL Server**: Running on localhost:5432 (or specify custom URI)
2. **Python Dependencies**:
   ```bash
   # Install required libraries
   uv pip install asyncpg psycopg2-binary sqlalchemy[asyncio] asyncpg
   ```

3. **Database**: Create benchmark database
   ```bash
   createdb cclab_benchmark
   ```

## Running Benchmarks

### Using dbtest (Recommended)

```bash
# Run all PostgreSQL benchmarks
POSTGRES_URI="postgresql://postgres:postgres@localhost:5432/cclab_benchmark" \
dbtest tests/postgres/benchmarks/

# Run specific benchmark file
dbtest tests/postgres/benchmarks/bench_insert.py

# Run with custom connection
POSTGRES_URI="postgresql://user:pass@localhost:5432/bench" \
dbtest tests/postgres/benchmarks/
```

### Using the comparison script

```bash
# Run comprehensive comparison
POSTGRES_URI="postgresql://postgres:postgres@localhost:5432/cclab_benchmark" \
uv run python benchmarks/bench_postgres_comparison.py

# With default connection (postgres/postgres@localhost:5432)
uv run python benchmarks/bench_postgres_comparison.py
```

### Using pytest

```bash
# Run all benchmarks
POSTGRES_URI="postgresql://postgres:postgres@localhost:5432/cclab_benchmark" \
pytest tests/postgres/benchmarks/ -v

# Run specific benchmark
pytest tests/postgres/benchmarks/bench_insert.py -v

# With benchmark plugin
pytest tests/postgres/benchmarks/ --benchmark-only
```

## Environment Variables

- `POSTGRES_URI`: PostgreSQL connection URI
  - Format: `postgresql://user:password@host:port/database`
  - Default: `postgresql://postgres:postgres@localhost:5432/cclab_benchmark`

## Performance Targets

Based on MongoDB performance gains (1.4-5.4x vs Beanie), we target:

| Operation | Target vs SQLAlchemy | Target vs asyncpg |
|-----------|---------------------|-------------------|
| Insert 1000 rows | ≥2.0x faster | ≥1.3x faster |
| Select 1000 rows | ≥1.5x faster | ≥1.2x faster |
| Update Many | ≥1.5x faster | ≥1.3x faster |
| Single ops | ≥1.3x faster | ≥1.1x faster |

## Benchmark Files

- `models.py`: Shared model definitions for all frameworks
- `conftest.py`: Pytest fixtures for connection management and test data
- `benchmark_setup.py`: Auto-initialization for dbtest
- `bench_insert.py`: Insert operation benchmarks
- `bench_find.py`: Select/find operation benchmarks
- `bench_update.py`: Update and delete operation benchmarks

## Expected Output

```
============================================================
PostgreSQL Performance Benchmarks
============================================================
Connection: postgresql://postgres@localhost:5432/cclab_benchmark
Database: cclab_benchmark
============================================================

Benchmarking: Insert One (100 iterations)
  cclab    :     0.85 ms
  asyncpg        :     1.12 ms
  psycopg2       :     1.45 ms
  sqlalchemy     :     2.31 ms

Benchmarking: Bulk Insert 1000 rows
  cclab    :    15.32 ms
  asyncpg        :    18.45 ms
  psycopg2       :    24.56 ms
  sqlalchemy     :    45.67 ms

============================================================
Benchmark Complete
============================================================

SUMMARY:

insert_one:
  cclab    :     0.85 ms  (2.72x)
  asyncpg        :     1.12 ms  (2.06x)
  psycopg2       :     1.45 ms  (1.59x)
  sqlalchemy     :     2.31 ms  (1.00x)

bulk_insert_1000:
  cclab    :    15.32 ms  (2.98x)
  asyncpg        :    18.45 ms  (2.48x)
  psycopg2       :    24.56 ms  (1.86x)
  sqlalchemy     :    45.67 ms  (1.00x)
```

## Troubleshooting

### Connection Issues

```bash
# Verify PostgreSQL is running
pg_isready

# Check connection
psql -U postgres -d cclab_benchmark -c "SELECT version();"
```

### Missing Dependencies

```bash
# Install all optional dependencies
uv pip install asyncpg psycopg2-binary "sqlalchemy[asyncio]" asyncpg
```

### Permission Issues

```bash
# Create user if needed
createuser -s postgres

# Grant permissions
psql -U postgres -c "GRANT ALL PRIVILEGES ON DATABASE cclab_benchmark TO postgres;"
```

## Contributing

When adding new benchmarks:

1. Follow the existing pattern in `bench_*.py` files
2. Use `BenchmarkGroup` and `register_group` from `cclab.test`
3. Add setup functions for data initialization
4. Include all 4 frameworks (cclab, asyncpg, psycopg2, SQLAlchemy)
5. Document expected performance targets
6. Add corresponding test cases if needed

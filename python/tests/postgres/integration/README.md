# PostgreSQL Integration Tests

This directory contains integration tests for cclab-titan that require a real PostgreSQL database.

## Prerequisites

- PostgreSQL running in Docker container `rstn-postgres` on port 5432
- Database credentials: `rstn:rstn`
- Test database: `cclab_test` (automatically created by setup script)

## Quick Start

```bash
# Run all integration tests (recommended)
bash scripts/run_integration_tests.sh

# Or manually with pytest
POSTGRES_URI="postgresql://rstn:rstn@localhost:5432/cclab_test" \
    uv run pytest tests/postgres/integration/ -v -m integration
```

## Test Files

### test_smoke.py
Basic smoke tests to verify infrastructure is working:
- Database connectivity
- Table creation and queries
- Fixture functionality
- Test isolation

**Status**: 7/7 tests passing ✅

### test_execute_integration.py
Integration tests for raw SQL execution:
- INSERT, SELECT, UPDATE, DELETE operations
- Parameterized queries
- NULL handling
- Type conversions
- Aggregate queries
- DDL operations
- Transaction support

**Status**: 9/11 tests passing (2 known issues)

## Test Fixtures

All fixtures are defined in `conftest.py`:

- **setup_test_database**: Session-scoped fixture that initializes the connection pool
- **cleanup_tables**: Function-scoped fixture that drops all tables after each test
- **test_table**: Creates a standard `test_users` table for testing
- **sample_data**: Inserts sample user data into the test table

## Running Specific Tests

```bash
# Smoke tests only
POSTGRES_URI="postgresql://rstn:rstn@localhost:5432/cclab_test" \
    uv run pytest tests/postgres/integration/test_smoke.py -v

# Single test
POSTGRES_URI="postgresql://rstn:rstn@localhost:5432/cclab_test" \
    uv run pytest tests/postgres/integration/test_smoke.py::test_database_connection -v
```

## Configuration

Default configuration (in conftest.py):
- **URI**: postgresql://rstn:rstn@localhost:5432/cclab_test
- **Min connections**: 2
- **Max connections**: 10

Override with environment variable:
```bash
POSTGRES_URI="postgresql://user:pass@host:port/db" uv run pytest ...
```

## Test Isolation

Tests are isolated through:
1. **Session-scoped connection**: Single connection pool shared across all tests
2. **Function-scoped cleanup**: All tables dropped after each test
3. **Fresh database**: Run `scripts/setup_test_db.sh` to reset database

## Known Issues

Two tests in `test_execute_integration.py` have known issues (not infrastructure-related):

1. **test_execute_insert_with_returning**: Test expects list but gets row count
2. **test_execute_aggregate_query**: Type conversion issue with DECIMAL/NUMERIC

These should be addressed in a separate task focused on the execute() function.

## Adding New Tests

1. Create test file in this directory
2. Add `@pytest.mark.integration` decorator to tests
3. Use fixtures from `conftest.py` as needed
4. Tests automatically get cleanup and connection handling

Example:
```python
import pytest
from cclab.titan import execute

@pytest.mark.integration
@pytest.mark.asyncio
async def test_my_feature(test_table):
    # Your test code here
    result = await execute(f"SELECT * FROM {test_table}")
    assert len(result) == 0
```

## Debugging

Run with verbose output and full tracebacks:
```bash
POSTGRES_URI="postgresql://rstn:rstn@localhost:5432/cclab_test" \
    uv run pytest tests/postgres/integration/ -vv --tb=long
```

Check PostgreSQL connection:
```bash
docker exec rstn-postgres psql -U rstn -d cclab_test -c "SELECT 1;"
```

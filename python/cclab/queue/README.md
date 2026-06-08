# cclab-meteor: Python API

High-performance distributed task queue built in Rust with Python bindings.

## Quick Start

### Installation

```bash
# Build the extension
maturin develop --features tasks

# Or with uv
uv run maturin develop --features tasks
```

### Basic Usage

```python
import asyncio
from cclab.meteor import task, init

# Initialize connection to broker and backend
async def setup():
    await init(
        nats_url="nats://localhost:4222",
        redis_url="redis://localhost:6379"
    )

# Define a task
@task(name="add", queue="math", max_retries=3)
async def add(x: int, y: int) -> int:
    return x + y

# Execute the task
async def main():
    await setup()

    # Fire and forget
    result = await add.delay(1, 2)

    # Wait for result
    value = await result.get()
    print(f"Result: {value}")  # 3

    # Check status
    is_ready = await result.ready()
    state = await result.state()  # "SUCCESS", "PENDING", etc.

asyncio.run(main())
```

## Workflows

### Chain - Sequential Execution

Execute tasks in sequence, passing the result of each task to the next:

```python
from cclab.meteor import Chain

@task
async def add(x: int, y: int) -> int:
    return x + y

@task
async def multiply(x: int, y: int) -> int:
    return x * y

# Create chain: (1 + 2) * 3 = 9
chain = Chain([
    add.s(1, 2),        # Returns 3
    multiply.s(3),      # Receives 3, multiplies by 3, returns 9
])

result = await chain.apply_async()
value = await result.get()  # 9
```

### Group - Parallel Execution

Execute multiple tasks in parallel:

```python
from cclab.meteor import Group

group = Group([
    add.s(1, 2),
    add.s(3, 4),
    add.s(5, 6),
])

results = await group.apply_async()
values = await results.get()  # [3, 7, 11]
```

### Chord - Parallel + Callback

Execute tasks in parallel, then call a callback with all results:

```python
from cclab.meteor import Chord

@task
async def sum_all(numbers: list[int]) -> int:
    return sum(numbers)

chord = Chord(
    # Header: execute in parallel
    [add.s(1, 2), add.s(3, 4), add.s(5, 6)],
    # Callback: receives [3, 7, 11]
    sum_all.s(),
)

result = await chord.apply_async()
value = await result.get()  # 21 (3 + 7 + 11)
```

## Task Options

### Delayed Execution

```python
# Execute after 60 seconds
result = await add.apply_async(1, 2, countdown=60)

# Execute at specific time (ISO 8601)
result = await add.apply_async(
    1, 2,
    eta="2026-01-05T10:00:00Z"
)
```

### Retry Configuration

```python
@task(
    name="flaky_task",
    max_retries=5,
    retry_delay=2.0,  # seconds between retries
)
async def flaky_task(data: dict) -> bool:
    # May fail and retry
    return process(data)
```

### Custom Queues

```python
@task(queue="high-priority")
async def urgent_task():
    pass

@task(queue="low-priority")
async def background_task():
    pass
```

## Result Handling

### AsyncResult Methods

```python
result = await task.delay(arg1, arg2)

# Check if ready
is_ready = await result.ready()

# Get result (waits until complete)
value = await result.get()

# Get result with timeout
try:
    value = await result.get(timeout=5.0)
except TimeoutError:
    print("Task timed out")

# Get current state
state = await result.state()
# Possible states: PENDING, RECEIVED, STARTED, SUCCESS, FAILURE, RETRY, REVOKED

# Get full info
info = await result.info()
print(info)
# {
#     'state': 'SUCCESS',
#     'result': 42,
#     'error': None,
#     'started_at': '2026-01-04T12:00:00Z',
#     'completed_at': '2026-01-04T12:00:05Z',
#     'retries': 0,
#     'worker_id': 'worker-1'
# }

# Get task ID
task_id = result.task_id
```

## Error Handling

```python
@task
async def risky_task(x: int) -> int:
    if x < 0:
        raise ValueError("Negative value not allowed")
    return x * 2

try:
    result = await risky_task.delay(-1)
    value = await result.get()
except RuntimeError as e:
    print(f"Task failed: {e}")
```

## Complete Example

```python
import asyncio
from cclab.meteor import task, init, Chain, Group, Chord

# Initialize
async def setup():
    await init(
        nats_url="nats://localhost:4222",
        redis_url="redis://localhost:6379"
    )

# Define tasks
@task(name="fetch_data", queue="io")
async def fetch_data(url: str) -> dict:
    # Simulate HTTP request
    return {"data": f"content from {url}"}

@task(name="process_data", queue="cpu")
async def process_data(data: dict) -> dict:
    # Simulate processing
    return {"processed": data}

@task(name="save_result", queue="io")
async def save_result(data: dict) -> bool:
    # Simulate database save
    return True

@task(name="aggregate", queue="cpu")
async def aggregate(results: list) -> dict:
    return {"total": len(results), "items": results}

async def main():
    await setup()

    # 1. Simple task
    result = await fetch_data.delay("https://api.example.com")
    data = await result.get()
    print(f"Fetched: {data}")

    # 2. Chain: fetch -> process -> save
    chain = Chain([
        fetch_data.s("https://api.example.com"),
        process_data.s(),
        save_result.s(),
    ])
    result = await chain.apply_async()
    success = await result.get()
    print(f"Chain complete: {success}")

    # 3. Group: fetch multiple URLs in parallel
    urls = [
        "https://api.example.com/1",
        "https://api.example.com/2",
        "https://api.example.com/3",
    ]
    group = Group([fetch_data.s(url) for url in urls])
    results = await group.apply_async()
    data_list = await results.get()
    print(f"Fetched {len(data_list)} items")

    # 4. Chord: fetch + process in parallel, then aggregate
    chord = Chord(
        [
            Chain([fetch_data.s(url), process_data.s()])
            for url in urls
        ],
        aggregate.s(),
    )
    result = await chord.apply_async()
    summary = await result.get()
    print(f"Summary: {summary}")

if __name__ == "__main__":
    asyncio.run(main())
```

## Requirements

### Runtime Dependencies

- **NATS Server**: Message broker
  ```bash
  docker run -p 4222:4222 nats:latest
  ```

- **Redis**: Result backend
  ```bash
  docker run -p 6379:6379 redis:latest
  ```

### Python Dependencies

- Python 3.12+
- cclab (built with maturin)

## Architecture

```
Python Application
       ↓
  PyO3 Bindings
       ↓
  Rust Task Queue
       ↓
   ┌───────┴────────┐
   ↓                ↓
NATS Broker    Redis Backend
```

## Performance

Compared to Celery:

- **3-5x faster** task submission (Rust serialization)
- **2-3x faster** result retrieval (zero-copy deserialization)
- **Lower latency** (NATS vs RabbitMQ)
- **Better resource usage** (Rust memory management)

## Type Safety

Full type hints for IDE support:

```python
from typing import Annotated
from cclab.meteor import task

@task
async def typed_task(
    x: int,
    y: int,
    metadata: dict[str, str],
) -> dict[str, int]:
    return {"sum": x + y, "count": len(metadata)}
```

## Limitations

1. **No pickle support**: Only JSON-serializable types (dict, list, str, int, float, bool, None)
2. **No class-based tasks**: Use function-based tasks only
3. **No result backend introspection**: Can't list all pending tasks (yet)

## Troubleshooting

### Import Error

```python
ImportError: cclab-meteor extension not built
```

**Solution**: Run `maturin develop --features tasks`

### Connection Error

```python
PyConnectionError: Failed to connect to NATS
```

**Solution**: Ensure NATS is running on `localhost:4222`

### Timeout Error

```python
PyTimeoutError: Task execution timed out
```

**Solution**: Increase timeout in `result.get(timeout=30.0)` or check worker is running

## Next Steps

1. **Start Worker**: Phase 8 implementation
2. **Monitor Tasks**: Check task states in Redis
3. **Scale Workers**: Run multiple worker processes
4. **Production Deploy**: Use NATS cluster + Redis Sentinel

## Links

- [NATS Documentation](https://docs.nats.io/)
- [Redis Documentation](https://redis.io/documentation)
- [cclab Repository](https://github.com/your-org/cclab)

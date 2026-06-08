# FastAPI + OpenTelemetry Quick Start

Get started with distributed tracing in 5 minutes.

## Prerequisites

- Python 3.12+
- Docker (for PostgreSQL and Jaeger)
- pip or uv

## Quick Start (Local Development)

### Step 1: Install Dependencies

```bash
pip install fastapi uvicorn pydantic[email] \
            opentelemetry-api opentelemetry-sdk \
            opentelemetry-instrumentation-fastapi \
            opentelemetry-exporter-otlp-proto-grpc
```

### Step 2: Start Infrastructure

```bash
# Start PostgreSQL
docker run -d --name postgres \
    -e POSTGRES_DB=fastapi_demo \
    -e POSTGRES_USER=postgres \
    -e POSTGRES_PASSWORD=postgres \
    -p 5432:5432 \
    postgres:16

# Start Jaeger
docker run -d --name jaeger \
    -e COLLECTOR_OTLP_ENABLED=true \
    -p 16686:16686 \
    -p 4317:4317 \
    jaegertracing/all-in-one:latest
```

### Step 3: Build cclab

```bash
# From project root
maturin develop --release
```

### Step 4: Run the API

```bash
python examples/fastapi_otel_example.py
```

Expected output:
```
OpenTelemetry configured:
  - Service: fastapi-databridge-api v1.0.0
  - Environment: development
  - OTLP Endpoint: http://localhost:4317
  - Insecure: True
  - cclab tracing: True

Connected to PostgreSQL: localhost:5432/fastapi_demo
Database tables ready

INFO:     Started server process [12345]
INFO:     Waiting for application startup.
INFO:     Application startup complete.
INFO:     Uvicorn running on http://0.0.0.0:8000
```

### Step 5: Test the API

In another terminal:

```bash
# Run test suite
python examples/test_fastapi_otel.py
```

Or manually test endpoints:

```bash
# Create a user
curl -X POST http://localhost:8000/users \
     -H "Content-Type: application/json" \
     -d '{"name": "Alice", "email": "alice@example.com", "age": 30}'

# List users
curl http://localhost:8000/users

# Create a post
curl -X POST http://localhost:8000/posts \
     -H "Content-Type: application/json" \
     -d '{"title": "Hello World", "content": "My first post", "author_id": 1}'

# List posts with eager loading
curl "http://localhost:8000/posts?eager=true"
```

### Step 6: View Traces

1. Open Jaeger UI: http://localhost:16686
2. Select service: `fastapi-databridge-api`
3. Click "Find Traces"
4. Click on any trace to see details

## Expected Trace Structure

```
HTTP Request: GET /users/1
│
├── FastAPI Handler
│   ├── span: GET /users/{user_id}
│   ├── http.method: GET
│   ├── http.route: /users/{user_id}
│   └── http.status_code: 200
│
└── cclab ORM
    ├── Query Span
    │   ├── span: db.query.find
    │   ├── db.system: postgresql
    │   ├── db.collection.name: users
    │   ├── db.operation.name: find
    │   ├── db.query.filters_count: 1
    │   └── db.result.count: 1
    │
    └── Relationship Span
        ├── span: db.relationship.load
        ├── db.relationship.name: posts
        ├── db.relationship.strategy: lazy
        └── db.result.count: 3
```

## Stopping Infrastructure

```bash
# Stop and remove containers
docker stop postgres jaeger
docker rm postgres jaeger
```

## Docker Compose (Alternative)

Instead of running containers manually:

```bash
# Start everything
cd examples
docker-compose up -d

# View logs
docker-compose logs -f api

# Stop everything
docker-compose down -v
```

## Troubleshooting

### Issue: "Could not connect to PostgreSQL"

**Solution**: Make sure PostgreSQL is running:
```bash
docker ps | grep postgres
```

If not running, start it:
```bash
docker start postgres
```

### Issue: "No traces appearing in Jaeger"

**Solution**: Check:
1. Jaeger is running: `docker ps | grep jaeger`
2. OTLP endpoint is correct: `echo $OTEL_EXPORTER_OTLP_ENDPOINT`
3. Tracing is enabled: `curl http://localhost:8000/ | jq .tracing`

### Issue: "ModuleNotFoundError: No module named 'cclab'"

**Solution**: Build cclab first:
```bash
cd /path/to/cclab-posgres
maturin develop --release
```

### Issue: "Port 8000 already in use"

**Solution**: Change port in code or kill existing process:
```bash
# Find process
lsof -i :8000

# Kill process
kill -9 <PID>
```

### Issue: "OpenTelemetry not installed"

**Solution**: Install OpenTelemetry SDK:
```bash
pip install opentelemetry-api opentelemetry-sdk \
            opentelemetry-instrumentation-fastapi \
            opentelemetry-exporter-otlp-proto-grpc
```

## Next Steps

1. **Explore API**: http://localhost:8000/docs
2. **View Traces**: http://localhost:16686
3. **Read Full Docs**: [README_FASTAPI_OTEL.md](./README_FASTAPI_OTEL.md)
4. **Try Different Backends**: [otel_backends.env.example](./otel_backends.env.example)

## Performance Tips

For production use:

1. **Enable Sampling** (reduce overhead):
   ```python
   from opentelemetry.sdk.trace.sampling import TraceIdRatioBased
   sampler = TraceIdRatioBased(0.1)  # 10% sampling
   ```

2. **Use Eager Loading** (avoid N+1):
   ```python
   query = session.find(Post).options(selectinload(Post.author))
   ```

3. **Increase Batch Size**:
   ```python
   processor = BatchSpanProcessor(
       otlp_exporter,
       max_export_batch_size=1024,
   )
   ```

## Resources

- [Full Documentation](./README_FASTAPI_OTEL.md)
- [OpenTelemetry Python](https://opentelemetry.io/docs/instrumentation/python/)
- [FastAPI](https://fastapi.tiangolo.com/)
- [Jaeger](https://www.jaegertracing.io/docs/)
- [cclab](../README.md)

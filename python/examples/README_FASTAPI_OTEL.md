# FastAPI + cclab + OpenTelemetry Integration

Production-ready example demonstrating distributed tracing with FastAPI, cclab PostgreSQL ORM, and OpenTelemetry.

## Overview

This example shows how to build a cloud-native API with complete observability:

- **FastAPI**: Modern Python web framework with automatic OpenAPI docs
- **cclab**: High-performance Rust-backed PostgreSQL ORM
- **OpenTelemetry**: Vendor-neutral telemetry (traces, metrics, logs)
- **OTLP Export**: Compatible with Jaeger, Grafana Cloud, DataDog, and more

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     HTTP Request                            │
└─────────────────┬───────────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────────────────┐
│         FastAPI (Auto-instrumented)                         │
│         - HTTP span (method, path, status)                  │
│         - Request/response metadata                         │
└─────────────────┬───────────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────────────────┐
│         cclab ORM                                     │
│         - Session spans (flush, commit)                     │
│         - Query spans (find, insert, update, delete)        │
│         - Relationship spans (lazy/eager loading)           │
└─────────────────┬───────────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────────────────┐
│         PostgreSQL Database                                 │
│         - SQL execution                                     │
│         - Connection pool                                   │
└─────────────────────────────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────────────────┐
│         OTLP Exporter (gRPC)                                │
│         - Batch span processor                              │
│         - Async export                                      │
└─────────────────┬───────────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────────────────┐
│         Observability Backend                               │
│         - Jaeger (local)                                    │
│         - Grafana Cloud (SaaS)                              │
│         - DataDog (APM)                                     │
│         - Any OTLP-compatible backend                       │
└─────────────────────────────────────────────────────────────┘
```

## Features Demonstrated

### 1. Distributed Tracing
- Parent-child span hierarchy (HTTP → ORM → DB)
- Trace context propagation across services
- Cross-cutting request correlation

### 2. ORM Instrumentation
- **Query Spans**: Operation type, table name, filters, pagination
- **Session Spans**: Pending/dirty/deleted counts, transaction boundaries
- **Relationship Spans**: Loading strategy (lazy vs eager), N+1 query detection

### 3. Performance Insights
- Query duration and count
- Connection pool utilization
- N+1 query identification
- Eager vs lazy loading comparison

### 4. Error Tracking
- Exception details in spans
- Stack traces and error messages
- HTTP status code correlation

## Quick Start

### Option 1: Docker Compose (Recommended)

```bash
# Start infrastructure (PostgreSQL + Jaeger)
cd examples
docker-compose up -d

# View logs
docker-compose logs -f api

# Stop infrastructure
docker-compose down -v
```

Access:
- API: http://localhost:8000
- API Docs: http://localhost:8000/docs
- Jaeger UI: http://localhost:16686

### Option 2: Local Development

```bash
# 1. Install dependencies
pip install fastapi uvicorn pydantic[email] \
            opentelemetry-api opentelemetry-sdk \
            opentelemetry-instrumentation-fastapi \
            opentelemetry-exporter-otlp-proto-grpc \
            cclab

# 2. Start PostgreSQL
docker run -d --name postgres \
    -e POSTGRES_DB=fastapi_demo \
    -e POSTGRES_USER=postgres \
    -e POSTGRES_PASSWORD=postgres \
    -p 5432:5432 \
    postgres:16

# 3. Start Jaeger
docker run -d --name jaeger \
    -e COLLECTOR_OTLP_ENABLED=true \
    -p 16686:16686 \
    -p 4317:4317 \
    -p 4318:4318 \
    jaegertracing/all-in-one:latest

# 4. Run the application
python examples/fastapi_otel_example.py
```

## API Endpoints

### Health Check
```bash
curl http://localhost:8000/
```

### Create User
```bash
curl -X POST http://localhost:8000/users \
     -H "Content-Type: application/json" \
     -d '{
       "name": "Alice",
       "email": "alice@example.com",
       "age": 30
     }'
```

### List Users
```bash
# Basic list
curl http://localhost:8000/users

# With pagination
curl "http://localhost:8000/users?limit=10&offset=0"
```

### Get Single User
```bash
curl http://localhost:8000/users/1
```

### Create Post
```bash
curl -X POST http://localhost:8000/posts \
     -H "Content-Type: application/json" \
     -d '{
       "title": "My First Post",
       "content": "Hello, World!",
       "author_id": 1
     }'
```

### List Posts
```bash
# Lazy loading (N+1 queries - demonstrates span hierarchy)
curl http://localhost:8000/posts

# Eager loading (optimized - single query)
curl "http://localhost:8000/posts?eager=true"
```

### Get Single Post
```bash
curl http://localhost:8000/posts/1
```

### Delete User
```bash
curl -X DELETE http://localhost:8000/users/1
```

## Viewing Traces

### Jaeger UI (Local)

1. Open http://localhost:16686
2. Select service: `fastapi-databridge-api`
3. Click "Find Traces"
4. Click on any trace to see span details

**Expected Span Hierarchy**:
```
GET /users/{user_id}                     [HTTP span]
├── db.query.find                        [Query span]
│   ├── db.system: postgresql
│   ├── db.collection.name: users
│   ├── db.operation.name: find
│   └── db.result.count: 1
└── db.relationship.load                 [Relationship span]
    ├── db.relationship.name: posts
    ├── db.relationship.strategy: lazy
    └── db.result.count: 5
```

### Grafana Cloud

```bash
# Set environment variables
export OTEL_EXPORTER_OTLP_ENDPOINT=https://otlp-gateway-prod-us-central-0.grafana.net/otlp
export OTEL_EXPORTER_OTLP_HEADERS="Authorization=Basic $(echo -n 'instance_id:token' | base64)"
export OTEL_EXPORTER_OTLP_INSECURE=false

# Run application
python examples/fastapi_otel_example.py
```

### DataDog APM

```bash
# Start DataDog agent with OTLP
docker run -d --name datadog-agent \
    -e DD_API_KEY=<your-api-key> \
    -e DD_SITE=datadoghq.com \
    -e DD_OTLP_CONFIG_RECEIVER_PROTOCOLS_GRPC_ENDPOINT=0.0.0.0:4317 \
    -p 4317:4317 \
    gcr.io/datadoghq/agent:latest

# Run application
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
python examples/fastapi_otel_example.py
```

## Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `SERVICE_NAME` | `fastapi-databridge-api` | Service name in traces |
| `SERVICE_VERSION` | `1.0.0` | Service version |
| `DEPLOYMENT_ENVIRONMENT` | `development` | Environment (dev/staging/prod) |
| `DATABASE_URL` | `postgresql://...` | PostgreSQL connection URL |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | `http://localhost:4317` | OTLP collector endpoint |
| `OTEL_EXPORTER_OTLP_INSECURE` | `true` | Use insecure gRPC (HTTP) |
| `OTEL_EXPORTER_OTLP_HEADERS` | `None` | Custom headers (e.g., auth) |
| `DATA_BRIDGE_TRACING_ENABLED` | `true` | Enable cclab tracing |

### Sampling Configuration

For production, configure sampling to reduce overhead:

```python
from opentelemetry.sdk.trace.sampling import TraceIdRatioBased

# Sample 10% of traces
sampler = TraceIdRatioBased(0.1)
provider = TracerProvider(sampler=sampler, resource=resource)
```

## Performance Considerations

### 1. Batch Span Processor

The example uses `BatchSpanProcessor` which:
- Batches spans before export
- Reduces network overhead
- Exports asynchronously (non-blocking)

Configuration:
```python
processor = BatchSpanProcessor(
    otlp_exporter,
    max_queue_size=2048,        # Max spans in queue
    schedule_delay_millis=5000,  # Export interval
    max_export_batch_size=512,   # Spans per batch
)
```

### 2. Span Cardinality

Avoid high-cardinality attributes:
- ❌ Don't: Full SQL statements with values
- ✅ Do: Operation types and table names
- ❌ Don't: User IDs in span names
- ✅ Do: Generic operation names

### 3. Connection Pool Metrics

Monitor connection pool health:
```python
from cclab.titan.telemetry import get_connection_pool_metrics

metrics = get_connection_pool_metrics()
metrics.record_pool_stats(in_use=5, idle=3, max_size=10)
```

## Troubleshooting

### No Traces Appearing

1. Check OpenTelemetry is enabled:
   ```bash
   curl http://localhost:8000/
   # Look for "tracing": true
   ```

2. Verify OTLP endpoint:
   ```bash
   curl http://localhost:4317
   # Should connect (may return error, but connection works)
   ```

3. Check cclab tracing:
   ```bash
   echo $DATA_BRIDGE_TRACING_ENABLED
   # Should be "true" or empty (defaults to true)
   ```

4. Check Jaeger logs:
   ```bash
   docker logs jaeger
   ```

### High Overhead

1. Reduce sampling rate:
   ```python
   sampler = TraceIdRatioBased(0.1)  # 10%
   ```

2. Disable debug logging:
   ```python
   import logging
   logging.getLogger("opentelemetry").setLevel(logging.WARNING)
   ```

3. Increase batch size:
   ```python
   processor = BatchSpanProcessor(
       otlp_exporter,
       max_export_batch_size=1024,
   )
   ```

## Best Practices

### 1. Resource Attributes

Always set service metadata:
```python
resource = Resource.create({
    SERVICE_NAME: "my-service",
    SERVICE_VERSION: "1.0.0",
    DEPLOYMENT_ENVIRONMENT: "production",
    "service.instance.id": os.environ.get("HOSTNAME"),
})
```

### 2. Error Handling

Exceptions are automatically recorded:
```python
@app.get("/users/{user_id}")
async def get_user(user_id: int):
    user = await session.find(User).filter(User.id == user_id).first()
    if user is None:
        # This exception will be recorded in the span
        raise HTTPException(status_code=404, detail="User not found")
```

### 3. Eager Loading

Use eager loading to avoid N+1 queries:
```python
# ❌ Bad: Lazy loading (N+1 queries)
posts = await session.find(Post).all()
for post in posts:
    print(post.author.name)  # Creates N spans

# ✅ Good: Eager loading (1 query)
posts = await session.find(Post).options(selectinload(Post.author)).all()
for post in posts:
    print(post.author.name)  # No additional spans
```

### 4. Custom Spans

Add custom spans for business logic:
```python
from cclab.titan.telemetry import instrument_span

@instrument_span("business.validate_order")
async def validate_order(order_id: int):
    # Your business logic
    pass
```

## Security Considerations

### 1. Credentials in Traces

Never include sensitive data in spans:
- ❌ Passwords, API keys, tokens
- ❌ Credit card numbers
- ❌ Personal identifiable information (PII)

### 2. OTLP Authentication

For production, use authenticated OTLP:
```bash
export OTEL_EXPORTER_OTLP_HEADERS="Authorization=Bearer <token>"
export OTEL_EXPORTER_OTLP_INSECURE=false
```

### 3. Network Security

Use TLS for OTLP export:
```python
from opentelemetry.exporter.otlp.proto.grpc.trace_exporter import OTLPSpanExporter

otlp_exporter = OTLPSpanExporter(
    endpoint="https://your-collector:4317",
    insecure=False,  # Use TLS
    credentials=ChannelCredentials(...),
)
```

## References

- [OpenTelemetry Python SDK](https://opentelemetry.io/docs/instrumentation/python/)
- [FastAPI Documentation](https://fastapi.tiangolo.com/)
- [cclab PostgreSQL ORM](../README.md)
- [Jaeger Documentation](https://www.jaegertracing.io/docs/)
- [OTLP Specification](https://opentelemetry.io/docs/specs/otlp/)

## License

MIT License - See [LICENSE](../LICENSE) for details.

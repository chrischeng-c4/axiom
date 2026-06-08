# FastAPI + OpenTelemetry Integration - Complete Package

This directory contains a production-ready example demonstrating distributed tracing with FastAPI, cclab PostgreSQL ORM, and OpenTelemetry.

## Files Overview

### Main Application
- **fastapi_otel_example.py** (18KB)
  - Complete FastAPI application with OpenTelemetry integration
  - User and Post models with relationships
  - Full CRUD operations
  - Lazy and eager loading examples
  - Production-ready configuration

### Testing
- **test_fastapi_otel.py** (12KB, executable)
  - Automated test suite for the API
  - Creates sample data (users and posts)
  - Tests all endpoints
  - Generates traces for Jaeger

### Documentation
- **QUICKSTART_FASTAPI_OTEL.md** (5.4KB)
  - 5-minute quick start guide
  - Step-by-step setup instructions
  - Common troubleshooting issues

- **README_FASTAPI_OTEL.md** (13KB)
  - Comprehensive documentation
  - Architecture overview
  - Configuration options
  - Performance considerations
  - Security best practices

- **TRACE_EXAMPLES.md** (16KB)
  - Visual trace examples
  - Expected span hierarchies
  - Performance analysis patterns
  - N+1 query detection
  - Tips for Jaeger UI

### Infrastructure
- **docker-compose.yml** (2KB)
  - Complete infrastructure setup
  - PostgreSQL database
  - Jaeger all-in-one
  - FastAPI application
  - Health checks and dependencies

- **Dockerfile** (851B)
  - Multi-stage build for API service
  - Rust toolchain included
  - Optimized for production

### Configuration
- **otel_backends.env.example** (5.2KB)
  - Configuration examples for 10+ backends:
    - Jaeger (local)
    - Grafana Cloud
    - DataDog APM
    - New Relic
    - Honeycomb.io
    - Lightstep
    - AWS X-Ray
    - Azure Monitor
    - Google Cloud Trace
    - Self-hosted collectors

## Quick Start

### Option 1: Docker Compose (Recommended)

```bash
cd examples
docker-compose up -d
docker-compose logs -f api
```

Access:
- API: http://localhost:8000
- Docs: http://localhost:8000/docs
- Jaeger: http://localhost:16686

### Option 2: Local Development

```bash
# Install dependencies
pip install fastapi uvicorn pydantic[email] \
            opentelemetry-api opentelemetry-sdk \
            opentelemetry-instrumentation-fastapi \
            opentelemetry-exporter-otlp-proto-grpc

# Start infrastructure
docker run -d --name postgres \
    -e POSTGRES_DB=fastapi_demo \
    -e POSTGRES_USER=postgres \
    -e POSTGRES_PASSWORD=postgres \
    -p 5432:5432 postgres:16

docker run -d --name jaeger \
    -e COLLECTOR_OTLP_ENABLED=true \
    -p 16686:16686 -p 4317:4317 \
    jaegertracing/all-in-one:latest

# Build cclab
maturin develop --release

# Run application
python examples/fastapi_otel_example.py

# Test (in another terminal)
python examples/test_fastapi_otel.py
```

## Features Demonstrated

### 1. OpenTelemetry Integration
- Tracer configuration with OTLP exporter
- Resource attributes (service name, version, environment)
- FastAPI automatic instrumentation
- cclab ORM spans

### 2. Distributed Tracing
- Parent-child span hierarchy
- Trace context propagation
- Cross-service correlation

### 3. ORM Instrumentation
- Query spans (find, insert, update, delete)
- Session spans (flush, commit)
- Relationship spans (lazy vs eager loading)
- Connection pool metrics

### 4. Performance Patterns
- N+1 query detection
- Eager loading optimization
- Pagination tracking
- Error handling

### 5. Multiple Backends
- Jaeger (local development)
- Cloud SaaS providers (Grafana, DataDog, etc.)
- Self-hosted collectors
- Easy configuration switching

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     HTTP Request                            │
│                  (trace_id: abc123)                         │
└─────────────────┬───────────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────────────────┐
│         FastAPI (Auto-instrumented)                         │
│         Span: GET /users/1                                  │
│         http.method, http.route, http.status_code           │
└─────────────────┬───────────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────────────────┐
│         cclab PostgreSQL ORM                          │
│         Span: db.query.find                                 │
│         db.system, db.collection.name, db.operation.name    │
└─────────────────┬───────────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────────────────┐
│         PostgreSQL Database                                 │
│         SQL Execution                                       │
└─────────────────┬───────────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────────────────┐
│         OTLP Exporter (gRPC)                                │
│         BatchSpanProcessor                                  │
└─────────────────┬───────────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────────────────┐
│         Observability Backend                               │
│         Jaeger / Grafana / DataDog / etc.                   │
└─────────────────────────────────────────────────────────────┘
```

## API Endpoints

| Method | Endpoint | Description | Demonstrates |
|--------|----------|-------------|--------------|
| GET | `/` | Health check | Service info |
| GET | `/users` | List users | Query spans, N+1 pattern |
| GET | `/users/{id}` | Get user | Lazy loading |
| POST | `/users` | Create user | Insert span, session |
| GET | `/posts` | List posts | Lazy vs eager loading |
| GET | `/posts/{id}` | Get post | Relationship loading |
| POST | `/posts` | Create post | Complex operation |
| DELETE | `/users/{id}` | Delete user | Cascade operations |

## Expected Trace Structure

```
GET /users/1 (18ms)
│
├─ db.query.find (7ms)
│  ├─ db.system: postgresql
│  ├─ db.collection.name: users
│  ├─ db.operation.name: find
│  ├─ db.query.filters_count: 1
│  └─ db.result.count: 1
│
└─ db.relationship.load (8ms)
   ├─ db.relationship.name: posts
   ├─ db.relationship.strategy: lazy
   └─ db.result.count: 5
```

## Performance Comparison

### Lazy Loading (N+1 Pattern)
```
GET /posts (10 posts)
Total queries: 11 (1 + 10)
Duration: 50ms+
```

### Eager Loading (Optimized)
```
GET /posts?eager=true (10 posts)
Total queries: 2 (1 + 1 batch)
Duration: 12ms
Improvement: 4x faster
```

## Configuration Examples

### Local Development (Jaeger)
```bash
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
export OTEL_EXPORTER_OTLP_INSECURE=true
```

### Grafana Cloud
```bash
export OTEL_EXPORTER_OTLP_ENDPOINT=https://otlp-gateway-prod-us-central-0.grafana.net/otlp
export OTEL_EXPORTER_OTLP_INSECURE=false
export OTEL_EXPORTER_OTLP_HEADERS="Authorization=Basic <base64-token>"
```

### DataDog APM
```bash
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
export DD_API_KEY=<your-api-key>
```

## Troubleshooting

### No traces appearing
1. Check Jaeger is running: `docker ps | grep jaeger`
2. Verify tracing enabled: `curl http://localhost:8000/ | jq .tracing`
3. Check OTLP endpoint: `echo $OTEL_EXPORTER_OTLP_ENDPOINT`

### Port conflicts
- PostgreSQL: Change `5432` to another port
- Jaeger UI: Change `16686` to another port
- API: Change `8000` to another port in code

### Module not found
```bash
cd /path/to/cclab-posgres
maturin develop --release
```

## Best Practices

### 1. Use Sampling in Production
```python
from opentelemetry.sdk.trace.sampling import TraceIdRatioBased
sampler = TraceIdRatioBased(0.1)  # 10% sampling
```

### 2. Optimize Queries
```python
# Use eager loading to avoid N+1
query = session.find(Post).options(selectinload(Post.author))
```

### 3. Monitor Connection Pool
```python
from cclab.titan.telemetry import get_connection_pool_metrics
metrics = get_connection_pool_metrics()
```

### 4. Add Custom Spans
```python
from cclab.titan.telemetry import instrument_span

@instrument_span("business.process_order")
async def process_order(order_id: int):
    # Your business logic
    pass
```

## Security Considerations

1. **Never log sensitive data**: Passwords, tokens, PII
2. **Use TLS in production**: Set `OTEL_EXPORTER_OTLP_INSECURE=false`
3. **Authenticate OTLP**: Use `OTEL_EXPORTER_OTLP_HEADERS`
4. **Environment variables**: Keep credentials out of code

## Next Steps

1. **Explore the Code**: Read `fastapi_otel_example.py`
2. **Run the Example**: Follow `QUICKSTART_FASTAPI_OTEL.md`
3. **View Traces**: Open http://localhost:16686
4. **Read Patterns**: Study `TRACE_EXAMPLES.md`
5. **Try Different Backends**: Use `otel_backends.env.example`
6. **Integrate Your App**: Adapt the patterns to your project

## Resources

- [OpenTelemetry Python](https://opentelemetry.io/docs/instrumentation/python/)
- [FastAPI Documentation](https://fastapi.tiangolo.com/)
- [Jaeger Documentation](https://www.jaegertracing.io/docs/)
- [cclab PostgreSQL](../README.md)
- [OTLP Specification](https://opentelemetry.io/docs/specs/otlp/)

## Support

For issues or questions:
1. Check troubleshooting sections in docs
2. Review trace examples in Jaeger
3. Enable debug logging: `OTEL_LOG_LEVEL=debug`
4. Check cclab documentation

## License

MIT License - See [LICENSE](../LICENSE) for details.

---

**Created**: 2026-01-06
**cclab Version**: 0.1.0
**Python**: 3.12+
**OpenTelemetry**: Latest stable

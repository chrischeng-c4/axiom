# Trace Examples and Patterns

This document shows example trace structures you'll see in Jaeger when using the FastAPI + OpenTelemetry example.

## 1. Simple Query (GET /users)

```
Trace: GET /users
Duration: 15ms
Spans: 3

┌─────────────────────────────────────────────────────────┐
│ GET /users                                              │ 15ms
│ http.method: GET                                        │
│ http.route: /users                                      │
│ http.status_code: 200                                   │
├─────────────────────────────────────────────────────────┤
│   ├─ db.query.find                                      │ 8ms
│   │  db.system: postgresql                              │
│   │  db.collection.name: users                          │
│   │  db.operation.name: find                            │
│   │  db.query.limit: 100                                │
│   │  db.query.offset: 0                                 │
│   │  db.result.count: 5                                 │
│   │                                                      │
│   └─ db.relationship.load (for each user's posts)      │ 5ms
│      db.relationship.name: posts                        │
│      db.relationship.strategy: lazy                     │
│      db.result.count: 3                                 │
└─────────────────────────────────────────────────────────┘
```

**Key Observations**:
- HTTP span is the parent
- Query span shows table and operation
- Relationship spans show lazy loading
- N+1 pattern visible (5 users = 5 relationship queries)

---

## 2. Eager Loading (GET /posts?eager=true)

```
Trace: GET /posts?eager=true
Duration: 12ms
Spans: 3

┌─────────────────────────────────────────────────────────┐
│ GET /posts                                              │ 12ms
│ http.method: GET                                        │
│ http.route: /posts                                      │
│ http.status_code: 200                                   │
├─────────────────────────────────────────────────────────┤
│   ├─ db.query.find                                      │ 6ms
│   │  db.system: postgresql                              │
│   │  db.collection.name: posts                          │
│   │  db.operation.name: find                            │
│   │  db.query.limit: 100                                │
│   │  db.result.count: 10                                │
│   │                                                      │
│   └─ db.relationship.load                               │ 4ms
│      db.relationship.name: author                       │
│      db.relationship.strategy: selectinload             │
│      db.result.count: 10                                │
│      db.relationship.batch_count: 1                     │
└─────────────────────────────────────────────────────────┘
```

**Key Observations**:
- Single relationship span instead of N spans
- strategy: selectinload (eager loading)
- batch_count: 1 (loaded all in one query)
- Faster than lazy loading (12ms vs 15ms+ for similar data)

---

## 3. Create User (POST /users)

```
Trace: POST /users
Duration: 20ms
Spans: 4

┌─────────────────────────────────────────────────────────┐
│ POST /users                                             │ 20ms
│ http.method: POST                                       │
│ http.route: /users                                      │
│ http.status_code: 201                                   │
├─────────────────────────────────────────────────────────┤
│   ├─ db.session.flush                                   │ 10ms
│   │  db.session.operation: flush                        │
│   │  db.session.pending_count: 1                        │
│   │  db.session.dirty_count: 0                          │
│   │  db.session.deleted_count: 0                        │
│   │  │                                                   │
│   │  └─ db.query.insert                                 │ 6ms
│   │     db.system: postgresql                           │
│   │     db.collection.name: users                       │
│   │     db.operation.name: insert                       │
│   │     db.result.affected_rows: 1                      │
│   │                                                      │
│   └─ db.session.commit                                  │ 4ms
│      db.session.operation: commit                       │
└─────────────────────────────────────────────────────────┘
```

**Key Observations**:
- Session spans wrap database operations
- pending_count shows how many objects will be inserted
- Insert span nested under flush span
- Commit span completes transaction

---

## 4. Get User with Posts (GET /users/1)

```
Trace: GET /users/1
Duration: 18ms
Spans: 4

┌─────────────────────────────────────────────────────────┐
│ GET /users/{user_id}                                    │ 18ms
│ http.method: GET                                        │
│ http.route: /users/{user_id}                            │
│ http.status_code: 200                                   │
├─────────────────────────────────────────────────────────┤
│   ├─ db.query.find                                      │ 7ms
│   │  db.system: postgresql                              │
│   │  db.collection.name: users                          │
│   │  db.operation.name: find                            │
│   │  db.query.filters_count: 1                          │
│   │  db.result.count: 1                                 │
│   │                                                      │
│   └─ db.relationship.load                               │ 8ms
│      db.relationship.name: posts                        │
│      db.relationship.strategy: lazy                     │
│      db.relationship.fk_column: author_id               │
│      db.relationship.target_model: Post                 │
│      db.result.count: 5                                 │
└─────────────────────────────────────────────────────────┘
```

**Key Observations**:
- filters_count: 1 (filtering by user_id)
- Lazy loading triggered when accessing user.posts
- fk_column shows relationship structure
- Result count shows 5 posts for this user

---

## 5. Error Handling (GET /users/99999)

```
Trace: GET /users/99999 (404)
Duration: 10ms
Spans: 2

┌─────────────────────────────────────────────────────────┐
│ GET /users/{user_id}                                    │ 10ms
│ http.method: GET                                        │
│ http.route: /users/{user_id}                            │
│ http.status_code: 404                                   │
│ error: true                                             │
├─────────────────────────────────────────────────────────┤
│   └─ db.query.find                                      │ 5ms
│      db.system: postgresql                              │
│      db.collection.name: users                          │
│      db.operation.name: find                            │
│      db.query.filters_count: 1                          │
│      db.result.count: 0                                 │
│                                                          │
│   Exception: HTTPException                              │
│   exception.type: fastapi.exceptions.HTTPException      │
│   exception.message: User not found                     │
│   exception.stacktrace: ...                             │
└─────────────────────────────────────────────────────────┘
```

**Key Observations**:
- error: true flag on parent span
- Exception details recorded
- result.count: 0 (not found)
- HTTP status: 404 properly set

---

## 6. Complex Operation (Create Post)

```
Trace: POST /posts
Duration: 35ms
Spans: 7

┌─────────────────────────────────────────────────────────┐
│ POST /posts                                             │ 35ms
│ http.method: POST                                       │
│ http.route: /posts                                      │
│ http.status_code: 201                                   │
├─────────────────────────────────────────────────────────┤
│   ├─ db.query.find (verify author exists)               │ 6ms
│   │  db.system: postgresql                              │
│   │  db.collection.name: users                          │
│   │  db.operation.name: find                            │
│   │  db.query.filters_count: 1                          │
│   │  db.result.count: 1                                 │
│   │                                                      │
│   ├─ db.session.flush                                   │ 12ms
│   │  db.session.operation: flush                        │
│   │  db.session.pending_count: 1                        │
│   │  │                                                   │
│   │  └─ db.query.insert                                 │ 8ms
│   │     db.system: postgresql                           │
│   │     db.collection.name: posts                       │
│   │     db.operation.name: insert                       │
│   │     db.result.affected_rows: 1                      │
│   │                                                      │
│   ├─ db.session.commit                                  │ 5ms
│   │  db.session.operation: commit                       │
│   │                                                      │
│   └─ db.relationship.load (load author for response)    │ 8ms
│      db.relationship.name: author                       │
│      db.relationship.strategy: lazy                     │
│      db.result.count: 1                                 │
└─────────────────────────────────────────────────────────┘
```

**Key Observations**:
- Multiple operations in one request
- Verify author → Insert post → Load author
- Session lifecycle visible (flush → commit)
- Clear parent-child relationships

---

## 7. Pagination (GET /users?limit=10&offset=20)

```
Trace: GET /users (paginated)
Duration: 14ms
Spans: 3

┌─────────────────────────────────────────────────────────┐
│ GET /users                                              │ 14ms
│ http.method: GET                                        │
│ http.route: /users                                      │
│ http.status_code: 200                                   │
├─────────────────────────────────────────────────────────┤
│   └─ db.query.find                                      │ 7ms
│      db.system: postgresql                              │
│      db.collection.name: users                          │
│      db.operation.name: find                            │
│      db.query.limit: 10                                 │
│      db.query.offset: 20                                │
│      db.result.count: 10                                │
└─────────────────────────────────────────────────────────┘
```

**Key Observations**:
- limit and offset visible in span attributes
- Result count matches limit (full page)
- Can track pagination patterns across traces

---

## Analyzing Performance

### N+1 Query Detection

Compare these two traces:

**Lazy Loading** (N+1):
```
GET /posts (10 posts, lazy loading)
├─ db.query.find: 6ms          [1 query]
└─ db.relationship.load: 8ms   [1 query per post = 10 queries]
Total queries: 11
Duration: 50ms+
```

**Eager Loading**:
```
GET /posts (10 posts, eager loading)
├─ db.query.find: 6ms          [1 query]
└─ db.relationship.load: 4ms   [1 batch query]
Total queries: 2
Duration: 12ms
```

**Improvement**: 4x faster, 5.5x fewer queries

### Session Overhead

Track session performance:
```
Session operations:
├─ db.session.flush: 10ms
│  └─ db.query.insert: 6ms    [actual DB time]
│  Overhead: 4ms               [ORM processing]
│
└─ db.session.commit: 4ms
Total session overhead: 8ms
```

### Connection Pool Monitoring

Look for these patterns in metrics:
- High `in_use` count → need more connections
- Low `idle` count → pool exhaustion
- Long query times → optimize queries or scale DB

---

## Trace Context Propagation

When calling external services:

```
HTTP Request → API Service → Database
     │              │            │
     │              │            └─ db.query.find
     │              │
     │              └─ HTTP Client Request → External API
     │                      │
     │                      └─ External Service Span
     │
     └─ All share same trace_id
```

Example trace_id: `a1b2c3d4e5f6g7h8i9j0`

All spans in this distributed trace share the same trace_id, enabling:
- End-to-end request tracking
- Cross-service latency analysis
- Dependency mapping

---

## Tips for Jaeger UI

### Finding Slow Queries

1. Go to "Search" tab
2. Filter by service: `fastapi-databridge-api`
3. Set "Min Duration": 100ms
4. Look for spans with `db.query.*` operation

### Comparing Lazy vs Eager Loading

1. Find traces for same endpoint
2. One with `eager=false`, one with `eager=true`
3. Compare:
   - Total duration
   - Number of spans
   - `db.relationship.strategy` attribute

### Tracking Errors

1. Go to "Search" tab
2. Select "Tags": `error=true`
3. Look for exception details in span logs

### Analyzing Patterns

1. Go to "System Architecture" tab
2. See service dependencies
3. Identify bottlenecks

---

## Custom Attributes

Add your own attributes for better insights:

```python
from cclab.titan.telemetry import instrument_span

@instrument_span("business.validate_order", attributes={
    "order_type": "standard",
    "payment_method": "credit_card",
})
async def validate_order(order_id: int):
    # Business logic
    pass
```

These will appear in Jaeger as searchable tags!

---

## References

- [OpenTelemetry Semantic Conventions](https://opentelemetry.io/docs/specs/semconv/)
- [Jaeger Documentation](https://www.jaegertracing.io/docs/)
- [cclab Telemetry Guide](../docs/telemetry.md)

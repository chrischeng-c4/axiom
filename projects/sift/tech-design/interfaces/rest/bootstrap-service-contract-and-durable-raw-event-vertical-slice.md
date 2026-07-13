---
id: "1576"
summary: (fill)
fill_sections: [logic]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
flowchart LR
    C[Client or OTLP producer] --> H[Single Sift h2c HTTP service]
    H --> V[Validate versioned operational-event envelope]
    V --> I[Deduplicate by event_id]
    I --> J[Append raw event journal]
    J --> F[fsync durable bytes]
    F --> A[Return accepted event id and durable cursor]
    J --> Q[Raw event query and replay]
    H --> O[Health readiness metrics OpenAPI and docs]
```

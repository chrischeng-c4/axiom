---
id: cclab-photon-v2
type: spec
title: "cclab-photon v2 Specification"
version: 1
spec_type: integration
created_at: 2026-01-28T08:27:41.900619+00:00
updated_at: 2026-01-28T08:27:41.900619+00:00
requirements:
  total: 7
  ids:
    - R1
    - R2
    - R3
    - R4
    - R5
    - R6
    - R7
design_elements:
  has_mermaid: true
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
history:
  - timestamp: 2026-01-28T08:27:41.900619+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# cclab-photon v2 Specification

## Overview

Upgrade cclab-photon to a production-grade HTTP client with full sync/async parity, middleware, and in-memory transport support. This specification defines the core abstractions and the technical design for a highly extensible and performant client.

## Requirements

### R1 - Synchronous Client Parity

```yaml
id: R1
priority: medium
status: draft
```

Implement SyncHttpClient using reqwest::blocking for parity with the async HttpClient.

### R2 - Middleware Architecture

```yaml
id: R2
priority: medium
status: draft
```

Define a Middleware trait and a chain for pre/post request processing.

### R3 - Automatic Retries

```yaml
id: R3
priority: medium
status: draft
```

Integrate RetryConfig into a Middleware for automatic request retries.

### R4 - Multipart Support

```yaml
id: R4
priority: medium
status: draft
```

Add support for multipart/form-data in RequestBuilder and ExtractedRequest.

### R5 - Streaming Support

```yaml
id: R5
priority: medium
status: draft
```

Enable streaming of large request and response bodies.

### R6 - Transport Abstraction

```yaml
id: R6
priority: medium
status: draft
```

Abstract the HTTP execution into a Transport trait to support ASGI/WSGI in-memory calls.

### R7 - Advanced Configuration

```yaml
id: R7
priority: medium
status: draft
```

Support proxies, HTTP/2, and persistent cookie jars in HttpClientConfig.

## Acceptance Criteria

### Scenario: Sync GET Request

- **GIVEN** A SyncHttpClient instance.
- **WHEN** The user calls client.get().
- **THEN** The request executes synchronously and returns an HttpResponse.

### Scenario: Proxy Support

- **GIVEN** A client with a configured proxy.
- **WHEN** A request is executed.
- **THEN** The request is routed through the specified proxy.

### Scenario: In-memory ASGI Call

- **GIVEN** An ASGI application (e.g. FastAPI).
- **WHEN** AsgiTransport is used.
- **THEN** The request is handled by the ASGI app directly without network IO.

### Scenario: Streaming Upload

- **GIVEN** A large file on disk.
- **WHEN** The user sends a request with a stream body.
- **THEN** The request body is streamed to the server without loading the entire file into memory.

### Scenario: Multipart Form Submission

- **GIVEN** A RequestBuilder.
- **WHEN** The user adds multiple parts (text and file) to the form.
- **THEN** The request is sent with 'multipart/form-data' content type and correct boundaries.

### Scenario: HTTP/2 Negotiation

- **GIVEN** A client configured for HTTP/2.
- **WHEN** A request is sent to an HTTP/2 compatible server.
- **THEN** The client successfully negotiates and uses the HTTP/2 protocol with the server.

### Scenario: Middleware Chain Order

- **GIVEN** A Middleware chain with 3 middlewares.
- **WHEN** A request is executed.
- **THEN** The middlewares are executed in the order they were added to the client.

## Flow Diagram

```mermaid
sequenceDiagram
    participant Client as Client
    participant M1 as AuthMiddleware
    participant M2 as RetryMiddleware
    participant Transport as Transport
    participant Server as Server
    Client->M1: handle(request)
    M1->M2: handle(request)
    M2->Transport: execute(request)
    Transport->Server: HTTP Request
    Server-->Transport: HTTP Response
    Transport-->M2: Response
    M2-->M1: Response
    M1-->Client: Response

classDiagram
    class Transport {
        <<interface>>
        +execute(request Request) Result<Response, Error>
    }
    class Middleware {
        <<interface>>
        +handle(request Request, next NextChain) Result<Response, Error>
    }
    class HttpTransport {
    }
    class AsgiTransport {
    }
    class HttpClient {
        +send(request Request) Result<Response, Error>
        +get(url String) RequestBuilder
    }
    Transport ..|> HttpTransport
    Transport ..|> AsgiTransport
    HttpClient --> Transport : uses
    HttpClient --> Middleware : has many
```

</spec>

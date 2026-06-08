---
id: photon-sequence
type: spec
title: "Photon HTTP Client Sequence Diagrams"
version: 1
spec_type: integration
created_at: 2026-01-31T12:45:00+00:00
updated_at: 2026-01-31T12:45:00+00:00
design_elements:
  has_mermaid: true
  diagrams:
    - type: sequence
      title: "HTTP Request Flow"
    - type: sequence
      title: "Connection Pooling"
---

<spec>

# Photon HTTP Client Sequences

## Overview

Photon provides high-performance async HTTP client for Python with Rust backend.

## HTTP Request Flow

```mermaid
sequenceDiagram
    participant App as Python App
    participant Client as HttpClient
    participant Pool as Connection Pool
    participant Conn as Connection
    participant Server as Remote Server

    App->>Client: await client.get(url)
    Client->>Pool: acquire_connection(host)

    alt Connection available
        Pool-->>Client: existing connection
    else No connection
        Pool->>Conn: create_new()
        Conn->>Server: TCP handshake
        Server-->>Conn: connected
        Conn->>Server: TLS handshake (if https)
        Server-->>Conn: TLS established
        Pool-->>Client: new connection
    end

    Client->>Conn: send_request(method, path, headers, body)
    Conn->>Server: HTTP request
    Server-->>Conn: HTTP response
    Conn-->>Client: Response object
    Client->>Pool: release_connection()
    Client-->>App: Response
```

## Retry Flow

```mermaid
sequenceDiagram
    participant App as Python App
    participant Client as HttpClient
    participant Retry as Retry Handler
    participant Server as Remote Server

    App->>Client: await client.get(url, retry=3)

    loop max retries
        Client->>Server: HTTP request
        alt Success (2xx)
            Server-->>Client: Response
            Client-->>App: Response
        else Retryable error (5xx, timeout)
            Server-->>Client: Error
            Client->>Retry: should_retry?
            Retry->>Retry: backoff_delay()
            Note over Retry: exponential backoff
        else Non-retryable error (4xx)
            Server-->>Client: Error
            Client-->>App: raise HTTPError
        end
    end

    Client-->>App: raise MaxRetriesExceeded
```

## Streaming Response

```mermaid
sequenceDiagram
    participant App as Python App
    participant Client as HttpClient
    participant Conn as Connection
    participant Server as Remote Server

    App->>Client: async with client.stream(url) as resp:
    Client->>Conn: send_request()
    Conn->>Server: HTTP request
    Server-->>Conn: Headers + Transfer-Encoding: chunked

    loop async for chunk in resp.aiter_bytes()
        Conn->>Server: read chunk
        Server-->>Conn: chunk data
        Conn-->>Client: bytes
        Client-->>App: yield bytes
    end

    Server-->>Conn: final chunk (0)
    Client-->>App: stream complete
```

</spec>

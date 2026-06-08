---
title: HTTP Client Data Flows
status: implemented
component: data-bridge-http
type: data-flow
---

# HTTP Client Data Flows

> Part of [HTTP Client Documentation](./README.md)

## 1. Request Execution Flow

```mermaid
sequenceDiagram
    participant Py as Python
    participant Builder as RequestBuilder
    participant Client as HttpClient
    participant Net as Network

    Py->>Builder: client.get("https://api.com")
    Py->>Builder: .header("Auth", "...")
    Py->>Builder: .send()
    
    Note right of Py: GIL HELD
    
    Builder->>Builder: extract_request()
    Note right of Builder: Creates pure Rust<br/>ExtractedRequest
    
    Builder->>Client: execute(extracted)
    
    Note right of Py: GIL RELEASED
    
    rect rgb(200, 255, 200)
        Note right of Client: Async Task
        Client->>Client: Build reqwest::Request
        Client->>Net: Send Request
        Net-->>Client: Receive Response
        Client->>Client: Measure Latency
        Client->>Client: Read Body (await)
    end
    
    Client-->>Py: Result<HttpResponse>
    
    Note right of Py: GIL ACQUIRED
```

## 2. Error Sanitization Flow

Goal: Ensure a failed request with sensitive data doesn't leak into logs.

```mermaid
sequenceDiagram
    participant Net as Network
    participant Client as HttpClient
    participant Sanitizer as Error Sanitizer
    participant Py as Python

    Client->>Net: GET https://api.com?key=SECRET
    Net-->>Client: Connection Refused
    
    Client->>Client: Create HttpError::Connect
    Client->>Sanitizer: format_error()
    
    Sanitizer->>Sanitizer: Regex Replace
    Note right of Sanitizer: key=SECRET -> key=[REDACTED]
    
    Sanitizer-->>Client: Safe Message
    
    Client-->>Py: Raise Exception(Safe Message)
```

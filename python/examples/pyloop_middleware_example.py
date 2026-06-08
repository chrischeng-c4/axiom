"""
Middleware example for PyLoop HTTP server.

Demonstrates:
- CORS middleware
- Logging middleware
- Custom middleware
- Middleware ordering
"""

import logging
from cclab.orbit import (
    App,
    CORSMiddleware,
    LoggingMiddleware,
    BaseMiddleware,
    HTTPException
)

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)

# Custom middleware example
class AuthMiddleware(BaseMiddleware):
    """Simple API key authentication middleware."""

    def __init__(self, api_key: str):
        self.api_key = api_key

    async def process_request(self, request):
        # Skip auth for health check
        if request.get("path") == "/health":
            return None

        # Check API key
        auth_header = request.get("headers", {}).get("authorization", "")
        if not auth_header.startswith("Bearer "):
            return {
                "status": 401,
                "body": {"error": "Missing or invalid authorization header"}
            }

        token = auth_header[7:]  # Remove "Bearer "
        if token != self.api_key:
            return {
                "status": 401,
                "body": {"error": "Invalid API key"}
            }

        # Auth successful, continue to handler
        return None

    async def process_response(self, request, response):
        # No response processing needed
        return response


class RateLimitMiddleware(BaseMiddleware):
    """Simple in-memory rate limiting middleware."""

    def __init__(self, max_requests: int = 100):
        self.max_requests = max_requests
        self.request_counts = {}

    async def process_request(self, request):
        ip = request.get("headers", {}).get("x-forwarded-for", "unknown")

        # Increment counter
        self.request_counts[ip] = self.request_counts.get(ip, 0) + 1

        if self.request_counts[ip] > self.max_requests:
            return {
                "status": 429,
                "body": {"error": "Too many requests"},
                "headers": {"Retry-After": "60"}
            }

        return None

    async def process_response(self, request, response):
        # Add rate limit headers
        ip = request.get("headers", {}).get("x-forwarded-for", "unknown")
        count = self.request_counts.get(ip, 0)

        if "headers" not in response:
            response["headers"] = {}

        response["headers"]["X-RateLimit-Limit"] = str(self.max_requests)
        response["headers"]["X-RateLimit-Remaining"] = str(max(0, self.max_requests - count))

        return response


# Create app
app = App(title="Middleware Demo", version="1.0.0", debug=True)

# Add middleware (order matters!)
# 1. CORS - handle preflight first
app.add_middleware(CORSMiddleware(
    allow_origins=["http://localhost:3000", "https://example.com"],
    allow_methods=["GET", "POST", "PUT", "DELETE"],
    allow_headers=["Content-Type", "Authorization"],
    allow_credentials=True,
    max_age=3600
))

# 2. Logging - log all requests
app.add_middleware(LoggingMiddleware(
    log_request_body=True,
    log_response_body=False
))

# 3. Rate limiting - before auth (cheaper check)
app.add_middleware(RateLimitMiddleware(max_requests=100))

# 4. Auth - protect routes
app.add_middleware(AuthMiddleware(api_key="secret-api-key-123"))

# Routes
@app.get("/")
async def root(request):
    """Public endpoint (requires auth)."""
    return {
        "message": "Hello from PyLoop with middleware!",
        "middlewares": ["CORS", "Logging", "RateLimit", "Auth"]
    }

@app.get("/health")
async def health(request):
    """Health check (no auth required)."""
    return {"status": "healthy"}

@app.get("/user")
async def get_user(request):
    """Protected endpoint."""
    return {
        "id": 1,
        "name": "Alice",
        "email": "alice@example.com"
    }

@app.post("/data")
async def create_data(request):
    """POST endpoint to test CORS."""
    body = request.get("body", {})
    return {
        "status": "created",
        "data": body
    }

if __name__ == "__main__":
    print("=" * 70)
    print("Phase 5: Middleware Demo")
    print("=" * 70)
    print("\nMiddleware stack (in order):")
    print("  1. CORSMiddleware - Handle CORS and preflight")
    print("  2. LoggingMiddleware - Log all requests/responses")
    print("  3. RateLimitMiddleware - Rate limiting")
    print("  4. AuthMiddleware - API key authentication")
    print("\nTest these commands:")
    print("\n  # Health check (no auth)")
    print("  curl http://127.0.0.1:8000/health")
    print("\n  # Protected endpoint (requires auth)")
    print("  curl http://127.0.0.1:8000/ \\")
    print("    -H 'Authorization: Bearer secret-api-key-123'")
    print("\n  # CORS preflight")
    print("  curl -X OPTIONS http://127.0.0.1:8000/data \\")
    print("    -H 'Origin: http://localhost:3000' \\")
    print("    -H 'Access-Control-Request-Method: POST'")
    print("\n  # CORS POST request")
    print("  curl -X POST http://127.0.0.1:8000/data \\")
    print("    -H 'Origin: http://localhost:3000' \\")
    print("    -H 'Content-Type: application/json' \\")
    print("    -H 'Authorization: Bearer secret-api-key-123' \\")
    print("    -d '{\"name\": \"test\"}'")
    print("\n" + "=" * 70)

    app.serve(host="127.0.0.1", port=8000)

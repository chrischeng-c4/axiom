"""
FastAPI + data-bridge PostgreSQL ORM + OpenTelemetry Integration Example

This example demonstrates a production-ready FastAPI application with:
- Complete OpenTelemetry instrumentation (OTLP exporter)
- Cloud-native observability (Jaeger, Grafana Cloud, DataDog)
- data-bridge ORM with automatic tracing
- Distributed tracing across HTTP → ORM → Database
- Connection pool metrics
- Multiple API endpoint patterns

Architecture:
    HTTP Request → FastAPI (auto-instrumented)
        ↓
    ORM Operations (data-bridge spans)
        ↓
    PostgreSQL (query spans)
        ↓
    OTLP Exporter → Jaeger/Grafana/DataDog

Setup:
    # Install dependencies
    pip install fastapi uvicorn opentelemetry-api opentelemetry-sdk \
                opentelemetry-instrumentation-fastapi \
                opentelemetry-exporter-otlp-proto-grpc \
                data-bridge

    # Start infrastructure (Jaeger + PostgreSQL)
    docker-compose up -d

    # Run application
    python examples/fastapi_otel_example.py

    # View traces
    open http://localhost:16686

Usage:
    # Create users
    curl -X POST http://localhost:8000/users \
         -H "Content-Type: application/json" \
         -d '{"name": "Alice", "email": "alice@example.com"}'

    # List users
    curl http://localhost:8000/users

    # Get user with posts (lazy loading)
    curl http://localhost:8000/users/1

    # Get post with author (demonstrates relationship)
    curl http://localhost:8000/posts/1

    # List posts with eager loading
    curl "http://localhost:8000/posts?eager=true"

OpenTelemetry Backends:
    1. Jaeger (local development):
       OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317

    2. Grafana Cloud:
       OTEL_EXPORTER_OTLP_ENDPOINT=https://otlp-gateway-prod-us-central-0.grafana.net/otlp
       OTEL_EXPORTER_OTLP_HEADERS="Authorization=Basic <base64-encoded-token>"

    3. DataDog:
       OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
       DD_API_KEY=<your-api-key>

    4. Generic OTLP:
       OTEL_EXPORTER_OTLP_ENDPOINT=<your-collector-endpoint>
       OTEL_EXPORTER_OTLP_INSECURE=true  # For HTTP
"""

import asyncio
import os
from contextlib import asynccontextmanager
from typing import List, Optional

from fastapi import FastAPI, HTTPException, Query
from pydantic import BaseModel, EmailStr

# OpenTelemetry SDK imports
from opentelemetry import trace, metrics
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import BatchSpanProcessor
from opentelemetry.sdk.resources import Resource, SERVICE_NAME, SERVICE_VERSION, DEPLOYMENT_ENVIRONMENT
from opentelemetry.exporter.otlp.proto.grpc.trace_exporter import OTLPSpanExporter
from opentelemetry.instrumentation.fastapi import FastAPIInstrumentor

# data-bridge imports
from cclab.titan import (
    Table, Column, relationship,
    init, close, Session,
    is_tracing_enabled,
    selectinload
)


# ============================================================================
# Configuration
# ============================================================================

# Application settings
APP_NAME = os.environ.get("SERVICE_NAME", "fastapi-databridge-api")
APP_VERSION = os.environ.get("SERVICE_VERSION", "1.0.0")
ENVIRONMENT = os.environ.get("DEPLOYMENT_ENVIRONMENT", "development")

# Database settings
DATABASE_URL = os.environ.get(
    "DATABASE_URL",
    "postgresql://postgres:postgres@localhost:5432/fastapi_demo"
)

# OpenTelemetry settings
OTLP_ENDPOINT = os.environ.get("OTEL_EXPORTER_OTLP_ENDPOINT", "http://localhost:4317")
OTLP_INSECURE = os.environ.get("OTEL_EXPORTER_OTLP_INSECURE", "true").lower() == "true"
OTLP_HEADERS = os.environ.get("OTEL_EXPORTER_OTLP_HEADERS", None)

# Parse headers if provided (format: "key1=value1,key2=value2")
otlp_headers = None
if OTLP_HEADERS:
    otlp_headers = dict(pair.split("=") for pair in OTLP_HEADERS.split(","))


# ============================================================================
# OpenTelemetry Setup
# ============================================================================

def setup_opentelemetry():
    """
    Configure OpenTelemetry SDK with OTLP exporter.

    This function sets up:
    1. TracerProvider with service metadata
    2. OTLP exporter (gRPC) for traces
    3. BatchSpanProcessor for performance
    """
    # Create resource with service metadata
    resource = Resource.create({
        SERVICE_NAME: APP_NAME,
        SERVICE_VERSION: APP_VERSION,
        DEPLOYMENT_ENVIRONMENT: ENVIRONMENT,
        "service.instance.id": os.environ.get("HOSTNAME", "localhost"),
    })

    # Create tracer provider
    provider = TracerProvider(resource=resource)

    # Configure OTLP exporter
    otlp_exporter = OTLPSpanExporter(
        endpoint=OTLP_ENDPOINT,
        insecure=OTLP_INSECURE,
        headers=otlp_headers,
    )

    # Add batch span processor for better performance
    # Batches spans before exporting to reduce network overhead
    processor = BatchSpanProcessor(otlp_exporter)
    provider.add_span_processor(processor)

    # Set as global tracer provider
    trace.set_tracer_provider(provider)

    print(f"OpenTelemetry configured:")
    print(f"  - Service: {APP_NAME} v{APP_VERSION}")
    print(f"  - Environment: {ENVIRONMENT}")
    print(f"  - OTLP Endpoint: {OTLP_ENDPOINT}")
    print(f"  - Insecure: {OTLP_INSECURE}")
    print(f"  - data-bridge tracing: {is_tracing_enabled()}")


# Initialize OpenTelemetry
setup_opentelemetry()


# ============================================================================
# ORM Models
# ============================================================================

class User(Table):
    """User model with posts relationship."""

    id: int = Column(primary_key=True)
    name: str = Column(nullable=False)
    email: str = Column(nullable=False, unique=True)
    age: Optional[int] = Column(nullable=True)

    # Relationship: one user has many posts
    # This will be lazy-loaded by default, creating separate spans
    posts: List["Post"] = relationship(
        "Post",
        foreign_key_column="author_id",
        back_populates="author"
    )

    class Settings:
        table_name = "users"


class Post(Table):
    """Post model with author relationship."""

    id: int = Column(primary_key=True)
    title: str = Column(nullable=False)
    content: str = Column(nullable=False)
    author_id: int = Column(foreign_key="users.id", nullable=False)

    # Relationship: one post belongs to one user
    # Lazy loading will create a span when accessed
    author: User = relationship(
        User,
        foreign_key_column="author_id",
        back_populates="posts"
    )

    class Settings:
        table_name = "posts"


# ============================================================================
# Pydantic Models (API Request/Response)
# ============================================================================

class UserCreate(BaseModel):
    """User creation request."""
    name: str
    email: EmailStr
    age: Optional[int] = None


class UserResponse(BaseModel):
    """User response model."""
    id: int
    name: str
    email: str
    age: Optional[int] = None
    posts_count: Optional[int] = None  # Only for list endpoint


class PostCreate(BaseModel):
    """Post creation request."""
    title: str
    content: str
    author_id: int


class PostResponse(BaseModel):
    """Post response model."""
    id: int
    title: str
    content: str
    author_id: int
    author_name: Optional[str] = None  # Only when author is loaded


# ============================================================================
# FastAPI Application
# ============================================================================

@asynccontextmanager
async def lifespan(app: FastAPI):
    """
    Application lifespan manager.

    Handles database connection initialization and cleanup.
    """
    # Parse DATABASE_URL
    # Format: postgresql://user:password@host:port/database
    parts = DATABASE_URL.replace("postgresql://", "").split("@")
    user_pass = parts[0].split(":")
    host_port_db = parts[1].split("/")
    host_port = host_port_db[0].split(":")

    username = user_pass[0]
    password = user_pass[1] if len(user_pass) > 1 else ""
    host = host_port[0]
    port = int(host_port[1]) if len(host_port) > 1 else 5432
    database = host_port_db[1]

    # Initialize database connection
    await init(
        host=host,
        port=port,
        database=database,
        username=username,
        password=password,
        max_connections=20,  # Connection pool size
    )

    print(f"Connected to PostgreSQL: {host}:{port}/{database}")

    # Create tables if they don't exist
    await _create_tables_if_needed()

    yield

    # Cleanup
    await close()
    print("Database connection closed")


# Create FastAPI app
app = FastAPI(
    title="FastAPI + data-bridge + OpenTelemetry",
    description="Production-ready API with distributed tracing",
    version="1.0.0",
    lifespan=lifespan,
)

# Instrument FastAPI with OpenTelemetry
# This automatically creates spans for all HTTP requests
FastAPIInstrumentor.instrument_app(app)


# ============================================================================
# Helper Functions
# ============================================================================

async def _create_tables_if_needed():
    """Create tables if they don't exist (for demo purposes)."""
    from cclab.titan import execute

    # Create users table
    await execute("""
        CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            email VARCHAR(255) NOT NULL UNIQUE,
            age INTEGER
        )
    """)

    # Create posts table
    await execute("""
        CREATE TABLE IF NOT EXISTS posts (
            id SERIAL PRIMARY KEY,
            title VARCHAR(255) NOT NULL,
            content TEXT NOT NULL,
            author_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE
        )
    """)

    print("Database tables ready")


# ============================================================================
# API Endpoints
# ============================================================================

@app.get("/")
async def root():
    """Health check endpoint."""
    return {
        "service": APP_NAME,
        "version": APP_VERSION,
        "environment": ENVIRONMENT,
        "tracing": is_tracing_enabled(),
    }


@app.get("/users", response_model=List[UserResponse])
async def list_users(
    limit: int = Query(default=100, le=1000),
    offset: int = Query(default=0, ge=0),
):
    """
    List users with pagination.

    Demonstrates:
    - Query span with filters_count, limit, offset attributes
    - Result count in span
    - Parent-child span hierarchy (HTTP → Query)
    """
    async with Session() as session:
        # This creates a query span with pagination info
        users = await session.find(User).limit(limit).offset(offset).all()

        # Count posts for each user (demonstrates N+1 query pattern)
        # Each posts access will create a separate relationship span
        results = []
        for user in users:
            # Accessing user.posts triggers lazy loading → creates relationship span
            posts_count = len(await user.posts) if user.posts else 0

            results.append(UserResponse(
                id=user.id,
                name=user.name,
                email=user.email,
                age=user.age,
                posts_count=posts_count,
            ))

        return results


@app.get("/users/{user_id}", response_model=UserResponse)
async def get_user(user_id: int):
    """
    Get single user by ID.

    Demonstrates:
    - Query span with primary key lookup
    - Session identity map (cached access)
    - 404 handling with span status
    """
    async with Session() as session:
        # Query creates a span with operation="find", filters_count=1
        user = await session.find(User).filter(User.id == user_id).first()

        if user is None:
            # Exception is automatically recorded in span
            raise HTTPException(status_code=404, detail="User not found")

        # Accessing posts triggers relationship loading span
        posts_count = len(await user.posts) if user.posts else 0

        return UserResponse(
            id=user.id,
            name=user.name,
            email=user.email,
            age=user.age,
            posts_count=posts_count,
        )


@app.post("/users", response_model=UserResponse, status_code=201)
async def create_user(user_data: UserCreate):
    """
    Create a new user.

    Demonstrates:
    - Insert span with session tracking
    - Pending/dirty count in session span
    - Transaction handling
    """
    async with Session() as session:
        # Create user instance
        user = User(
            name=user_data.name,
            email=user_data.email,
            age=user_data.age,
        )

        # Add to session (marks as pending)
        session.add(user)

        # Flush creates a session span with pending_count=1
        await session.flush()

        # Commit creates a span
        await session.commit()

        return UserResponse(
            id=user.id,
            name=user.name,
            email=user.email,
            age=user.age,
            posts_count=0,
        )


@app.get("/posts", response_model=List[PostResponse])
async def list_posts(
    eager: bool = Query(default=False, description="Eager load authors"),
    limit: int = Query(default=100, le=1000),
):
    """
    List posts with optional eager loading.

    Demonstrates:
    - Eager loading vs lazy loading span differences
    - Query options (selectinload)
    - Reduced N+1 queries with eager loading
    """
    async with Session() as session:
        query = session.find(Post).limit(limit)

        if eager:
            # Eager loading: loads all authors in a single query
            # Creates a relationship span with strategy="selectinload"
            query = query.options(selectinload(Post.author))

        posts = await query.all()

        results = []
        for post in posts:
            # If eager=True, author is already loaded (no new span)
            # If eager=False, accessing author triggers lazy loading (creates span)
            author_name = post.author.name if post.author else None

            results.append(PostResponse(
                id=post.id,
                title=post.title,
                content=post.content,
                author_id=post.author_id,
                author_name=author_name,
            ))

        return results


@app.get("/posts/{post_id}", response_model=PostResponse)
async def get_post(post_id: int):
    """
    Get single post by ID with author.

    Demonstrates:
    - Primary key lookup span
    - Lazy relationship loading span
    - Span hierarchy: HTTP → Query → Relationship
    """
    async with Session() as session:
        post = await session.find(Post).filter(Post.id == post_id).first()

        if post is None:
            raise HTTPException(status_code=404, detail="Post not found")

        # Accessing author triggers lazy loading → creates relationship span
        author_name = post.author.name if post.author else None

        return PostResponse(
            id=post.id,
            title=post.title,
            content=post.content,
            author_id=post.author_id,
            author_name=author_name,
        )


@app.post("/posts", response_model=PostResponse, status_code=201)
async def create_post(post_data: PostCreate):
    """
    Create a new post.

    Demonstrates:
    - Insert span with foreign key relationship
    - Session pending count
    - Transaction commit span
    """
    async with Session() as session:
        # Verify author exists (creates a query span)
        author = await session.find(User).filter(User.id == post_data.author_id).first()
        if author is None:
            raise HTTPException(status_code=404, detail="Author not found")

        # Create post
        post = Post(
            title=post_data.title,
            content=post_data.content,
            author_id=post_data.author_id,
        )

        session.add(post)
        await session.flush()
        await session.commit()

        # Load author for response (creates relationship span)
        author_name = post.author.name if post.author else None

        return PostResponse(
            id=post.id,
            title=post.title,
            content=post.content,
            author_id=post.author_id,
            author_name=author_name,
        )


@app.delete("/users/{user_id}", status_code=204)
async def delete_user(user_id: int):
    """
    Delete user and cascade to posts.

    Demonstrates:
    - Delete span
    - Cascade operations tracked in spans
    """
    async with Session() as session:
        user = await session.find(User).filter(User.id == user_id).first()

        if user is None:
            raise HTTPException(status_code=404, detail="User not found")

        session.delete(user)
        await session.flush()
        await session.commit()


# ============================================================================
# Main Entry Point
# ============================================================================

if __name__ == "__main__":
    import uvicorn

    print("\n" + "=" * 70)
    print("FastAPI + data-bridge + OpenTelemetry Example")
    print("=" * 70)
    print(f"\nService: {APP_NAME}")
    print(f"Environment: {ENVIRONMENT}")
    print(f"OpenTelemetry: Enabled")
    print(f"OTLP Endpoint: {OTLP_ENDPOINT}")
    print(f"\nAPI Documentation: http://localhost:8000/docs")
    print(f"Jaeger UI: http://localhost:16686")
    print("\nExample requests:")
    print('  curl -X POST http://localhost:8000/users \\')
    print('       -H "Content-Type: application/json" \\')
    print('       -d \'{"name": "Alice", "email": "alice@example.com"}\'')
    print("\n  curl http://localhost:8000/users")
    print("\n  curl http://localhost:8000/posts?eager=true")
    print("\n" + "=" * 70 + "\n")

    # Run FastAPI with uvicorn
    uvicorn.run(
        app,
        host="0.0.0.0",
        port=8000,
        log_level="info",
    )

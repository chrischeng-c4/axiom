#!/usr/bin/env python3
"""
Test script for FastAPI + OpenTelemetry example.

This script demonstrates all API endpoints and generates traces.

Usage:
    # Start the API first
    python examples/fastapi_otel_example.py

    # In another terminal, run this script
    python examples/test_fastapi_otel.py

    # View traces in Jaeger
    open http://localhost:16686
"""

import asyncio
import httpx
import sys
from typing import Dict, Any


BASE_URL = "http://localhost:8000"


async def test_health_check():
    """Test health check endpoint."""
    print("\n" + "=" * 70)
    print("1. Health Check")
    print("=" * 70)

    async with httpx.AsyncClient() as client:
        response = await client.get(f"{BASE_URL}/")
        print(f"Status: {response.status_code}")
        print(f"Response: {response.json()}")
        return response.json()


async def create_user(name: str, email: str, age: int = None) -> Dict[str, Any]:
    """Create a new user."""
    async with httpx.AsyncClient() as client:
        data = {"name": name, "email": email}
        if age is not None:
            data["age"] = age

        response = await client.post(f"{BASE_URL}/users", json=data)
        if response.status_code == 201:
            user = response.json()
            print(f"✅ Created user: {user['name']} (ID: {user['id']})")
            return user
        else:
            print(f"❌ Failed to create user: {response.status_code}")
            print(f"   Response: {response.text}")
            return None


async def test_create_users():
    """Test user creation."""
    print("\n" + "=" * 70)
    print("2. Create Users")
    print("=" * 70)

    users = [
        ("Alice Johnson", "alice@example.com", 30),
        ("Bob Smith", "bob@example.com", 25),
        ("Charlie Brown", "charlie@example.com", 35),
        ("Diana Prince", "diana@example.com", 28),
        ("Eve Wilson", "eve@example.com", 32),
    ]

    created_users = []
    for name, email, age in users:
        user = await create_user(name, email, age)
        if user:
            created_users.append(user)
        await asyncio.sleep(0.1)  # Small delay for readability

    return created_users


async def test_list_users():
    """Test listing users."""
    print("\n" + "=" * 70)
    print("3. List Users")
    print("=" * 70)

    async with httpx.AsyncClient() as client:
        response = await client.get(f"{BASE_URL}/users")
        users = response.json()
        print(f"Found {len(users)} users:")
        for user in users:
            print(f"  - {user['name']} ({user['email']}) - {user.get('posts_count', 0)} posts")


async def test_get_user(user_id: int):
    """Test getting single user."""
    print("\n" + "=" * 70)
    print(f"4. Get User (ID: {user_id})")
    print("=" * 70)

    async with httpx.AsyncClient() as client:
        response = await client.get(f"{BASE_URL}/users/{user_id}")
        if response.status_code == 200:
            user = response.json()
            print(f"User: {user['name']} ({user['email']})")
            print(f"Posts: {user.get('posts_count', 0)}")
            return user
        else:
            print(f"❌ Failed to get user: {response.status_code}")
            return None


async def create_post(title: str, content: str, author_id: int) -> Dict[str, Any]:
    """Create a new post."""
    async with httpx.AsyncClient() as client:
        data = {
            "title": title,
            "content": content,
            "author_id": author_id,
        }
        response = await client.post(f"{BASE_URL}/posts", json=data)
        if response.status_code == 201:
            post = response.json()
            print(f"✅ Created post: {post['title']} by {post.get('author_name', 'Unknown')}")
            return post
        else:
            print(f"❌ Failed to create post: {response.status_code}")
            print(f"   Response: {response.text}")
            return None


async def test_create_posts(users):
    """Test post creation."""
    print("\n" + "=" * 70)
    print("5. Create Posts")
    print("=" * 70)

    if not users or len(users) == 0:
        print("⚠️ No users available to create posts")
        return []

    posts_data = [
        ("Introduction to Python", "Python is a versatile programming language...", 0),
        ("Data Engineering Basics", "Data engineering involves building pipelines...", 0),
        ("FastAPI Tutorial", "FastAPI is a modern web framework...", 1),
        ("Database Optimization", "Optimizing database queries is crucial...", 1),
        ("Distributed Tracing", "OpenTelemetry provides distributed tracing...", 2),
        ("Rust Performance", "Rust offers memory safety without GC...", 2),
        ("PostgreSQL Tips", "PostgreSQL is a powerful relational database...", 3),
        ("API Design Patterns", "RESTful APIs follow specific design patterns...", 3),
        ("Cloud Native Apps", "Cloud native applications are designed for...", 4),
        ("Observability Best Practices", "Observability includes logs, metrics, and traces...", 4),
    ]

    created_posts = []
    for title, content, user_idx in posts_data:
        if user_idx < len(users):
            author_id = users[user_idx]["id"]
            post = await create_post(title, content, author_id)
            if post:
                created_posts.append(post)
            await asyncio.sleep(0.1)

    return created_posts


async def test_list_posts_lazy():
    """Test listing posts with lazy loading (N+1 pattern)."""
    print("\n" + "=" * 70)
    print("6. List Posts (Lazy Loading)")
    print("=" * 70)
    print("⚠️ This demonstrates N+1 query pattern - check Jaeger for multiple spans")

    async with httpx.AsyncClient() as client:
        response = await client.get(f"{BASE_URL}/posts?limit=5")
        posts = response.json()
        print(f"Found {len(posts)} posts:")
        for post in posts:
            author = post.get('author_name', 'Unknown')
            print(f"  - {post['title']} by {author}")


async def test_list_posts_eager():
    """Test listing posts with eager loading (optimized)."""
    print("\n" + "=" * 70)
    print("7. List Posts (Eager Loading)")
    print("=" * 70)
    print("✅ This uses eager loading - check Jaeger for optimized query")

    async with httpx.AsyncClient() as client:
        response = await client.get(f"{BASE_URL}/posts?eager=true&limit=5")
        posts = response.json()
        print(f"Found {len(posts)} posts:")
        for post in posts:
            author = post.get('author_name', 'Unknown')
            print(f"  - {post['title']} by {author}")


async def test_get_post(post_id: int):
    """Test getting single post."""
    print("\n" + "=" * 70)
    print(f"8. Get Post (ID: {post_id})")
    print("=" * 70)

    async with httpx.AsyncClient() as client:
        response = await client.get(f"{BASE_URL}/posts/{post_id}")
        if response.status_code == 200:
            post = response.json()
            print(f"Post: {post['title']}")
            print(f"Content: {post['content'][:100]}...")
            print(f"Author: {post.get('author_name', 'Unknown')}")
            return post
        else:
            print(f"❌ Failed to get post: {response.status_code}")
            return None


async def test_pagination():
    """Test pagination."""
    print("\n" + "=" * 70)
    print("9. Test Pagination")
    print("=" * 70)

    async with httpx.AsyncClient() as client:
        # Page 1
        response = await client.get(f"{BASE_URL}/users?limit=2&offset=0")
        users_page1 = response.json()
        print(f"Page 1 (limit=2, offset=0): {len(users_page1)} users")
        for user in users_page1:
            print(f"  - {user['name']}")

        # Page 2
        response = await client.get(f"{BASE_URL}/users?limit=2&offset=2")
        users_page2 = response.json()
        print(f"Page 2 (limit=2, offset=2): {len(users_page2)} users")
        for user in users_page2:
            print(f"  - {user['name']}")


async def test_error_handling():
    """Test error handling and 404 responses."""
    print("\n" + "=" * 70)
    print("10. Test Error Handling")
    print("=" * 70)

    async with httpx.AsyncClient() as client:
        # Try to get non-existent user
        response = await client.get(f"{BASE_URL}/users/99999")
        print(f"GET /users/99999 → Status: {response.status_code}")
        if response.status_code == 404:
            print(f"✅ Correctly returned 404")
        else:
            print(f"❌ Expected 404, got {response.status_code}")

        # Try to get non-existent post
        response = await client.get(f"{BASE_URL}/posts/99999")
        print(f"GET /posts/99999 → Status: {response.status_code}")
        if response.status_code == 404:
            print(f"✅ Correctly returned 404")
        else:
            print(f"❌ Expected 404, got {response.status_code}")

        # Try to create post with non-existent author
        response = await client.post(
            f"{BASE_URL}/posts",
            json={"title": "Test", "content": "Test", "author_id": 99999}
        )
        print(f"POST /posts (invalid author) → Status: {response.status_code}")
        if response.status_code == 404:
            print(f"✅ Correctly returned 404")
        else:
            print(f"❌ Expected 404, got {response.status_code}")


async def main():
    """Run all tests."""
    print("\n" + "=" * 70)
    print("FastAPI + data-bridge + OpenTelemetry Test Suite")
    print("=" * 70)
    print("\nThis script will:")
    print("1. Create sample users and posts")
    print("2. Test various API endpoints")
    print("3. Generate traces for Jaeger")
    print("\nMake sure the API is running: python examples/fastapi_otel_example.py")
    print("View traces at: http://localhost:16686")
    print("\nPress Ctrl+C to cancel...")

    try:
        await asyncio.sleep(2)

        # Test health check
        health = await test_health_check()

        if not health.get("tracing"):
            print("\n⚠️ WARNING: Tracing is not enabled!")
            print("Set DATA_BRIDGE_TRACING_ENABLED=true")

        # Create users
        users = await test_create_users()

        # List users
        await test_list_users()

        # Get single user
        if users:
            await test_get_user(users[0]["id"])

        # Create posts
        posts = await test_create_posts(users)

        # List posts with lazy loading
        await test_list_posts_lazy()

        # List posts with eager loading
        await test_list_posts_eager()

        # Get single post
        if posts:
            await test_get_post(posts[0]["id"])

        # Test pagination
        await test_pagination()

        # Test error handling
        await test_error_handling()

        # Summary
        print("\n" + "=" * 70)
        print("Test Suite Completed!")
        print("=" * 70)
        print(f"✅ Created {len(users)} users")
        print(f"✅ Created {len(posts)} posts")
        print(f"✅ Tested all endpoints")
        print("\nView traces in Jaeger:")
        print("1. Open http://localhost:16686")
        print("2. Select service: fastapi-databridge-api")
        print("3. Click 'Find Traces'")
        print("\nLook for:")
        print("  - HTTP spans (GET /users, POST /posts, etc.)")
        print("  - Query spans (db.query.find, db.query.insert)")
        print("  - Relationship spans (db.relationship.load)")
        print("  - Session spans (db.session.flush, db.session.commit)")
        print("=" * 70 + "\n")

    except httpx.ConnectError:
        print("\n❌ ERROR: Could not connect to API")
        print("Make sure the API is running:")
        print("  python examples/fastapi_otel_example.py")
        sys.exit(1)
    except KeyboardInterrupt:
        print("\n\n⚠️ Cancelled by user")
        sys.exit(0)
    except Exception as e:
        print(f"\n❌ ERROR: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    asyncio.run(main())

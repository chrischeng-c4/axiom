"""Example: HTTP Client Integration with data-bridge-api

This example demonstrates how to use the HTTP client integration
to make external API calls from route handlers.
"""
import asyncio
from cclab.quasar import App, RequestContext
from cclab.photon import HttpClient


async def main():
    # Create app
    app = App(
        title="HTTP Client Integration Demo",
        version="1.0.0"
    )

    # Configure HTTP client with base URL
    app.configure_http_client(
        base_url="https://jsonplaceholder.typicode.com",
        timeout=30.0,
        connect_timeout=10.0
    )

    # Example 1: Use HttpClient as a dependency with bare type hint
    @app.get("/posts/{post_id}")
    async def get_post(post_id: int, http: HttpClient) -> dict:
        """Fetch a post from external API."""
        response = await http.get(f"/posts/{post_id}")
        if response.is_success():
            return response.json()
        return {"error": "Post not found"}

    # Example 2: Multiple external API calls in one handler
    @app.get("/user-posts/{user_id}")
    async def get_user_with_posts(user_id: int, http: HttpClient) -> dict:
        """Fetch user and their posts."""
        # Get user
        user_resp = await http.get(f"/users/{user_id}")
        user = user_resp.json() if user_resp.is_success() else None

        # Get user's posts (params must be strings)
        posts_resp = await http.get(f"/posts", params={"userId": str(user_id)})
        posts = posts_resp.json() if posts_resp.is_success() else []

        return {
            "user": user,
            "posts": posts
        }

    # Example 3: Using RequestContext for HTTP client access
    @app.post("/proxy")
    async def proxy_request(ctx: RequestContext, data: dict) -> dict:
        """Proxy a request to external API."""
        response = await ctx.http.post("/posts", json=data)
        return {
            "status": response.status_code,
            "latency_ms": response.latency_ms,
            "data": response.json() if response.is_success() else None
        }

    # Example 4: Direct access to HTTP client (outside of handlers)
    print("Direct HTTP client usage:")
    client = app.http_client
    response = await client.get("/posts/1")
    post = response.json()
    print(f"  Post {post['id']}: {post['title']}")
    print(f"  Latency: {response.latency_ms}ms")

    # Test handlers by resolving dependencies manually
    print("\nTesting route handlers:")

    from cclab.quasar.dependencies import RequestContext as DepContext

    # Test get_post
    context = DepContext()
    deps = await app.resolve_dependencies(get_post, context)
    result = await get_post(post_id=2, http=deps['http'])
    print(f"  get_post(2): {result['title']}")

    # Test get_user_with_posts
    deps = await app.resolve_dependencies(get_user_with_posts, context)
    result = await get_user_with_posts(user_id=1, http=deps['http'])
    print(f"  get_user_with_posts(1): {result['user']['name']} has {len(result['posts'])} posts")

    # Test proxy_request
    api_ctx = RequestContext(_http_client=client)
    result = await proxy_request(
        ctx=api_ctx,
        data={"title": "Test", "body": "Test body", "userId": 1}
    )
    print(f"  proxy_request: Status {result['status']}, Latency {result['latency_ms']}ms")

    print("\n✅ All examples completed successfully!")


if __name__ == "__main__":
    asyncio.run(main())

"""
HTTP client tests for data-bridge.

Tests the HttpClient and HttpResponse functionality against httpbin.org.
"""
from cclab.photon import HttpClient
from cclab.probe import test, expect
from tests.base import HttpTestSuite


class TestHttpClientBasics(HttpTestSuite):
    """Basic HTTP client functionality tests."""

    async def setup_suite(self):
        """Create shared HTTP client for tests."""
        self.client = HttpClient(
            base_url="https://httpbin.org",
            timeout=30.0,
        )

    @test(tags=["http", "client"])
    async def test_get_request(self):
        """Test basic GET request."""
        response = await self.client.get("/get")
        expect(response.status_code).to_equal(200)
        expect(response.is_success()).to_be_true()

    @test(tags=["http", "client"])
    async def test_get_with_params(self):
        """Test GET request with query parameters."""
        response = await self.client.get("/get", params={"foo": "bar", "baz": "123"})
        expect(response.status_code).to_equal(200)
        data = response.json()
        expect(data["args"]["foo"]).to_equal("bar")
        expect(data["args"]["baz"]).to_equal("123")

    @test(tags=["http", "client"])
    async def test_get_with_headers(self):
        """Test GET request with custom headers."""
        response = await self.client.get(
            "/headers",
            headers={"X-Custom-Header": "test-value"}
        )
        expect(response.status_code).to_equal(200)
        data = response.json()
        expect(data["headers"].get("X-Custom-Header")).to_equal("test-value")

    @test(tags=["http", "client"])
    async def test_post_json(self):
        """Test POST request with JSON body."""
        payload = {"name": "Alice", "age": 30}
        response = await self.client.post("/post", json=payload)
        expect(response.status_code).to_equal(200)
        data = response.json()
        expect(data["json"]).to_equal(payload)

    @test(tags=["http", "client"])
    async def test_post_form(self):
        """Test POST request with form data."""
        form_data = {"username": "alice", "password": "secret"}
        response = await self.client.post("/post", form=form_data)
        expect(response.status_code).to_equal(200)
        data = response.json()
        expect(data["form"]["username"]).to_equal("alice")
        expect(data["form"]["password"]).to_equal("secret")

    @test(tags=["http", "client"])
    async def test_put_request(self):
        """Test PUT request."""
        payload = {"updated": True}
        response = await self.client.put("/put", json=payload)
        expect(response.status_code).to_equal(200)
        data = response.json()
        expect(data["json"]).to_equal(payload)

    @test(tags=["http", "client"])
    async def test_patch_request(self):
        """Test PATCH request."""
        payload = {"partial": "update"}
        response = await self.client.patch("/patch", json=payload)
        expect(response.status_code).to_equal(200)
        data = response.json()
        expect(data["json"]["partial"]).to_equal("update")

    @test(tags=["http", "client"])
    async def test_delete_request(self):
        """Test DELETE request."""
        response = await self.client.delete("/delete")
        expect(response.status_code).to_equal(200)

    @test(tags=["http", "client"])
    async def test_head_request(self):
        """Test HEAD request (no body)."""
        response = await self.client.head("/get")
        expect(response.status_code).to_equal(200)
        # HEAD has no body
        expect(response.text()).to_equal("")


class TestHttpResponse(HttpTestSuite):
    """Test HttpResponse methods."""

    async def setup_suite(self):
        """Create shared HTTP client for tests."""
        self.client = HttpClient(base_url="https://httpbin.org")

    @test(tags=["http", "response"])
    async def test_response_status_code(self):
        """Test status_code attribute."""
        response = await self.client.get("/status/201")
        expect(response.status_code).to_equal(201)

    @test(tags=["http", "response"])
    async def test_response_is_success(self):
        """Test is_success() for 2xx codes."""
        response_200 = await self.client.get("/status/200")
        response_201 = await self.client.get("/status/201")
        response_204 = await self.client.get("/status/204")

        expect(response_200.is_success()).to_be_true()
        expect(response_201.is_success()).to_be_true()
        expect(response_204.is_success()).to_be_true()

    @test(tags=["http", "response"])
    async def test_response_is_client_error(self):
        """Test is_client_error() for 4xx codes."""
        response = await self.client.get("/status/404")
        expect(response.is_client_error()).to_be_true()
        expect(response.is_success()).to_be_false()
        expect(response.is_server_error()).to_be_false()

    @test(tags=["http", "response"])
    async def test_response_is_server_error(self):
        """Test is_server_error() for 5xx codes."""
        response = await self.client.get("/status/500")
        expect(response.is_server_error()).to_be_true()
        expect(response.is_success()).to_be_false()
        expect(response.is_client_error()).to_be_false()

    @test(tags=["http", "response"])
    async def test_response_json(self):
        """Test json() method for JSON parsing."""
        response = await self.client.get("/json")
        expect(response.status_code).to_equal(200)
        data = response.json()
        expect(isinstance(data, dict)).to_be_true()

    @test(tags=["http", "response"])
    async def test_response_text(self):
        """Test text() method."""
        response = await self.client.get("/html")
        expect(response.status_code).to_equal(200)
        text = response.text()
        expect("Herman Melville" in text).to_be_true()

    @test(tags=["http", "response"])
    async def test_response_content_type(self):
        """Test content_type() method."""
        response = await self.client.get("/json")
        content_type = response.content_type()
        expect("application/json" in content_type).to_be_true()

    @test(tags=["http", "response"])
    async def test_response_latency(self):
        """Test latency_ms attribute."""
        response = await self.client.get("/get")
        # Latency should be positive
        expect(response.latency_ms > 0).to_be_true()
        # Latency should be reasonable (< 30 seconds)
        expect(response.latency_ms < 30000).to_be_true()

    @test(tags=["http", "response"])
    async def test_response_url(self):
        """Test url attribute after request."""
        response = await self.client.get("/get")
        expect("httpbin.org/get" in response.url).to_be_true()


class TestHttpClientConfiguration(HttpTestSuite):
    """Test HttpClient configuration options."""

    @test(tags=["http", "config"])
    async def test_client_without_base_url(self):
        """Test client without base_url uses full URLs."""
        client = HttpClient(timeout=30.0)
        response = await client.get("https://httpbin.org/get")
        expect(response.status_code).to_equal(200)

    @test(tags=["http", "config"])
    async def test_client_with_custom_user_agent(self):
        """Test custom User-Agent header."""
        client = HttpClient(
            base_url="https://httpbin.org",
            user_agent="data-bridge-test/1.0"
        )
        response = await client.get("/user-agent")
        expect(response.status_code).to_equal(200)
        data = response.json()
        # httpbin returns lowercase key
        user_agent = data.get("user-agent") or data.get("User-Agent")
        expect(user_agent).to_equal("data-bridge-test/1.0")

    @test(tags=["http", "config"])
    async def test_client_follows_redirects(self):
        """Test redirect following (default behavior)."""
        client = HttpClient(
            base_url="https://httpbin.org",
            follow_redirects=True,
        )
        response = await client.get("/redirect/2")  # Redirects 2 times
        expect(response.status_code).to_equal(200)
        expect("httpbin.org/get" in response.url).to_be_true()


# Run tests when executed directly
if __name__ == "__main__":
    from cclab.probe import run_suites

    run_suites([
        TestHttpClientBasics,
        TestHttpResponse,
        TestHttpClientConfiguration,
    ], verbose=True)

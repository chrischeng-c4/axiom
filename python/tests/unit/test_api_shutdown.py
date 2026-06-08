"""Tests for App shutdown and startup hooks."""
import asyncio
import pytest
from cclab.quasar import App, setup_signal_handlers


class TestStartupShutdownHooks:
    """Test startup and shutdown hook functionality."""

    @pytest.mark.asyncio
    async def test_on_startup_decorator(self):
        """Test that on_startup decorator registers hooks."""
        app = App()
        call_order = []

        @app.on_startup
        async def startup_hook1():
            call_order.append("startup1")

        @app.on_startup
        def startup_hook2():
            call_order.append("startup2")

        await app.startup()
        assert call_order == ["startup1", "startup2"]

    @pytest.mark.asyncio
    async def test_on_shutdown_decorator(self):
        """Test that on_shutdown decorator registers hooks."""
        app = App()
        call_order = []

        @app.on_shutdown
        async def shutdown_hook1():
            call_order.append("shutdown1")

        @app.on_shutdown
        def shutdown_hook2():
            call_order.append("shutdown2")

        await app.shutdown()
        # Shutdown hooks run in LIFO order
        assert call_order == ["shutdown2", "shutdown1"]

    @pytest.mark.asyncio
    async def test_shutdown_lifo_order(self):
        """Test that shutdown hooks run in LIFO (reverse) order."""
        app = App()
        call_order = []

        @app.on_shutdown
        async def first():
            call_order.append(1)

        @app.on_shutdown
        async def second():
            call_order.append(2)

        @app.on_shutdown
        async def third():
            call_order.append(3)

        await app.shutdown()
        assert call_order == [3, 2, 1]

    @pytest.mark.asyncio
    async def test_startup_fifo_order(self):
        """Test that startup hooks run in FIFO (registration) order."""
        app = App()
        call_order = []

        @app.on_startup
        async def first():
            call_order.append(1)

        @app.on_startup
        async def second():
            call_order.append(2)

        @app.on_startup
        async def third():
            call_order.append(3)

        await app.startup()
        assert call_order == [1, 2, 3]

    @pytest.mark.asyncio
    async def test_shutdown_timeout(self):
        """Test that shutdown hooks timeout properly."""
        app = App(shutdown_timeout=0.1)
        completed = []

        @app.on_shutdown
        async def slow_hook():
            await asyncio.sleep(1.0)  # Longer than timeout
            completed.append("slow")

        @app.on_shutdown
        async def fast_hook():
            completed.append("fast")

        # Should not raise, should continue with other hooks
        await app.shutdown()
        assert "fast" in completed
        assert "slow" not in completed

    @pytest.mark.asyncio
    async def test_shutdown_continues_on_error(self):
        """Test that shutdown continues even if a hook raises an error."""
        app = App()
        completed = []

        @app.on_shutdown
        async def good_hook1():
            completed.append("good1")

        @app.on_shutdown
        async def bad_hook():
            raise ValueError("Hook error")

        @app.on_shutdown
        async def good_hook2():
            completed.append("good2")

        # Should not raise, should continue with other hooks
        await app.shutdown()
        assert "good1" in completed
        assert "good2" in completed

    @pytest.mark.asyncio
    async def test_is_shutting_down_flag(self):
        """Test that is_shutting_down flag is set correctly."""
        app = App()
        assert not app.is_shutting_down

        @app.on_shutdown
        async def check_flag():
            assert app.is_shutting_down

        await app.shutdown()
        assert app.is_shutting_down

    @pytest.mark.asyncio
    async def test_custom_shutdown_timeout(self):
        """Test that custom shutdown timeout can be provided."""
        app = App(shutdown_timeout=5.0)
        completed = []

        @app.on_shutdown
        async def hook():
            await asyncio.sleep(0.05)
            completed.append("done")

        # Use custom timeout
        await app.shutdown(timeout=0.1)
        assert "done" in completed

    @pytest.mark.asyncio
    async def test_http_client_cleanup(self):
        """Test that HTTP client is cleaned up on shutdown."""
        app = App()
        app.configure_http_client(base_url="https://example.com")

        # Get client to initialize it
        _ = app.http_client

        # Provider should have client
        assert app._http_provider._client is not None

        await app.shutdown()

        # Provider should have cleared client
        assert app._http_provider._client is None

    @pytest.mark.asyncio
    async def test_mixed_sync_async_hooks(self):
        """Test mixing sync and async hooks."""
        app = App()
        call_order = []

        @app.on_startup
        def sync_startup():
            call_order.append("sync_start")

        @app.on_startup
        async def async_startup():
            call_order.append("async_start")

        @app.on_shutdown
        def sync_shutdown():
            call_order.append("sync_stop")

        @app.on_shutdown
        async def async_shutdown():
            call_order.append("async_stop")

        await app.startup()
        await app.shutdown()

        assert call_order == [
            "sync_start",
            "async_start",
            "async_stop",
            "sync_stop",
        ]


class TestSignalHandlers:
    """Test signal handler setup."""

    def test_setup_signal_handlers(self):
        """Test that signal handlers can be set up."""
        import signal

        app = App()

        # Get original handlers
        original_sigterm = signal.getsignal(signal.SIGTERM)
        original_sigint = signal.getsignal(signal.SIGINT)

        try:
            setup_signal_handlers(app)

            # Check handlers were changed
            new_sigterm = signal.getsignal(signal.SIGTERM)
            new_sigint = signal.getsignal(signal.SIGINT)

            assert new_sigterm != original_sigterm
            assert new_sigint != original_sigint

        finally:
            # Restore original handlers
            signal.signal(signal.SIGTERM, original_sigterm)
            signal.signal(signal.SIGINT, original_sigint)


class TestShutdownTimeout:
    """Test shutdown timeout configuration."""

    def test_default_shutdown_timeout(self):
        """Test default shutdown timeout is 30 seconds."""
        app = App()
        assert app._shutdown_timeout == 30.0

    def test_custom_shutdown_timeout(self):
        """Test custom shutdown timeout can be set."""
        app = App(shutdown_timeout=60.0)
        assert app._shutdown_timeout == 60.0

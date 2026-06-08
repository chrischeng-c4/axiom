"""Tests for App lifespan context manager."""
import asyncio
import pytest
from contextlib import asynccontextmanager
from cclab.quasar import App, AppState


class TestLifespanContextManager:
    """Test lifespan context manager functionality."""

    @pytest.mark.asyncio
    async def test_basic_lifespan_context(self):
        """Test basic lifespan context manager usage without custom lifespan."""
        app = App()
        call_order = []

        @app.on_startup
        async def startup():
            call_order.append("startup")

        @app.on_shutdown
        async def shutdown():
            call_order.append("shutdown")

        async with app.lifespan_context():
            call_order.append("running")

        assert call_order == ["startup", "running", "shutdown"]

    @pytest.mark.asyncio
    async def test_custom_lifespan_function(self):
        """Test custom lifespan function integration."""
        call_order = []

        @asynccontextmanager
        async def lifespan(app: App):
            call_order.append("lifespan_start")
            yield
            call_order.append("lifespan_end")

        app = App(lifespan=lifespan)

        @app.on_startup
        async def startup():
            call_order.append("startup")

        @app.on_shutdown
        async def shutdown():
            call_order.append("shutdown")

        async with app.lifespan_context():
            call_order.append("running")

        # Order: startup hooks -> lifespan start -> running -> lifespan end -> shutdown hooks
        assert call_order == [
            "startup",
            "lifespan_start",
            "running",
            "lifespan_end",
            "shutdown",
        ]

    @pytest.mark.asyncio
    async def test_app_state_storage(self):
        """Test AppState storage and retrieval."""
        @asynccontextmanager
        async def lifespan(app: App):
            # Startup
            app.state.db = "mock_db_connection"
            app.state.cache = "mock_cache"
            yield
            # Shutdown
            app.state.db = None
            app.state.cache = None

        app = App(lifespan=lifespan)

        # State should be empty initially
        assert not hasattr(app.state, "db")
        assert not hasattr(app.state, "cache")

        async with app.lifespan_context():
            # State should be populated during lifespan
            assert app.state.db == "mock_db_connection"
            assert app.state.cache == "mock_cache"

        # State should be cleaned up after lifespan
        assert app.state.db is None
        assert app.state.cache is None

    @pytest.mark.asyncio
    async def test_startup_shutdown_order_with_lifespan(self):
        """Test that startup/shutdown hooks execute in correct order with lifespan."""
        execution_log = []

        @asynccontextmanager
        async def lifespan(app: App):
            execution_log.append(("lifespan", "enter"))
            yield
            execution_log.append(("lifespan", "exit"))

        app = App(lifespan=lifespan)

        @app.on_startup
        async def startup1():
            execution_log.append(("startup", 1))

        @app.on_startup
        async def startup2():
            execution_log.append(("startup", 2))

        @app.on_shutdown
        async def shutdown1():
            execution_log.append(("shutdown", 1))

        @app.on_shutdown
        async def shutdown2():
            execution_log.append(("shutdown", 2))

        async with app.lifespan_context():
            execution_log.append(("app", "running"))

        # Verify complete execution order
        assert execution_log == [
            ("startup", 1),
            ("startup", 2),
            ("lifespan", "enter"),
            ("app", "running"),
            ("lifespan", "exit"),
            ("shutdown", 2),  # LIFO order
            ("shutdown", 1),
        ]

    @pytest.mark.asyncio
    async def test_multiple_state_attributes(self):
        """Test storing and accessing multiple state attributes."""
        @asynccontextmanager
        async def lifespan(app: App):
            app.state.value1 = 42
            app.state.value2 = "test"
            app.state.value3 = [1, 2, 3]
            app.state.nested = {"key": "value"}
            yield

        app = App(lifespan=lifespan)

        async with app.lifespan_context():
            assert app.state.value1 == 42
            assert app.state.value2 == "test"
            assert app.state.value3 == [1, 2, 3]
            assert app.state.nested == {"key": "value"}

    @pytest.mark.asyncio
    async def test_lifespan_cleanup_on_error(self):
        """Test that lifespan cleanup happens even on error."""
        cleanup_called = []

        @asynccontextmanager
        async def lifespan(app: App):
            app.state.resource = "allocated"
            try:
                yield
            finally:
                cleanup_called.append(True)
                app.state.resource = None

        app = App(lifespan=lifespan)

        with pytest.raises(RuntimeError):
            async with app.lifespan_context():
                raise RuntimeError("Test error")

        # Cleanup should still have been called
        assert cleanup_called == [True]
        assert app.state.resource is None

    @pytest.mark.asyncio
    async def test_lifespan_without_custom_function(self):
        """Test lifespan context manager with default behavior (no custom lifespan)."""
        app = App()  # No lifespan parameter
        call_order = []

        @app.on_startup
        async def startup():
            call_order.append("startup")
            app.state.initialized = True

        @app.on_shutdown
        async def shutdown():
            call_order.append("shutdown")
            app.state.initialized = False

        async with app.lifespan_context():
            assert app.state.initialized
            call_order.append("running")

        assert call_order == ["startup", "running", "shutdown"]
        assert not app.state.initialized

    @pytest.mark.asyncio
    async def test_lifespan_with_resource_management(self):
        """Test lifespan context manager with realistic resource management."""
        class MockDatabase:
            def __init__(self):
                self.connected = False

            async def connect(self):
                self.connected = True

            async def close(self):
                self.connected = False

        @asynccontextmanager
        async def lifespan(app: App):
            # Startup: connect to database
            db = MockDatabase()
            await db.connect()
            app.state.db = db
            yield
            # Shutdown: close database
            await db.close()

        app = App(lifespan=lifespan)

        async with app.lifespan_context():
            # Database should be connected
            assert app.state.db.connected

        # Database should be disconnected
        assert not app.state.db.connected

    @pytest.mark.asyncio
    async def test_nested_startup_shutdown_with_lifespan(self):
        """Test that startup/shutdown hooks are properly wrapped by lifespan."""
        execution_order = []

        @asynccontextmanager
        async def lifespan(app: App):
            execution_order.append("lifespan_enter")
            app.state.lifespan_active = True
            yield
            app.state.lifespan_active = False
            execution_order.append("lifespan_exit")

        app = App(lifespan=lifespan)

        @app.on_startup
        async def verify_startup():
            # Startup hooks run before lifespan enters
            assert not hasattr(app.state, "lifespan_active")
            execution_order.append("startup_hook")

        @app.on_shutdown
        async def verify_shutdown():
            # Shutdown hooks run after lifespan exits
            assert not app.state.lifespan_active
            execution_order.append("shutdown_hook")

        async with app.lifespan_context():
            # Inside context, lifespan should be active
            assert app.state.lifespan_active
            execution_order.append("context_body")

        # Verify complete order
        assert execution_order == [
            "startup_hook",
            "lifespan_enter",
            "context_body",
            "lifespan_exit",
            "shutdown_hook",
        ]

    @pytest.mark.asyncio
    async def test_app_state_is_persistent(self):
        """Test that AppState instance persists across the app lifetime."""
        app = App()

        # Get state reference before lifespan
        state_before = app.state

        async with app.lifespan_context():
            # State reference should be the same
            assert app.state is state_before
            app.state.value = 42

        # State reference should still be the same after lifespan
        assert app.state is state_before
        assert app.state.value == 42

    @pytest.mark.asyncio
    async def test_shutdown_called_even_on_lifespan_error(self):
        """Test that shutdown hooks are called even if lifespan raises an error."""
        shutdown_called = []

        @asynccontextmanager
        async def lifespan(app: App):
            app.state.resource = "allocated"
            yield
            raise RuntimeError("Lifespan cleanup error")

        app = App(lifespan=lifespan)

        @app.on_shutdown
        async def cleanup():
            shutdown_called.append(True)

        with pytest.raises(RuntimeError, match="Lifespan cleanup error"):
            async with app.lifespan_context():
                pass

        # Shutdown hook should still be called
        assert shutdown_called == [True]


class TestAppStateClass:
    """Test AppState class functionality."""

    def test_app_state_creation(self):
        """Test that AppState can be created and used."""
        state = AppState()
        assert isinstance(state, AppState)

    def test_app_state_dynamic_attributes(self):
        """Test that AppState allows dynamic attribute assignment."""
        state = AppState()

        # Should be able to set arbitrary attributes
        state.db = "database"
        state.cache = "cache"
        state.config = {"key": "value"}

        assert state.db == "database"
        assert state.cache == "cache"
        assert state.config == {"key": "value"}

    def test_app_has_state_attribute(self):
        """Test that App instances have a state attribute."""
        app = App()
        assert hasattr(app, "state")
        assert isinstance(app.state, AppState)

    def test_app_state_independent_per_app(self):
        """Test that each App instance has its own AppState."""
        app1 = App()
        app2 = App()

        app1.state.value = 1
        app2.state.value = 2

        assert app1.state.value == 1
        assert app2.state.value == 2
        assert app1.state is not app2.state

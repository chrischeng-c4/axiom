"""
Tests for health check endpoints and HealthManager.
"""
import pytest
import asyncio
from cclab.quasar import App, HealthManager, HealthCheck, HealthStatus


class TestHealthCheck:
    """Test HealthCheck dataclass"""

    def test_health_check_creation(self):
        """Test creating a HealthCheck"""
        def check():
            return True

        hc = HealthCheck(name="test", check=check, critical=True)
        assert hc.name == "test"
        assert hc.check is check
        assert hc.critical is True

    def test_health_check_default_critical(self):
        """Test HealthCheck defaults to critical=True"""
        def check():
            return True

        hc = HealthCheck(name="test", check=check)
        assert hc.critical is True


class TestHealthStatus:
    """Test HealthStatus dataclass"""

    def test_health_status_creation(self):
        """Test creating a HealthStatus"""
        status = HealthStatus(status="healthy", checks={"db": True, "cache": True})
        assert status.status == "healthy"
        assert status.checks == {"db": True, "cache": True}

    def test_health_status_to_dict(self):
        """Test HealthStatus.to_dict()"""
        status = HealthStatus(status="degraded", checks={"db": True, "cache": False})
        result = status.to_dict()
        assert result == {
            "status": "degraded",
            "checks": {"db": True, "cache": False}
        }

    def test_health_status_empty_checks(self):
        """Test HealthStatus with no checks"""
        status = HealthStatus(status="healthy")
        assert status.checks == {}
        assert status.to_dict() == {"status": "healthy", "checks": {}}


class TestHealthManager:
    """Test HealthManager class"""

    def test_health_manager_init(self):
        """Test HealthManager initialization"""
        manager = HealthManager()
        assert manager._checks == []
        assert manager._is_ready is False

    def test_add_check(self):
        """Test adding a health check"""
        manager = HealthManager()

        def check():
            return True

        manager.add_check("test", check, critical=True)
        assert len(manager._checks) == 1
        assert manager._checks[0].name == "test"
        assert manager._checks[0].check is check
        assert manager._checks[0].critical is True

    def test_add_multiple_checks(self):
        """Test adding multiple health checks"""
        manager = HealthManager()

        def check1():
            return True

        def check2():
            return False

        manager.add_check("check1", check1, critical=True)
        manager.add_check("check2", check2, critical=False)

        assert len(manager._checks) == 2
        assert manager._checks[0].name == "check1"
        assert manager._checks[1].name == "check2"

    def test_set_ready(self):
        """Test setting readiness state"""
        manager = HealthManager()
        assert manager._is_ready is False

        manager.set_ready(True)
        assert manager._is_ready is True

        manager.set_ready(False)
        assert manager._is_ready is False

    def test_is_live(self):
        """Test liveness check"""
        manager = HealthManager()
        assert manager.is_live() is True  # Should always return True

    @pytest.mark.asyncio
    async def test_check_health_all_healthy(self):
        """Test check_health when all checks pass"""
        manager = HealthManager()

        def check1():
            return True

        def check2():
            return True

        manager.add_check("check1", check1)
        manager.add_check("check2", check2)

        status = await manager.check_health()
        assert status.status == "healthy"
        assert status.checks == {"check1": True, "check2": True}

    @pytest.mark.asyncio
    async def test_check_health_critical_failure(self):
        """Test check_health with critical check failure"""
        manager = HealthManager()

        def good_check():
            return True

        def bad_check():
            return False

        manager.add_check("good", good_check, critical=False)
        manager.add_check("bad", bad_check, critical=True)

        status = await manager.check_health()
        assert status.status == "unhealthy"
        assert status.checks == {"good": True, "bad": False}

    @pytest.mark.asyncio
    async def test_check_health_non_critical_failure(self):
        """Test check_health with non-critical check failure"""
        manager = HealthManager()

        def good_check():
            return True

        def bad_check():
            return False

        manager.add_check("good", good_check, critical=True)
        manager.add_check("bad", bad_check, critical=False)

        status = await manager.check_health()
        assert status.status == "degraded"
        assert status.checks == {"good": True, "bad": False}

    @pytest.mark.asyncio
    async def test_check_health_async_check(self):
        """Test check_health with async check functions"""
        manager = HealthManager()

        async def async_check():
            await asyncio.sleep(0.01)
            return True

        manager.add_check("async", async_check)

        status = await manager.check_health()
        assert status.status == "healthy"
        assert status.checks == {"async": True}

    @pytest.mark.asyncio
    async def test_check_health_exception_handling(self):
        """Test check_health handles exceptions"""
        manager = HealthManager()

        def failing_check():
            raise RuntimeError("Check failed")

        manager.add_check("failing", failing_check, critical=True)

        status = await manager.check_health()
        assert status.status == "unhealthy"
        assert status.checks == {"failing": False}

    @pytest.mark.asyncio
    async def test_check_health_mixed_async_sync(self):
        """Test check_health with mixed async and sync checks"""
        manager = HealthManager()

        def sync_check():
            return True

        async def async_check():
            await asyncio.sleep(0.01)
            return True

        manager.add_check("sync", sync_check)
        manager.add_check("async", async_check)

        status = await manager.check_health()
        assert status.status == "healthy"
        assert status.checks == {"sync": True, "async": True}

    @pytest.mark.asyncio
    async def test_is_ready_when_not_ready(self):
        """Test is_ready when readiness is False"""
        manager = HealthManager()
        manager.set_ready(False)

        is_ready = await manager.is_ready()
        assert is_ready is False

    @pytest.mark.asyncio
    async def test_is_ready_when_ready_and_healthy(self):
        """Test is_ready when ready and all checks pass"""
        manager = HealthManager()

        def check():
            return True

        manager.add_check("test", check)
        manager.set_ready(True)

        is_ready = await manager.is_ready()
        assert is_ready is True

    @pytest.mark.asyncio
    async def test_is_ready_when_ready_but_unhealthy(self):
        """Test is_ready when ready but critical check fails"""
        manager = HealthManager()

        def check():
            return False

        manager.add_check("test", check, critical=True)
        manager.set_ready(True)

        is_ready = await manager.is_ready()
        assert is_ready is False

    @pytest.mark.asyncio
    async def test_is_ready_when_ready_and_degraded(self):
        """Test is_ready when ready but non-critical check fails"""
        manager = HealthManager()

        def good_check():
            return True

        def bad_check():
            return False

        manager.add_check("good", good_check, critical=True)
        manager.add_check("bad", bad_check, critical=False)
        manager.set_ready(True)

        is_ready = await manager.is_ready()
        assert is_ready is True  # Degraded is still ready


class TestAppHealthIntegration:
    """Test health integration with App class"""

    def test_app_has_health_manager(self):
        """Test App has health manager attribute"""
        app = App()
        assert hasattr(app, "_health_manager")
        assert isinstance(app._health_manager, HealthManager)

    def test_app_health_property(self):
        """Test App.health property"""
        app = App()
        assert app.health is app._health_manager

    def test_app_startup_sets_ready(self):
        """Test App startup sets health to ready"""
        app = App()
        assert app._health_manager._is_ready is False

        # Startup hook should be registered
        assert len(app._startup_hooks) > 0

    @pytest.mark.asyncio
    async def test_app_startup_execution_sets_ready(self):
        """Test executing startup actually sets ready"""
        app = App()
        assert app._health_manager._is_ready is False

        await app.startup()
        assert app._health_manager._is_ready is True

    @pytest.mark.asyncio
    async def test_app_shutdown_sets_not_ready(self):
        """Test App shutdown sets health to not ready"""
        app = App()
        await app.startup()
        assert app._health_manager._is_ready is True

        await app.shutdown()
        assert app._health_manager._is_ready is False

    def test_include_health_routes_registers_endpoints(self):
        """Test include_health_routes registers routes"""
        app = App()
        app.include_health_routes()

        # Check routes were registered
        route_paths = [r.path for r in app.routes]
        assert "/health" in route_paths
        assert "/live" in route_paths
        assert "/ready" in route_paths

    def test_include_health_routes_with_prefix(self):
        """Test include_health_routes with custom prefix"""
        app = App()
        app.include_health_routes(prefix="/api")

        route_paths = [r.path for r in app.routes]
        assert "/api/health" in route_paths
        assert "/api/live" in route_paths
        assert "/api/ready" in route_paths

    def test_health_routes_have_correct_tags(self):
        """Test health routes are tagged correctly"""
        app = App()
        app.include_health_routes()

        for route in app.routes:
            if route.path in ["/health", "/live", "/ready"]:
                assert "health" in route.tags

    @pytest.mark.asyncio
    async def test_custom_health_check_integration(self):
        """Test adding custom health checks to App"""
        app = App()

        call_count = {"value": 0}

        def custom_check():
            call_count["value"] += 1
            return True

        app.health.add_check("custom", custom_check)

        status = await app.health.check_health()
        assert status.checks["custom"] is True
        assert call_count["value"] == 1

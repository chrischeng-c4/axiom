"""
Example demonstrating K8s health check endpoints.

This shows how to:
1. Add built-in health routes
2. Register custom health checks
3. Use async health checks
"""
import asyncio
from cclab.quasar import App


# Create app
app = App(title="Health Check Example", version="1.0.0")


# Add custom health checks
def check_database():
    """Simulate database connection check"""
    # In real app: return db.is_connected()
    return True


async def check_cache():
    """Simulate async cache connection check"""
    # In real app: return await cache.ping()
    await asyncio.sleep(0.001)
    return True


def check_disk_space():
    """Simulate disk space check (non-critical)"""
    # In real app: check disk space percentage
    return True


# Register health checks
app.health.add_check("database", check_database, critical=True)
app.health.add_check("cache", check_cache, critical=True)
app.health.add_check("disk_space", check_disk_space, critical=False)


# Include standard K8s health routes
app.include_health_routes()

# Or with custom prefix:
# app.include_health_routes(prefix="/api")


# Example routes
@app.get("/")
async def root():
    """Root endpoint"""
    return {"message": "Hello World"}


@app.get("/users/{user_id}")
async def get_user(user_id: str):
    """Get user by ID"""
    return {"user_id": user_id, "name": "John Doe"}


if __name__ == "__main__":
    print("Health endpoints available at:")
    print("  GET /health - Overall health status")
    print("  GET /live   - Liveness probe (K8s)")
    print("  GET /ready  - Readiness probe (K8s)")
    print("\nCustom health checks registered:")
    print("  - database (critical)")
    print("  - cache (critical)")
    print("  - disk_space (non-critical)")
    print("\nExample responses:")
    print("\n  /health:")
    print('  {"status": "healthy", "checks": {"database": true, "cache": true, "disk_space": true}}')
    print("\n  /live:")
    print('  {"status": "alive"}')
    print("\n  /ready:")
    print('  {"status": "ready"}')

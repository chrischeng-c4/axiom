"""Tests for API dependency injection system."""

import pytest
from cclab.probe import expect
from typing import Annotated
from cclab.quasar.dependencies import (
    Depends, Scope, DependencyContainer, RequestContext,
    extract_dependencies, build_dependency_graph
)


class TestDepends:
    """Test Depends marker."""

    def test_basic_depends(self):
        def get_db():
            return "db"

        dep = Depends(get_db)
        assert dep.dependency == get_db
        assert dep.use_cache is True
        assert dep.scope == Scope.REQUEST

    def test_singleton_scope(self):
        dep = Depends(lambda: "value", scope=Scope.SINGLETON)
        assert dep.scope == Scope.SINGLETON

    def test_string_scope(self):
        dep = Depends(lambda: "value", scope="transient")
        assert dep.scope == Scope.TRANSIENT

    def test_transient_scope(self):
        dep = Depends(lambda: "value", scope=Scope.TRANSIENT)
        assert dep.scope == Scope.TRANSIENT

    def test_use_cache_false(self):
        dep = Depends(lambda: "value", use_cache=False)
        assert dep.use_cache is False

    def test_repr(self):
        def my_factory():
            return "value"

        dep = Depends(my_factory, scope=Scope.SINGLETON)
        repr_str = repr(dep)
        assert "my_factory" in repr_str
        assert "singleton" in repr_str


class TestDependencyContainer:
    """Test DependencyContainer."""

    def test_register_and_compile(self):
        container = DependencyContainer()
        container.register("db", lambda: "database")
        container.compile()
        assert container._compiled

    def test_cannot_register_after_compile(self):
        container = DependencyContainer()
        container.register("db", lambda: "database")
        container.compile()

        expect(lambda: container.register("cache", lambda: "cache")).to_raise(RuntimeError)

    def test_topological_sort(self):
        container = DependencyContainer()
        container.register("config", lambda: {})
        container.register("db", lambda config: f"db({config})", sub_dependencies=["config"])
        container.register("service", lambda db: f"svc({db})", sub_dependencies=["db"])
        container.compile()

        order = container.get_resolution_order(["service"])
        assert order == ["config", "db", "service"]

    def test_topological_sort_multiple_roots(self):
        container = DependencyContainer()
        container.register("a", lambda: "a")
        container.register("b", lambda: "b")
        container.register("c", lambda a, b: f"c({a},{b})", sub_dependencies=["a", "b"])
        container.compile()

        order = container.get_resolution_order(["c"])
        # Both a and b should come before c
        assert order.index("a") < order.index("c")
        assert order.index("b") < order.index("c")

    def test_cycle_detection(self):
        container = DependencyContainer()
        container.register("a", lambda b: "a", sub_dependencies=["b"])
        container.register("b", lambda a: "b", sub_dependencies=["a"])

        expect(lambda: container.compile()).to_raise(ValueError)

    def test_missing_dependency(self):
        container = DependencyContainer()
        container.register("service", lambda db: "svc", sub_dependencies=["db"])

        expect(lambda: container.compile()).to_raise(ValueError)

    @pytest.mark.asyncio
    async def test_resolve_simple(self):
        container = DependencyContainer()
        container.register("value", lambda: 42)
        container.compile()

        ctx = RequestContext()
        result = await container.resolve("value", ctx)
        assert result == 42

    @pytest.mark.asyncio
    async def test_resolve_with_dependencies(self):
        container = DependencyContainer()
        container.register("a", lambda: 1)
        container.register("b", lambda a: a + 1, sub_dependencies=["a"])
        container.register("c", lambda b: b + 1, sub_dependencies=["b"])
        container.compile()

        ctx = RequestContext()
        result = await container.resolve("c", ctx)
        assert result == 3

    @pytest.mark.asyncio
    async def test_resolve_unknown_dependency(self):
        container = DependencyContainer()
        container.compile()

        ctx = RequestContext()
        expect(lambda: await container.resolve("unknown", ctx)).to_raise(ValueError)

    @pytest.mark.asyncio
    async def test_request_scope_caching(self):
        call_count = 0

        def factory():
            nonlocal call_count
            call_count += 1
            return call_count

        container = DependencyContainer()
        container.register("counter", factory, scope=Scope.REQUEST)
        container.compile()

        ctx = RequestContext()
        result1 = await container.resolve("counter", ctx)
        result2 = await container.resolve("counter", ctx)

        assert result1 == result2 == 1
        assert call_count == 1

    @pytest.mark.asyncio
    async def test_singleton_caching(self):
        call_count = 0

        def factory():
            nonlocal call_count
            call_count += 1
            return call_count

        container = DependencyContainer()
        container.register("singleton", factory, scope=Scope.SINGLETON)
        container.compile()

        ctx1 = RequestContext()
        ctx2 = RequestContext()

        result1 = await container.resolve("singleton", ctx1)
        result2 = await container.resolve("singleton", ctx2)

        assert result1 == result2 == 1
        assert call_count == 1

    @pytest.mark.asyncio
    async def test_transient_no_caching(self):
        call_count = 0

        def factory():
            nonlocal call_count
            call_count += 1
            return call_count

        container = DependencyContainer()
        container.register("transient", factory, scope=Scope.TRANSIENT)
        container.compile()

        ctx = RequestContext()
        result1 = await container.resolve("transient", ctx)
        result2 = await container.resolve("transient", ctx)

        assert result1 == 1
        assert result2 == 2
        assert call_count == 2

    @pytest.mark.asyncio
    async def test_async_factory(self):
        async def async_factory():
            return "async_value"

        container = DependencyContainer()
        container.register("async_dep", async_factory)
        container.compile()

        ctx = RequestContext()
        result = await container.resolve("async_dep", ctx)
        assert result == "async_value"

    @pytest.mark.asyncio
    async def test_generator_factory(self):
        cleanup_called = False

        def generator_factory():
            value = "gen_value"
            yield value
            nonlocal cleanup_called
            cleanup_called = True

        container = DependencyContainer()
        container.register("gen_dep", generator_factory)
        container.compile()

        ctx = RequestContext()
        result = await container.resolve("gen_dep", ctx)
        assert result == "gen_value"
        assert not cleanup_called

        await ctx.cleanup()
        assert cleanup_called

    @pytest.mark.asyncio
    async def test_async_generator_factory(self):
        cleanup_called = False

        async def async_generator_factory():
            value = "async_gen_value"
            yield value
            nonlocal cleanup_called
            cleanup_called = True

        container = DependencyContainer()
        container.register("async_gen_dep", async_generator_factory)
        container.compile()

        ctx = RequestContext()
        result = await container.resolve("async_gen_dep", ctx)
        assert result == "async_gen_value"
        assert not cleanup_called

        await ctx.cleanup()
        assert cleanup_called

    @pytest.mark.asyncio
    async def test_resolve_all(self):
        container = DependencyContainer()
        container.register("a", lambda: 1)
        container.register("b", lambda: 2)
        container.register("c", lambda a, b: a + b, sub_dependencies=["a", "b"])
        container.compile()

        result = await container.resolve_all(["a", "b", "c"])
        assert result == {"a": 1, "b": 2, "c": 3}

    @pytest.mark.asyncio
    async def test_resolve_all_with_context(self):
        call_count = 0

        def factory():
            nonlocal call_count
            call_count += 1
            return call_count

        container = DependencyContainer()
        container.register("counter", factory, scope=Scope.REQUEST)
        container.compile()

        ctx = RequestContext()
        result1 = await container.resolve_all(["counter"], ctx)
        result2 = await container.resolve_all(["counter"], ctx)

        assert result1 == result2 == {"counter": 1}
        assert call_count == 1

    @pytest.mark.asyncio
    async def test_get_resolution_order_not_compiled(self):
        container = DependencyContainer()
        container.register("a", lambda: 1)

        expect(lambda: container.get_resolution_order(["a"])).to_raise(RuntimeError)


class TestRequestContext:
    """Test RequestContext."""

    def test_get_set(self):
        ctx = RequestContext()
        ctx.set("key", "value")
        assert ctx.get("key") == "value"

    def test_get_missing(self):
        ctx = RequestContext()
        assert ctx.get("missing") is None

    @pytest.mark.asyncio
    async def test_cleanup(self):
        ctx = RequestContext()
        ctx.set("key", "value")

        cleanup1_called = False
        cleanup2_called = False

        def gen1():
            yield "value1"
            nonlocal cleanup1_called
            cleanup1_called = True

        async def gen2():
            yield "value2"
            nonlocal cleanup2_called
            cleanup2_called = True

        g1 = gen1()
        next(g1)
        ctx.add_generator(g1)

        g2 = gen2()
        await g2.asend(None)
        ctx.add_async_generator(g2)

        await ctx.cleanup()

        assert cleanup1_called
        assert cleanup2_called
        assert ctx.get("key") is None


class TestExtractDependencies:
    """Test extract_dependencies function."""

    def test_extract_annotated_depends(self):
        def get_db():
            return "db"

        async def handler(db: Annotated[str, Depends(get_db)]):
            pass

        deps = extract_dependencies(handler)
        assert "db" in deps
        assert deps["db"].dependency == get_db

    def test_extract_multiple_dependencies(self):
        def get_db():
            return "db"

        def get_cache():
            return "cache"

        async def handler(
            db: Annotated[str, Depends(get_db)],
            cache: Annotated[str, Depends(get_cache)],
        ):
            pass

        deps = extract_dependencies(handler)
        assert len(deps) == 2
        assert "db" in deps
        assert "cache" in deps

    def test_extract_no_dependencies(self):
        async def handler(name: str, age: int):
            pass

        deps = extract_dependencies(handler)
        assert len(deps) == 0

    def test_extract_skips_self_and_cls(self):
        def get_db():
            return "db"

        class MyClass:
            def method(self, db: Annotated[str, Depends(get_db)]):
                pass

            @classmethod
            def classmethod(cls, db: Annotated[str, Depends(get_db)]):
                pass

        deps1 = extract_dependencies(MyClass.method)
        assert "self" not in deps1
        assert "db" in deps1

        deps2 = extract_dependencies(MyClass.classmethod)
        assert "cls" not in deps2
        assert "db" in deps2

    def test_extract_with_scope(self):
        def get_db():
            return "db"

        async def handler(
            db: Annotated[str, Depends(get_db, scope=Scope.SINGLETON)]
        ):
            pass

        deps = extract_dependencies(handler)
        assert deps["db"].scope == Scope.SINGLETON


class TestBuildDependencyGraph:
    """Test build_dependency_graph function."""

    def test_build_simple_graph(self):
        def get_config():
            return {}

        async def handler(config: Annotated[dict, Depends(get_config)]):
            pass

        container = DependencyContainer()
        deps = build_dependency_graph(handler, container)

        assert "config" in deps
        assert "config" in container._nodes

    def test_build_nested_graph(self):
        def get_config():
            return {}

        def get_db(config: Annotated[dict, Depends(get_config)]):
            return "db"

        async def handler(db: Annotated[str, Depends(get_db)]):
            pass

        container = DependencyContainer()
        deps = build_dependency_graph(handler, container)

        assert "db" in deps
        assert "db" in container._nodes
        # Note: sub-dependencies are now shared globally (no prefix)
        assert "config" in container._nodes
        # Verify the dependency relationship
        assert "config" in container._nodes["db"].sub_dependencies

    def test_build_no_dependencies(self):
        async def handler(name: str):
            pass

        container = DependencyContainer()
        deps = build_dependency_graph(handler, container)

        assert len(deps) == 0
        assert len(container._nodes) == 0

    def test_build_with_prefix(self):
        def get_config():
            return {}

        async def handler(config: Annotated[dict, Depends(get_config)]):
            pass

        container = DependencyContainer()
        deps = build_dependency_graph(handler, container, prefix="test.")

        assert "test.config" in deps
        assert "test.config" in container._nodes

    def test_build_multiple_dependencies(self):
        def get_config():
            return {}

        def get_cache():
            return "cache"

        async def handler(
            config: Annotated[dict, Depends(get_config)],
            cache: Annotated[str, Depends(get_cache)],
        ):
            pass

        container = DependencyContainer()
        deps = build_dependency_graph(handler, container)

        assert len(deps) == 2
        assert "config" in deps
        assert "cache" in deps


class TestIntegration:
    """Integration tests."""

    @pytest.mark.asyncio
    async def test_full_dependency_chain(self):
        # Define dependencies
        def get_config():
            return {"db_url": "mongodb://localhost"}

        def get_db(config: Annotated[dict, Depends(get_config)]):
            return f"Database({config['db_url']})"

        async def get_service(db: Annotated[str, Depends(get_db)]):
            return f"Service({db})"

        async def handler(
            service: Annotated[str, Depends(get_service)]
        ):
            return f"Handler({service})"

        # Build container
        container = DependencyContainer()
        build_dependency_graph(handler, container)
        container.compile()

        # Resolve
        ctx = RequestContext()
        deps = await container.resolve_all(["service"], ctx)

        assert "service" in deps
        assert "Database(mongodb://localhost)" in deps["service"]

    @pytest.mark.asyncio
    async def test_shared_dependency(self):
        call_count = 0

        def get_config():
            nonlocal call_count
            call_count += 1
            return {"count": call_count}

        def get_db(config: Annotated[dict, Depends(get_config)]):
            return f"db{config['count']}"

        def get_cache(config: Annotated[dict, Depends(get_config)]):
            return f"cache{config['count']}"

        async def handler(
            db: Annotated[str, Depends(get_db)],
            cache: Annotated[str, Depends(get_cache)],
        ):
            return (db, cache)

        container = DependencyContainer()
        build_dependency_graph(handler, container)
        container.compile()

        ctx = RequestContext()
        deps = await container.resolve_all(["db", "cache"], ctx)

        # Config should be called only once (cached)
        assert deps["db"] == "db1"
        assert deps["cache"] == "cache1"
        assert call_count == 1

    @pytest.mark.asyncio
    async def test_context_manager_cleanup(self):
        cleanup_order = []

        def get_resource1():
            cleanup_order.append("resource1_start")
            yield "resource1"
            cleanup_order.append("resource1_end")

        def get_resource2():
            cleanup_order.append("resource2_start")
            yield "resource2"
            cleanup_order.append("resource2_end")

        async def handler(
            r1: Annotated[str, Depends(get_resource1)],
            r2: Annotated[str, Depends(get_resource2)],
        ):
            return (r1, r2)

        container = DependencyContainer()
        build_dependency_graph(handler, container)
        container.compile()

        ctx = RequestContext()
        deps = await container.resolve_all(["r1", "r2"], ctx)

        assert deps == {"r1": "resource1", "r2": "resource2"}
        assert "resource1_start" in cleanup_order
        assert "resource2_start" in cleanup_order
        assert "resource1_end" not in cleanup_order
        assert "resource2_end" not in cleanup_order

        await ctx.cleanup()

        # Cleanup should be in reverse order
        assert cleanup_order[-2:] == ["resource2_end", "resource1_end"]

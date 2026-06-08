"""
Test decorators for cclab.qc

Provides decorator-based test syntax:
- @test: Standard unit tests
- @profile: Performance profiling tests
- @stress: Load/stress tests
- @security: Security/fuzzing tests
"""

from __future__ import annotations

import functools
import inspect
from typing import Any, Callable, List, Optional, TypeVar, Union

# Import from Rust bindings (with fallback for missing extension)
try:
    from .. import cclab as _rust_module
    _test = _rust_module.qc
except (ImportError, AttributeError):
    _test = None

TestType = _test.TestType if _test else None
TestMeta = _test.TestMeta if _test else None

F = TypeVar("F", bound=Callable[..., Any])


class TestDescriptor:
    """Descriptor that holds test metadata"""

    def __init__(
        self,
        func: Callable[..., Any],
        test_type: TestType,
        timeout: Optional[float] = None,
        tags: Optional[List[str]] = None,
        skip: Optional[str] = None,
        # Profile-specific
        iterations: Optional[int] = None,
        warmup: Optional[int] = None,
        measure: Optional[List[str]] = None,
        # Stress-specific
        concurrent_users: Optional[int] = None,
        duration: Optional[float] = None,
        target_rps: Optional[float] = None,
        # Security-specific
        fuzz_inputs: bool = False,
        injection_types: Optional[List[str]] = None,
    ):
        self.func = func
        self.test_type = test_type
        self.timeout = timeout
        self.tags = tags or []
        self.skip = skip

        # Profile config
        self.iterations = iterations
        self.warmup = warmup
        self.measure = measure or ["cpu", "memory"]

        # Stress config
        self.concurrent_users = concurrent_users
        self.duration = duration
        self.target_rps = target_rps

        # Security config
        self.fuzz_inputs = fuzz_inputs
        self.injection_types = injection_types or []

        # Preserve function metadata
        functools.update_wrapper(self, func)

    def __get__(self, obj: Any, objtype: Any = None) -> Callable[..., Any]:
        if obj is None:
            return self
        # Return bound method
        return functools.partial(self.__call__, obj)

    def __call__(self, *args: Any, **kwargs: Any) -> Any:
        return self.func(*args, **kwargs)

    def get_meta(self) -> TestMeta:
        """Get TestMeta for this test"""
        meta = TestMeta(
            name=self.func.__name__,
            test_type=self.test_type,
            timeout=self.timeout,
            tags=self.tags,
        )

        # Set full name from module and qualname
        module = getattr(self.func, "__module__", "")
        qualname = getattr(self.func, "__qualname__", self.func.__name__)
        meta.full_name = f"{module}.{qualname}" if module else qualname

        # Set source location
        try:
            source_file = inspect.getfile(self.func)
            source_lines = inspect.getsourcelines(self.func)
            meta.set_file_path(source_file)
            meta.set_line_number(source_lines[1])
        except (TypeError, OSError):
            pass

        # Set skip reason if skipped
        if self.skip:
            meta.skip(self.skip)

        return meta

    @property
    def is_async(self) -> bool:
        """Check if the test function is async"""
        return inspect.iscoroutinefunction(self.func)


def test(
    func: Optional[F] = None,
    *,
    timeout: Optional[float] = None,
    tags: Optional[List[str]] = None,
    skip: Optional[str] = None,
) -> Union[F, Callable[[F], TestDescriptor]]:
    """
    Decorator for unit tests.

    Args:
        timeout: Test timeout in seconds (default: None = no timeout)
        tags: List of tags for filtering tests
        skip: Skip reason (if provided, test will be skipped)

    Example:
        @test(timeout=5.0, tags=["unit", "fast"])
        async def test_example(self):
            expect(1 + 1).to_equal(2)
    """

    def decorator(f: F) -> TestDescriptor:
        return TestDescriptor(
            func=f,
            test_type=TestType.Unit,
            timeout=timeout,
            tags=tags,
            skip=skip,
        )

    if func is not None:
        return decorator(func)
    return decorator


def profile(
    func: Optional[F] = None,
    *,
    iterations: int = 100,
    warmup: int = 10,
    measure: Optional[List[str]] = None,
    timeout: Optional[float] = None,
    tags: Optional[List[str]] = None,
    skip: Optional[str] = None,
) -> Union[F, Callable[[F], TestDescriptor]]:
    """
    Decorator for performance profiling tests.

    Args:
        iterations: Number of iterations to run (default: 100)
        warmup: Number of warmup iterations (default: 10)
        measure: What to measure - ["cpu", "memory", "rust_boundary"] (default: ["cpu", "memory"])
        timeout: Test timeout in seconds
        tags: List of tags for filtering tests
        skip: Skip reason

    Example:
        @profile(iterations=100, warmup=10, measure=["cpu", "memory", "rust_boundary"])
        async def serialization_performance(self):
            return await self.client.post("/users", json={"name": "Test"})
    """

    def decorator(f: F) -> TestDescriptor:
        return TestDescriptor(
            func=f,
            test_type=TestType.Profile,
            timeout=timeout,
            tags=tags,
            skip=skip,
            iterations=iterations,
            warmup=warmup,
            measure=measure,
        )

    if func is not None:
        return decorator(func)
    return decorator


def stress(
    func: Optional[F] = None,
    *,
    concurrent_users: int = 10,
    duration: float = 10.0,
    target_rps: Optional[float] = None,
    timeout: Optional[float] = None,
    tags: Optional[List[str]] = None,
    skip: Optional[str] = None,
) -> Union[F, Callable[[F], TestDescriptor]]:
    """
    Decorator for load/stress tests.

    Args:
        concurrent_users: Number of concurrent virtual users (default: 10)
        duration: Test duration in seconds (default: 10.0)
        target_rps: Target requests per second (default: None = unlimited)
        timeout: Test timeout in seconds
        tags: List of tags for filtering tests
        skip: Skip reason

    Example:
        @stress(concurrent_users=100, duration=60.0, target_rps=500)
        async def high_load_test(self):
            response = await self.client.get("/health")
            return response.status_code == 200
    """

    def decorator(f: F) -> TestDescriptor:
        return TestDescriptor(
            func=f,
            test_type=TestType.Stress,
            timeout=timeout,
            tags=tags,
            skip=skip,
            concurrent_users=concurrent_users,
            duration=duration,
            target_rps=target_rps,
        )

    if func is not None:
        return decorator(func)
    return decorator


def security(
    func: Optional[F] = None,
    *,
    fuzz_inputs: bool = True,
    injection_types: Optional[List[str]] = None,
    timeout: Optional[float] = None,
    tags: Optional[List[str]] = None,
    skip: Optional[str] = None,
) -> Union[F, Callable[[F], TestDescriptor]]:
    """
    Decorator for security/fuzzing tests.

    Args:
        fuzz_inputs: Whether to fuzz input parameters (default: True)
        injection_types: Types of injection to test - ["sql", "nosql", "xss", "cmd"] (default: all)
        timeout: Test timeout in seconds
        tags: List of tags for filtering tests
        skip: Skip reason

    Example:
        @security(fuzz_inputs=True, injection_types=["sql", "nosql", "xss"])
        async def input_sanitization(self, payload: str):
            response = await self.client.post("/search", json={"q": payload})
            expect(response.status_code).to_not_equal(500)
    """
    if injection_types is None:
        injection_types = ["sql", "nosql", "xss", "cmd"]

    def decorator(f: F) -> TestDescriptor:
        return TestDescriptor(
            func=f,
            test_type=TestType.Security,
            timeout=timeout,
            tags=tags,
            skip=skip,
            fuzz_inputs=fuzz_inputs,
            injection_types=injection_types,
        )

    if func is not None:
        return decorator(func)
    return decorator


def fixture(
    func: Optional[F] = None,
    *,
    scope: str = "function",
    autouse: bool = False,
) -> Union[F, Callable[[F], F]]:
    """
    Decorator to mark a method as a fixture (pytest-compatible API).

    Fixtures provide reusable test resources with setup/teardown lifecycle.
    They can depend on other fixtures and are cached based on their scope.

    Args:
        scope: Fixture scope - "function", "class", "module", or "session" (default: "function")
        autouse: Whether fixture is automatically used for all tests (default: False)

    Example:
        from cclab.qc import TestSuite, test, fixture, expect

        class DatabaseTests(TestSuite):
            @fixture(scope="class")
            async def db_connection(self):
                '''Class-scoped fixture with setup/teardown.'''
                conn = await create_connection()
                yield conn  # pytest-compatible yield syntax
                await conn.close()

            @fixture(scope="function")
            async def test_user(self, db_connection):
                '''Fixture depending on another fixture.'''
                user = await db_connection.create_user("test@example.com")
                yield user
                await db_connection.delete_user(user.id)

            @test
            async def test_query(self, db_connection, test_user):
                '''Fixtures auto-injected via parameter names.'''
                result = await db_connection.query("SELECT * FROM users WHERE id = ?", test_user.id)
                expect(result).to_not_be_none()
    """

    def decorator(f: F) -> F:
        # Store fixture metadata on the function
        f._fixture_meta = {  # type: ignore
            "scope": scope,
            "autouse": autouse,
            "name": f.__name__,
        }
        return f

    if func is not None:
        return decorator(func)
    return decorator


def parametrize(name: str, values: List[Any]) -> Callable[[F], F]:
    """
    Decorator to parametrize a test function with multiple values.

    Similar to pytest.mark.parametrize, this decorator generates multiple test instances
    from a single test function, each with different parameter values.

    Single parameter:
        @parametrize("batch_size", [10, 100, 1000])
        - Generates 3 test instances

    Multiple parameters (Cartesian product):
        @parametrize("method", ["GET", "POST"])
        @parametrize("auth", [True, False])
        - Generates 4 test instances (2 × 2)

    Args:
        name: Parameter name (must match test function argument)
        values: List of values for this parameter

    Example:
        from cclab.qc import TestSuite, test, parametrize, expect

        class BulkTests(TestSuite):
            @test
            @parametrize("batch_size", [10, 100, 1000, 10000, 50000])
            async def test_bulk_insert(self, batch_size):
                '''Runs 5 times with different batch_size values.'''
                docs = [{"value": i} for i in range(batch_size)]
                result = await collection.insert_many(docs)
                expect(len(result.inserted_ids)).to_equal(batch_size)

            @test
            @parametrize("method", ["GET", "POST", "PUT", "DELETE"])
            @parametrize("auth", [True, False])
            async def test_http_methods(self, method, auth):
                '''Generates 8 test cases (4 × 2 cartesian product).'''
                response = await client.request(method, "/api/test", auth=auth)
                expect(response.status).to_equal(200)
    """

    def decorator(func: F) -> F:
        # Store parametrize metadata on the function
        if not hasattr(func, "_parametrize"):
            func._parametrize = []  # type: ignore
        func._parametrize.append((name, values))  # type: ignore
        return func

    return decorator

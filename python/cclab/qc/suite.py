"""
TestSuite base class for cclab.qc

Provides a base class for organizing tests into suites.
"""

from __future__ import annotations

import asyncio
import inspect
import time
import traceback
from pathlib import Path
from typing import Any, Callable, Dict, Generator, List, Optional, Tuple, Type, Union

# Import from Rust bindings helper (with fallback to stubs)
from ._rust_bindings import qc as _test

TestRunner = _test.TestRunner
TestResult = _test.TestResult
TestReport = _test.TestReport
Reporter = _test.Reporter
ReportFormat = _test.ReportFormat
FileCoverage = _test.FileCoverage
CoverageInfo = _test.CoverageInfo
ParameterValue = _test.ParameterValue

from .decorators import TestDescriptor
Parameter = _test.Parameter
ParametrizedTest = _test.ParametrizedTest
HookType = _test.HookType
HookRegistry = _test.HookRegistry
FixtureRegistry = _test.FixtureRegistry
FixtureScope = _test.FixtureScope


class FixtureRunner:
    """
    Manages fixture lifecycle including setup, caching, and teardown.

    Supports:
    - Scope-based caching (function, class, module, session)
    - Yield-based setup/teardown (sync and async)
    - Dependency resolution and injection
    """

    def __init__(self, suite_instance: Any, registry: FixtureRegistry):
        self.suite = suite_instance
        self.registry = registry
        # Fixture functions by name
        self._fixtures: Dict[str, Callable] = {}
        # Track if fixture is a method (needs self) or module-level function
        self._fixture_is_method: Dict[str, bool] = {}
        # Pending module fixtures (deferred registration for dependency resolution)
        self._pending_module_fixtures: Dict[str, Tuple[Callable, Dict[str, Any]]] = {}
        # Cached fixture values by (scope, name)
        self._cache: Dict[Tuple[str, str], Any] = {}
        # Active generators (for teardown) by (scope, name)
        self._generators: Dict[Tuple[str, str], Union[Generator, Any]] = {}
        # Current scope context
        self._current_class: Optional[str] = None
        self._current_module: Optional[str] = None

    def register_fixture(self, name: str, func: Callable, meta: Dict[str, Any]) -> None:
        """Register a class-level fixture function (method with self)."""
        self._fixtures[name] = func
        self._fixture_is_method[name] = True

        # Parse dependencies from function signature
        sig = inspect.signature(func)
        deps = [
            p for p in sig.parameters.keys()
            if p != 'self' and self.registry.has_fixture(p)
        ]

        # Check if function is a generator (has yield)
        has_teardown = (
            inspect.isgeneratorfunction(func) or
            inspect.isasyncgenfunction(func)
        )

        # Register with Rust registry
        scope_str = meta.get("scope", "function")
        scope = FixtureScope.from_string(scope_str)
        self.registry.register(
            name,
            scope,
            meta.get("autouse", False),
            deps,
            has_teardown,
        )

    def register_module_fixture(self, name: str, func: Callable, meta: Dict[str, Any]) -> None:
        """Register a module-level fixture function (no self parameter)."""
        # Skip if already registered (class fixtures take precedence)
        if name in self._fixtures:
            return

        self._fixtures[name] = func
        self._fixture_is_method[name] = False  # Module-level, no self
        # Store meta for deferred dependency resolution
        self._pending_module_fixtures[name] = (func, meta)

    def finalize_fixture_registration(self) -> None:
        """Finalize registration after all fixtures are discovered (resolve dependencies)."""
        # Process pending module fixtures now that all fixtures are known
        for name, (func, meta) in self._pending_module_fixtures.items():
            # Parse dependencies from function signature
            sig = inspect.signature(func)
            deps = [
                p for p in sig.parameters.keys()
                if p in self._fixtures  # Check against all discovered fixtures
            ]

            # Check if function is a generator (has yield)
            has_teardown = (
                inspect.isgeneratorfunction(func) or
                inspect.isasyncgenfunction(func)
            )

            # Register with Rust registry
            scope_str = meta.get("scope", "function")
            scope = FixtureScope.from_string(scope_str)
            self.registry.register(
                name,
                scope,
                meta.get("autouse", False),
                deps,
                has_teardown,
            )
        self._pending_module_fixtures.clear()

    async def get_fixture_value(self, name: str) -> Any:
        """
        Get fixture value, setting up if needed.

        Returns cached value if available, otherwise runs fixture setup.
        """
        meta = self.registry.get_meta(name)
        if meta is None:
            raise ValueError(f"Unknown fixture: {name}")

        scope_key = self._get_scope_key(meta.scope)
        cache_key = (scope_key, name)

        # Return cached value if available
        if cache_key in self._cache:
            return self._cache[cache_key]

        # Resolve dependencies first (in topological order)
        deps = meta.dependencies
        dep_values = {}
        if deps:
            resolved_order = self.registry.resolve_order(deps)
            for dep_name in resolved_order:
                dep_values[dep_name] = await self.get_fixture_value(dep_name)

        # Get the fixture function
        func = self._fixtures.get(name)
        if func is None:
            raise ValueError(f"Fixture function not found: {name}")

        # Build kwargs for fixture call
        sig = inspect.signature(func)
        kwargs = {}
        for param_name in sig.parameters.keys():
            if param_name == 'self':
                continue
            if param_name in dep_values:
                kwargs[param_name] = dep_values[param_name]

        # Execute fixture setup
        value = await self._execute_fixture(name, func, kwargs, cache_key)
        return value

    async def _execute_fixture(
        self,
        name: str,
        func: Callable,
        kwargs: Dict[str, Any],
        cache_key: Tuple[str, str],
    ) -> Any:
        """Execute fixture function and handle yield for teardown."""
        # Check if fixture is a method (needs self) or module-level function
        is_method = self._fixture_is_method.get(name, True)

        if inspect.isasyncgenfunction(func):
            # Async generator fixture
            if is_method:
                gen = func(self.suite, **kwargs)
            else:
                gen = func(**kwargs)
            value = await gen.__anext__()
            self._generators[cache_key] = gen
        elif inspect.isgeneratorfunction(func):
            # Sync generator fixture
            if is_method:
                gen = func(self.suite, **kwargs)
            else:
                gen = func(**kwargs)
            value = next(gen)
            self._generators[cache_key] = gen
        elif inspect.iscoroutinefunction(func):
            # Async function (no teardown)
            if is_method:
                value = await func(self.suite, **kwargs)
            else:
                value = await func(**kwargs)
        else:
            # Sync function (no teardown)
            if is_method:
                value = func(self.suite, **kwargs)
            else:
                value = func(**kwargs)

        self._cache[cache_key] = value
        return value

    async def teardown_scope(self, scope: str) -> None:
        """
        Teardown all fixtures for a given scope.

        Teardown runs in reverse dependency order.
        """
        scope_key = self._get_scope_key_for_string(scope)

        # Find all cache keys matching this scope
        keys_to_teardown = [
            key for key in self._cache.keys()
            if key[0] == scope_key
        ]

        # Get fixture names and resolve reverse order
        fixture_names = [key[1] for key in keys_to_teardown]
        if fixture_names:
            try:
                ordered = self.registry.resolve_order(fixture_names)
                # Reverse for teardown
                ordered = list(reversed(ordered))
            except Exception:
                # If resolution fails, use original order reversed
                ordered = list(reversed(fixture_names))
        else:
            ordered = []

        # Teardown each fixture
        for name in ordered:
            cache_key = (scope_key, name)
            await self._teardown_fixture(cache_key)

    async def _teardown_fixture(self, cache_key: Tuple[str, str]) -> None:
        """Run teardown for a single fixture."""
        if cache_key not in self._generators:
            # No teardown needed (not a generator fixture)
            self._cache.pop(cache_key, None)
            return

        gen = self._generators.pop(cache_key)
        self._cache.pop(cache_key, None)

        try:
            if inspect.isasyncgen(gen):
                # Async generator - continue to teardown
                try:
                    await gen.__anext__()
                except StopAsyncIteration:
                    pass
            else:
                # Sync generator
                try:
                    next(gen)
                except StopIteration:
                    pass
        except Exception as e:
            # Log teardown error but don't propagate
            # Teardown should always run to completion
            pass

    async def teardown_all(self) -> None:
        """Teardown all active fixtures in reverse order."""
        # Teardown in order: function -> class -> module -> session
        for scope in ["function", "class", "module", "session"]:
            await self.teardown_scope(scope)

    def _get_scope_key(self, scope: Any) -> str:
        """Get cache key prefix for a scope."""
        scope_str = str(scope)
        return self._get_scope_key_for_string(scope_str)

    def _get_scope_key_for_string(self, scope: str) -> str:
        """Get cache key prefix for a scope string."""
        if scope == "function":
            return "function"
        elif scope == "class":
            return f"class:{self._current_class or 'default'}"
        elif scope == "module":
            return f"module:{self._current_module or 'default'}"
        elif scope == "session":
            return "session"
        return scope

    def set_class_context(self, class_name: Optional[str]) -> None:
        """Set current class context for class-scoped fixtures."""
        self._current_class = class_name

    def set_module_context(self, module_name: Optional[str]) -> None:
        """Set current module context for module-scoped fixtures."""
        self._current_module = module_name

    async def inject_fixtures(self, func: Callable) -> Dict[str, Any]:
        """
        Resolve and inject fixtures based on function signature.

        Returns a dict of fixture name -> value for all fixture parameters.
        """
        sig = inspect.signature(func)
        kwargs = {}

        for param_name in sig.parameters.keys():
            if param_name == 'self':
                continue
            if self.registry.has_fixture(param_name):
                kwargs[param_name] = await self.get_fixture_value(param_name)

        return kwargs


class ParametrizedTestInstance:
    """Wrapper for a single instance of a parametrized test"""

    def __init__(self, test_desc: TestDescriptor, param_set: Any, instance_name: str):
        self.test_desc = test_desc
        self.param_set = param_set
        self.instance_name = instance_name

    def get_meta(self):
        """Get TestMeta with parametrized name"""
        # Get the base meta from the test descriptor
        base_meta = self.test_desc.get_meta()

        # Import TestMeta from Rust bindings helper
        from ._rust_bindings import qc
        TestMeta = qc.TestMeta

        # Create a new TestMeta with the parametrized name
        meta = TestMeta(
            name=self.instance_name,
            test_type=base_meta.test_type,
            timeout=base_meta.timeout,
            tags=base_meta.tags,
        )

        # Update full_name to include parameters
        base_full_name = base_meta.full_name
        if '.' in base_full_name:
            parts = base_full_name.rsplit('.', 1)
            meta.full_name = f"{parts[0]}.{self.instance_name}"
        else:
            meta.full_name = self.instance_name

        # Copy skip reason if present
        if base_meta.is_skipped():
            meta.skip(base_meta.skip_reason or "Skipped")

        return meta

    @property
    def is_async(self) -> bool:
        return self.test_desc.is_async

    def __call__(self, suite_instance: Any, fixture_kwargs: Optional[Dict[str, Any]] = None) -> Any:
        """Execute the test with parameter and fixture injection"""
        # Get the test function signature to extract parameter names
        sig = inspect.signature(self.test_desc.func)
        param_names = [p for p in sig.parameters.keys() if p != 'self']

        # Build kwargs from ParameterSet by converting to dict first
        param_dict = self.param_set.to_dict()

        # Extract values for the specific parameters needed by this test
        kwargs = {}
        for param_name in param_names:
            if param_name in param_dict:
                kwargs[param_name] = param_dict[param_name]

        # Merge fixture kwargs (fixtures take precedence over parametrize for same name)
        if fixture_kwargs:
            for k, v in fixture_kwargs.items():
                if k not in kwargs:
                    kwargs[k] = v

        # Call the original test function with injected parameters
        # Return whatever the function returns (coroutine for async, value for sync)
        return self.test_desc.func(suite_instance, **kwargs)


class TestSuite:
    """
    Base class for test suites.

    Subclass this to create a test suite with setup/teardown hooks
    and test discovery.

    Example:
        from cclab.qc import TestSuite, test, expect
        from cclab.http import HttpClient

        class UserAPITests(TestSuite):
            async def setup_suite(self):
                self.client = HttpClient(base_url="http://localhost:8000")

            async def teardown_suite(self):
                pass  # cleanup

            async def setup(self):
                pass  # before each test

            async def teardown(self):
                pass  # after each test

            @test(timeout=5.0, tags=["unit"])
            async def login_returns_token(self):
                response = await self.client.post("/auth/login", json={
                    "email": "test@example.com",
                    "password": "secret"
                })
                expect(response.status_code).to_equal(200)
    """

    def __init__(self) -> None:
        self._tests: List[TestDescriptor] = []
        self._hook_registry: HookRegistry = HookRegistry()
        self._fixture_registry: FixtureRegistry = FixtureRegistry()
        self._fixture_runner: Optional[FixtureRunner] = None
        self._discover_fixtures()
        self._discover_tests()
        self._discover_hooks()

    def _discover_fixtures(self) -> None:
        """Discover all @fixture decorated methods in this suite, module, and parent modules."""
        self._fixture_runner = FixtureRunner(self, self._fixture_registry)

        import sys

        # Collect all modules to check (current + all parent class modules)
        modules_to_check = set()

        # Current class module
        module_name = self.__class__.__module__
        if module_name in sys.modules:
            modules_to_check.add(sys.modules[module_name])

        # Parent class modules (for inherited fixtures)
        for base_class in self.__class__.__mro__:
            if base_class is object:
                continue
            base_module_name = base_class.__module__
            if base_module_name in sys.modules:
                modules_to_check.add(sys.modules[base_module_name])

        # 1. Discover module-level fixtures from all relevant modules
        for module in modules_to_check:
            for name in dir(module):
                attr = getattr(module, name, None)
                if attr is None:
                    continue
                # Check for _fixture_meta attribute (set by @fixture decorator)
                fixture_meta = getattr(attr, '_fixture_meta', None)
                if fixture_meta is not None:
                    # Module-level fixtures don't have 'self', register as-is
                    self._fixture_runner.register_module_fixture(name, attr, fixture_meta)

        # 2. Discover class-level fixtures (methods)
        for name in dir(self):
            attr = getattr(self.__class__, name, None)
            if attr is None:
                continue

            # Check for _fixture_meta attribute (set by @fixture decorator)
            fixture_meta = getattr(attr, '_fixture_meta', None)
            if fixture_meta is not None:
                self._fixture_runner.register_fixture(name, attr, fixture_meta)

        # 3. Finalize registration (resolve dependencies for module fixtures)
        self._fixture_runner.finalize_fixture_registration()

    def _discover_tests(self) -> None:
        """Discover all test methods in this suite"""
        self._tests = []

        for name in dir(self):
            attr = getattr(self.__class__, name, None)
            if isinstance(attr, TestDescriptor):
                # Check if test has parametrize decorators
                if hasattr(attr.func, '_parametrize'):
                    # Expand parametrized test into multiple instances
                    expanded_tests = self._expand_parametrized_test(attr)
                    self._tests.extend(expanded_tests)
                else:
                    # Regular test (no parametrization)
                    self._tests.append(attr)

    def _discover_hooks(self) -> None:
        """Discover and register lifecycle hooks"""
        # Class-level hooks (run once per test class)
        if hasattr(self.__class__, 'setup_class') and callable(getattr(self.__class__, 'setup_class')):
            setup_class = getattr(self.__class__, 'setup_class')
            # Only register if it's not the base TestSuite method
            if setup_class.__qualname__ != 'TestSuite.setup_class':
                self._hook_registry.register_hook(HookType.SetupClass, setup_class)

        if hasattr(self.__class__, 'teardown_class') and callable(getattr(self.__class__, 'teardown_class')):
            teardown_class = getattr(self.__class__, 'teardown_class')
            if teardown_class.__qualname__ != 'TestSuite.teardown_class':
                self._hook_registry.register_hook(HookType.TeardownClass, teardown_class)

        # Method-level hooks (run before/after each test)
        if hasattr(self.__class__, 'setup_method') and callable(getattr(self.__class__, 'setup_method')):
            setup_method = getattr(self.__class__, 'setup_method')
            if setup_method.__qualname__ != 'TestSuite.setup_method':
                self._hook_registry.register_hook(HookType.SetupMethod, setup_method)

        if hasattr(self.__class__, 'teardown_method') and callable(getattr(self.__class__, 'teardown_method')):
            teardown_method = getattr(self.__class__, 'teardown_method')
            if teardown_method.__qualname__ != 'TestSuite.teardown_method':
                self._hook_registry.register_hook(HookType.TeardownMethod, teardown_method)

    def _expand_parametrized_test(self, test_desc: TestDescriptor) -> List:
        """Expand a parametrized test into multiple test instances"""
        # Get parametrize metadata from the function
        parametrize_data = getattr(test_desc.func, '_parametrize', [])
        if not parametrize_data:
            return [test_desc]

        # Create ParametrizedTest and add parameters
        param_test = ParametrizedTest(test_desc.func.__name__)

        for param_name, param_values in parametrize_data:
            # Convert Python values to ParameterValue objects
            converted_values = []
            for value in param_values:
                # Auto-convert based on Python type
                # IMPORTANT: Check bool BEFORE int since bool is a subclass of int in Python
                if isinstance(value, bool):
                    converted_values.append(ParameterValue.bool(value))
                elif isinstance(value, int):
                    converted_values.append(ParameterValue.int(value))
                elif isinstance(value, float):
                    converted_values.append(ParameterValue.float(value))
                elif isinstance(value, str):
                    converted_values.append(ParameterValue.string(value))
                elif value is None:
                    converted_values.append(ParameterValue.none())
                else:
                    raise TypeError(f"Unsupported parameter type for value {value}: {type(value)}")

            # Create Parameter and add to ParametrizedTest
            param = Parameter(param_name, converted_values)
            param_test.add_parameter(param)

        # Expand into test instances
        expanded = param_test.expand()

        # Create ParametrizedTestInstance wrappers
        instances = []
        for instance_name, param_set in expanded:
            instances.append(ParametrizedTestInstance(test_desc, param_set, instance_name))

        return instances

    @property
    def test_count(self) -> int:
        """Number of tests in this suite"""
        return len(self._tests)

    @property
    def suite_name(self) -> str:
        """Name of this test suite"""
        return self.__class__.__name__

    # Lifecycle hooks (override in subclasses)

    async def setup_suite(self) -> None:
        """Called once before all tests in the suite (legacy, use setup_class)"""
        pass

    async def teardown_suite(self) -> None:
        """Called once after all tests in the suite (legacy, use teardown_class)"""
        pass

    async def setup(self) -> None:
        """Called before each test (legacy, use setup_method)"""
        pass

    async def teardown(self) -> None:
        """Called after each test (legacy, use teardown_method)"""
        pass

    # New hook methods (pytest-compatible)

    async def setup_class(self) -> None:
        """Called once before all tests in the class"""
        pass

    async def teardown_class(self) -> None:
        """Called once after all tests in the class"""
        pass

    async def setup_method(self) -> None:
        """Called before each test method"""
        pass

    async def teardown_method(self) -> None:
        """Called after each test method"""
        pass

    # Test execution

    async def run(
        self,
        runner: Optional[TestRunner] = None,
        verbose: bool = False,
    ) -> TestReport:
        """
        Run all tests in this suite.

        Args:
            runner: Optional test runner with filters. If None, runs all tests.
            verbose: Whether to print verbose output

        Returns:
            TestReport with all results
        """
        if runner is None:
            runner = TestRunner()

        runner.start()

        # Set fixture context for class scope
        if self._fixture_runner is not None:
            self._fixture_runner.set_class_context(self.__class__.__name__)
            self._fixture_runner.set_module_context(self.__class__.__module__)

        # Run setup_class hooks (and legacy setup_suite)
        try:
            await self.setup_suite()  # Legacy support
            # Run setup_class hooks
            error = await self._hook_registry.run_hooks(HookType.SetupClass, self)
            if error:
                raise RuntimeError(error)
        except Exception as e:
            # If setup fails, mark all tests as error
            for test_desc in self._tests:
                meta = test_desc.get_meta()
                result = TestResult.error(meta, 0, f"Class setup failed: {e}")
                result.set_stack_trace(traceback.format_exc())
                runner.record(result)
            return TestReport(self.suite_name, runner.results())

        # Run each test
        for test_desc in self._tests:
            meta = test_desc.get_meta()

            # Check if test should run based on filters
            if not runner.should_run(meta):
                continue

            # Check if skipped
            if meta.is_skipped():
                result = TestResult.skipped(meta, meta.skip_reason or "Skipped")
                runner.record(result)
                if verbose:
                    print(f"  SKIPPED: {meta.name}")
                continue

            # Run setup_method hooks (and legacy setup)
            try:
                await self.setup()  # Legacy support
                # Run setup_method hooks
                error = await self._hook_registry.run_hooks(HookType.SetupMethod, self)
                if error:
                    raise RuntimeError(error)
            except Exception as e:
                result = TestResult.error(meta, 0, f"Method setup failed: {e}")
                result.set_stack_trace(traceback.format_exc())
                runner.record(result)
                if verbose:
                    print(f"  ERROR: {meta.name} (setup failed)")
                continue

            # Run the test with fixture injection
            start_time = time.perf_counter()
            try:
                # Inject fixtures based on test function signature
                fixture_kwargs = {}
                if self._fixture_runner is not None:
                    # Get the underlying function for signature inspection
                    if isinstance(test_desc, ParametrizedTestInstance):
                        func = test_desc.test_desc.func
                    else:
                        func = test_desc.func
                    fixture_kwargs = await self._fixture_runner.inject_fixtures(func)

                if test_desc.is_async:
                    if isinstance(test_desc, ParametrizedTestInstance):
                        # ParametrizedTestInstance handles both param and fixture injection
                        await test_desc(self, fixture_kwargs)
                    else:
                        await test_desc.func(self, **fixture_kwargs)
                else:
                    if isinstance(test_desc, ParametrizedTestInstance):
                        test_desc(self, fixture_kwargs)
                    else:
                        test_desc.func(self, **fixture_kwargs)

                duration_ms = int((time.perf_counter() - start_time) * 1000)
                result = TestResult.passed(meta, duration_ms)

                if verbose:
                    print(f"  PASSED: {meta.name} ({duration_ms}ms)")

            except AssertionError as e:
                duration_ms = int((time.perf_counter() - start_time) * 1000)
                result = TestResult.failed(meta, duration_ms, str(e))
                result.set_stack_trace(traceback.format_exc())

                if verbose:
                    print(f"  FAILED: {meta.name} ({duration_ms}ms)")
                    print(f"    Error: {e}")

            except Exception as e:
                duration_ms = int((time.perf_counter() - start_time) * 1000)
                result = TestResult.error(meta, duration_ms, str(e))
                result.set_stack_trace(traceback.format_exc())

                if verbose:
                    print(f"  ERROR: {meta.name} ({duration_ms}ms)")
                    print(f"    Error: {e}")

            runner.record(result)

            # Run teardown_method hooks (and legacy teardown)
            try:
                # Teardown function-scoped fixtures first
                if self._fixture_runner is not None:
                    await self._fixture_runner.teardown_scope("function")
                # Run teardown_method hooks
                error = await self._hook_registry.run_hooks(HookType.TeardownMethod, self)
                if error and verbose:
                    print(f"  WARNING: Method teardown error for {meta.name}: {error}")
                await self.teardown()  # Legacy support
            except Exception as e:
                # Log teardown error but don't override test result
                if verbose:
                    print(f"  WARNING: Method teardown failed for {meta.name}: {e}")

        # Run teardown_class hooks (and legacy teardown_suite)
        try:
            # Teardown class-scoped fixtures first
            if self._fixture_runner is not None:
                await self._fixture_runner.teardown_scope("class")
            # Run teardown_class hooks
            error = await self._hook_registry.run_hooks(HookType.TeardownClass, self)
            if error and verbose:
                print(f"WARNING: Class teardown error: {error}")
            await self.teardown_suite()  # Legacy support
        except Exception as e:
            if verbose:
                print(f"WARNING: Class teardown failed: {e}")

        # Final cleanup: teardown any remaining fixtures (module/session scope)
        try:
            if self._fixture_runner is not None:
                await self._fixture_runner.teardown_all()
        except Exception as e:
            if verbose:
                print(f"WARNING: Fixture cleanup failed: {e}")

        return TestReport(self.suite_name, runner.results())


def run_suite(
    suite_class: Type[TestSuite],
    output_format: ReportFormat = ReportFormat.Markdown,
    output_file: Optional[str] = None,
    verbose: bool = True,
    **runner_kwargs: Any,
) -> TestReport:
    """
    Convenience function to run a test suite.

    Args:
        suite_class: The TestSuite subclass to run
        output_format: Report output format (default: Markdown)
        output_file: Optional file path to write report
        verbose: Whether to print verbose output
        **runner_kwargs: Additional arguments for TestRunner

    Returns:
        TestReport with all results

    Example:
        from cclab.qc import run_suite, ReportFormat

        report = run_suite(MyTests, output_format=ReportFormat.Html, output_file="report.html")
    """
    suite = suite_class()
    runner = TestRunner(**runner_kwargs)

    if verbose:
        print(f"\nRunning: {suite.suite_name}")
        print("=" * 50)

    report = asyncio.run(suite.run(runner=runner, verbose=verbose))

    if verbose:
        print("=" * 50)
        summary = report.summary
        print(f"Results: {summary.passed}/{summary.total} passed")
        if summary.failed > 0:
            print(f"  Failed: {summary.failed}")
        if summary.errors > 0:
            print(f"  Errors: {summary.errors}")
        if summary.skipped > 0:
            print(f"  Skipped: {summary.skipped}")
        print(f"Duration: {summary.total_duration_ms}ms")

    # Generate and optionally save report
    if output_file:
        reporter = Reporter(output_format)
        report_content = reporter.generate(report)

        with open(output_file, "w") as f:
            f.write(report_content)

        if verbose:
            print(f"\nReport written to: {output_file}")

    return report


def run_suites(
    suite_classes: List[Type[TestSuite]],
    output_format: ReportFormat = ReportFormat.Markdown,
    output_file: Optional[str] = None,
    verbose: bool = True,
    **runner_kwargs: Any,
) -> List[TestReport]:
    """
    Run multiple test suites.

    Args:
        suite_classes: List of TestSuite subclasses to run
        output_format: Report output format
        output_file: Optional file path for combined report
        verbose: Whether to print verbose output
        **runner_kwargs: Additional arguments for TestRunner

    Returns:
        List of TestReports, one per suite
    """
    reports = []

    for suite_class in suite_classes:
        report = run_suite(
            suite_class,
            output_format=output_format,
            verbose=verbose,
            **runner_kwargs,
        )
        reports.append(report)

    # Optionally combine reports into one file
    if output_file and reports:
        reporter = Reporter(output_format)
        combined = "\n\n---\n\n".join(reporter.generate(r) for r in reports)

        with open(output_file, "w") as f:
            f.write(combined)

        if verbose:
            print(f"\nCombined report written to: {output_file}")

    return reports


def _collect_coverage_from_coveragepy(
    source_dirs: List[str],
    omit_patterns: Optional[List[str]] = None,
) -> Optional[CoverageInfo]:
    """
    Collect coverage data from coverage.py.

    Must be called after coverage.stop() and coverage.save().

    Args:
        source_dirs: Directories to collect coverage from
        omit_patterns: Patterns to omit from coverage

    Returns:
        CoverageInfo object or None if coverage module not available
    """
    try:
        import coverage
    except ImportError:
        return None

    # Load existing coverage data
    cov = coverage.Coverage()
    try:
        cov.load()
    except coverage.misc.CoverageException:
        return None

    # Get analysis data
    coverage_info = CoverageInfo()

    for source_dir in source_dirs:
        source_path = Path(source_dir)
        if not source_path.exists():
            continue

        # Find all Python files
        for py_file in source_path.rglob("*.py"):
            # Skip test files and __pycache__
            if "__pycache__" in str(py_file):
                continue
            if omit_patterns:
                skip = False
                for pattern in omit_patterns:
                    if pattern in str(py_file):
                        skip = True
                        break
                if skip:
                    continue

            try:
                analysis = cov.analysis2(str(py_file))
                # analysis returns: (filename, executable, excluded, missing, formatted)
                filename, executable, excluded, missing, _ = analysis

                statements = len(executable)
                covered = statements - len(missing)

                if statements > 0:
                    file_cov = FileCoverage(
                        path=str(py_file.relative_to(source_path.parent)),
                        statements=statements,
                        covered=covered,
                        missing_lines=list(missing),
                    )
                    coverage_info.add_file(file_cov)
            except Exception:
                # File might not have been imported/executed
                coverage_info.add_uncovered_file(str(py_file.relative_to(source_path.parent)))

    return coverage_info


def run_suite_with_coverage(
    suite_class: Type[TestSuite],
    source_dirs: List[str],
    output_format: ReportFormat = ReportFormat.Markdown,
    output_file: Optional[str] = None,
    verbose: bool = True,
    omit_patterns: Optional[List[str]] = None,
    **runner_kwargs: Any,
) -> TestReport:
    """
    Run a test suite with coverage collection.

    Requires coverage.py to be installed.

    Args:
        suite_class: The TestSuite subclass to run
        source_dirs: Directories to measure coverage for
        output_format: Report output format (default: Markdown)
        output_file: Optional file path to write report
        verbose: Whether to print verbose output
        omit_patterns: Patterns to omit from coverage (e.g., ["test_", "__pycache__"])
        **runner_kwargs: Additional arguments for TestRunner

    Returns:
        TestReport with coverage data included

    Example:
        from cclab.qc import run_suite_with_coverage, ReportFormat

        report = run_suite_with_coverage(
            MyTests,
            source_dirs=["python/cclab"],
            output_format=ReportFormat.Html,
            output_file="coverage_report.html"
        )
    """
    try:
        import coverage
    except ImportError:
        raise ImportError("coverage.py is required for coverage collection. Install with: pip install coverage")

    # Start coverage
    cov = coverage.Coverage(
        source=source_dirs,
        omit=omit_patterns or ["*test*", "*__pycache__*"],
    )
    cov.start()

    try:
        # Run the test suite
        report = run_suite(
            suite_class,
            output_format=output_format,
            verbose=verbose,
            **runner_kwargs,
        )
    finally:
        # Stop and save coverage
        cov.stop()
        cov.save()

    # Collect coverage data
    coverage_info = _collect_coverage_from_coveragepy(
        source_dirs,
        omit_patterns=omit_patterns or ["test_", "__pycache__"],
    )

    if coverage_info:
        report.set_coverage(coverage_info)

        if verbose:
            print(f"\nCoverage: {coverage_info.coverage_percent:.1f}% "
                  f"({coverage_info.covered_statements}/{coverage_info.total_statements} statements)")

    # Generate and optionally save report
    if output_file:
        reporter = Reporter(output_format)
        report_content = reporter.generate(report)

        with open(output_file, "w") as f:
            f.write(report_content)

        if verbose:
            print(f"Report written to: {output_file}")

    return report


def run_suites_with_coverage(
    suite_classes: List[Type[TestSuite]],
    source_dirs: List[str],
    output_format: ReportFormat = ReportFormat.Markdown,
    output_file: Optional[str] = None,
    verbose: bool = True,
    omit_patterns: Optional[List[str]] = None,
    **runner_kwargs: Any,
) -> List[TestReport]:
    """
    Run multiple test suites with combined coverage collection.

    Args:
        suite_classes: List of TestSuite subclasses to run
        source_dirs: Directories to measure coverage for
        output_format: Report output format
        output_file: Optional file path for combined report
        verbose: Whether to print verbose output
        omit_patterns: Patterns to omit from coverage
        **runner_kwargs: Additional arguments for TestRunner

    Returns:
        List of TestReports with coverage data
    """
    try:
        import coverage
    except ImportError:
        raise ImportError("coverage.py is required for coverage collection. Install with: pip install coverage")

    # Start coverage
    cov = coverage.Coverage(
        source=source_dirs,
        omit=omit_patterns or ["*test*", "*__pycache__*"],
    )
    cov.start()

    reports = []
    try:
        for suite_class in suite_classes:
            report = run_suite(
                suite_class,
                output_format=output_format,
                verbose=verbose,
                **runner_kwargs,
            )
            reports.append(report)
    finally:
        # Stop and save coverage
        cov.stop()
        cov.save()

    # Collect coverage data
    coverage_info = _collect_coverage_from_coveragepy(
        source_dirs,
        omit_patterns=omit_patterns or ["test_", "__pycache__"],
    )

    # Add coverage to all reports (shared coverage data)
    if coverage_info:
        for report in reports:
            report.set_coverage(coverage_info)

        if verbose:
            print(f"\nCoverage: {coverage_info.coverage_percent:.1f}% "
                  f"({coverage_info.covered_statements}/{coverage_info.total_statements} statements)")

    # Optionally combine reports into one file
    if output_file and reports:
        reporter = Reporter(output_format)
        combined = "\n\n---\n\n".join(reporter.generate(r) for r in reports)

        with open(output_file, "w") as f:
            f.write(combined)

        if verbose:
            print(f"\nCombined report written to: {output_file}")

    return reports

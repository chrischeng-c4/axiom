"""
Tests for OpenTelemetry integration.

Tests graceful degradation when OpenTelemetry SDK is not installed,
and proper instrumentation when it is available.
"""
import os
from unittest.mock import Mock, patch, MagicMock
from cclab.probe import expect, test
from tests.postgres.base import PostgresSuite

class TestTelemetry(PostgresSuite):

    @test
    def test_import_without_otel(self):
        """Test that telemetry module can be imported without OpenTelemetry SDK."""
        from cclab.titan import telemetry
        expect(telemetry).to_not_be_none()

    @test
    def test_is_tracing_enabled_without_otel(self):
        """Test is_tracing_enabled returns False when OpenTelemetry is not available."""
        with patch('cclab.titan.telemetry.OTEL_AVAILABLE', False):
            from cclab.titan.telemetry import is_tracing_enabled
            expect(is_tracing_enabled()).to_be(False)

    @test
    def test_is_tracing_enabled_with_env_var(self):
        """Test is_tracing_enabled respects environment variable."""
        with patch('cclab.titan.telemetry.OTEL_AVAILABLE', True):
            from cclab.titan.telemetry import is_tracing_enabled
            with patch.dict(os.environ, {}, clear=True):
                expect(is_tracing_enabled()).to_be(True)
            with patch.dict(os.environ, {'DATA_BRIDGE_TRACING_ENABLED': 'false'}):
                expect(is_tracing_enabled()).to_be(False)
            with patch.dict(os.environ, {'DATA_BRIDGE_TRACING_ENABLED': '0'}):
                expect(is_tracing_enabled()).to_be(False)
            with patch.dict(os.environ, {'DATA_BRIDGE_TRACING_ENABLED': 'no'}):
                expect(is_tracing_enabled()).to_be(False)
            with patch.dict(os.environ, {'DATA_BRIDGE_TRACING_ENABLED': 'true'}):
                expect(is_tracing_enabled()).to_be(True)

    @test
    def test_get_tracer_without_otel(self):
        """Test get_tracer returns None when OpenTelemetry is not available."""
        with patch('cclab.titan.telemetry.OTEL_AVAILABLE', False):
            from cclab.titan.telemetry import get_tracer
            tracer = get_tracer()
            expect(tracer).to_be_none()

    @test
    def test_get_meter_without_otel(self):
        """Test get_meter returns None when OpenTelemetry is not available."""
        with patch('cclab.titan.telemetry.OTEL_AVAILABLE', False):
            from cclab.titan.telemetry import get_meter
            meter = get_meter()
            expect(meter).to_be_none()

    @test
    def test_create_query_span_without_otel(self):
        """Test create_query_span works without OpenTelemetry (no-op)."""
        with patch('cclab.titan.telemetry.OTEL_AVAILABLE', False):
            from cclab.titan.telemetry import create_query_span
            with create_query_span('find', 'users', filters_count=2) as span:
                expect(span).to_be_none()

    @test
    def test_create_session_span_without_otel(self):
        """Test create_session_span works without OpenTelemetry (no-op)."""
        with patch('cclab.titan.telemetry.OTEL_AVAILABLE', False):
            from cclab.titan.telemetry import create_session_span
            with create_session_span('flush', pending_count=5) as span:
                expect(span).to_be_none()

    @test
    def test_create_relationship_span_without_otel(self):
        """Test create_relationship_span works without OpenTelemetry (no-op)."""
        with patch('cclab.titan.telemetry.OTEL_AVAILABLE', False):
            from cclab.titan.telemetry import create_relationship_span
            with create_relationship_span('user.posts', strategy='selectin') as span:
                expect(span).to_be_none()

    @test
    def test_add_exception_without_otel(self):
        """Test add_exception handles None span gracefully."""
        from cclab.titan.telemetry import add_exception
        add_exception(None, ValueError('test error'))

    @test
    def test_set_span_result_without_otel(self):
        """Test set_span_result handles None span gracefully."""
        from cclab.titan.telemetry import set_span_result
        set_span_result(None, count=10, affected_rows=5)

    @test
    async def test_instrument_span_decorator_async(self):
        """Test instrument_span decorator on async function."""
        with patch('cclab.titan.telemetry.OTEL_AVAILABLE', False):
            from cclab.titan.telemetry import instrument_span

            @instrument_span('test.operation')
            async def async_func(value: int) -> int:
                return value * 2
            result = await async_func(5)
            expect(result).to_equal(10)

    @test
    def test_instrument_span_decorator_sync(self):
        """Test instrument_span decorator on sync function."""
        with patch('cclab.titan.telemetry.OTEL_AVAILABLE', False):
            from cclab.titan.telemetry import instrument_span

            @instrument_span('test.operation')
            def sync_func(value: int) -> int:
                return value * 2
            result = sync_func(5)
            expect(result).to_equal(10)

    @test
    async def test_instrument_query_decorator_async(self):
        """Test instrument_query decorator on async function."""
        with patch('cclab.titan.telemetry.OTEL_AVAILABLE', False):
            from cclab.titan.telemetry import instrument_query

            @instrument_query('find')
            async def async_find(query: str) -> list:
                return ['result1', 'result2']
            result = await async_find('test query')
            expect(result).to_equal(['result1', 'result2'])

    @test
    def test_instrument_query_decorator_sync(self):
        """Test instrument_query decorator on sync function."""
        with patch('cclab.titan.telemetry.OTEL_AVAILABLE', False):
            from cclab.titan.telemetry import instrument_query

            @instrument_query('find')
            def sync_find(query: str) -> list:
                return ['result1', 'result2']
            result = sync_find('test query')
            expect(result).to_equal(['result1', 'result2'])

    @test
    async def test_instrument_session_decorator_async(self):
        """Test instrument_session decorator on async function."""
        with patch('cclab.titan.telemetry.OTEL_AVAILABLE', False):
            from cclab.titan.telemetry import instrument_session

            @instrument_session('flush')
            async def async_flush() -> None:
                pass
            await async_flush()

    @test
    def test_instrument_session_decorator_sync(self):
        """Test instrument_session decorator on sync function."""
        with patch('cclab.titan.telemetry.OTEL_AVAILABLE', False):
            from cclab.titan.telemetry import instrument_session

            @instrument_session('flush')
            def sync_flush() -> None:
                pass
            sync_flush()

    @test
    def test_connection_pool_metrics_without_otel(self):
        """Test ConnectionPoolMetrics without OpenTelemetry."""
        with patch('cclab.titan.telemetry.OTEL_AVAILABLE', False):
            from cclab.titan.telemetry import ConnectionPoolMetrics
            metrics = ConnectionPoolMetrics()
            metrics.record_pool_stats(in_use=5, idle=3, max_size=10)

    @test
    def test_get_connection_pool_metrics(self):
        """Test get_connection_pool_metrics returns singleton instance."""
        with patch('cclab.titan.telemetry.OTEL_AVAILABLE', False):
            from cclab.titan.telemetry import get_connection_pool_metrics
            metrics1 = get_connection_pool_metrics()
            metrics2 = get_connection_pool_metrics()
            expect(metrics1).to_be(metrics2)

    @test
    def test_semantic_conventions(self):
        """Test semantic convention constants are defined."""
        from cclab.titan.telemetry import SpanAttributes, MetricNames
        expect(SpanAttributes.DB_SYSTEM).to_equal('db.system')
        expect(SpanAttributes.DB_OPERATION_NAME).to_equal('db.operation.name')
        expect(SpanAttributes.DB_COLLECTION_NAME).to_equal('db.collection.name')
        expect(SpanAttributes.DB_STATEMENT).to_equal('db.statement')
        expect(MetricNames.CONNECTION_POOL_IN_USE).to_equal('db.connection.pool.in_use')
        expect(MetricNames.CONNECTION_POOL_IDLE).to_equal('db.connection.pool.idle')
        expect(MetricNames.CONNECTION_POOL_MAX).to_equal('db.connection.pool.max')

    @test
    def test_exports(self):
        """Test that all expected functions are exported."""
        from cclab.titan import telemetry
        expected_exports = ['is_tracing_enabled', 'get_tracer', 'get_meter', 'SpanAttributes', 'MetricNames', 'create_query_span', 'create_session_span', 'create_relationship_span', 'add_exception', 'set_span_result', 'instrument_span', 'instrument_query', 'instrument_session', 'ConnectionPoolMetrics', 'get_connection_pool_metrics']
        for export in expected_exports:
            expect(hasattr(telemetry, export)).to_be_true()

    @test
    def test_module_imports_from_postgres_package(self):
        """Test that telemetry functions can be imported from postgres package."""
        from cclab.titan import is_tracing_enabled, get_tracer, get_meter, SpanAttributes, MetricNames, create_query_span, create_session_span, create_relationship_span, add_exception, set_span_result, instrument_span, instrument_query, instrument_session, ConnectionPoolMetrics, get_connection_pool_metrics
        expect(is_tracing_enabled).to_not_be_none()
        expect(get_tracer).to_not_be_none()
        expect(get_meter).to_not_be_none()
        expect(SpanAttributes).to_not_be_none()
        expect(MetricNames).to_not_be_none()
        expect(create_query_span).to_not_be_none()
        expect(create_session_span).to_not_be_none()
        expect(create_relationship_span).to_not_be_none()
        expect(add_exception).to_not_be_none()
        expect(set_span_result).to_not_be_none()
        expect(instrument_span).to_not_be_none()
        expect(instrument_query).to_not_be_none()
        expect(instrument_session).to_not_be_none()
        expect(ConnectionPoolMetrics).to_not_be_none()
        expect(get_connection_pool_metrics).to_not_be_none()

    @test
    def test_query_builder_imports_telemetry(self):
        """Test that QueryBuilder imports telemetry functions correctly."""
        from cclab.titan.query import QueryBuilder
        import cclab.titan.query as query_module
        expect(hasattr(query_module, 'create_query_span')).to_be_true()
        expect(hasattr(query_module, 'set_span_result')).to_be_true()
        expect(hasattr(query_module, 'add_exception')).to_be_true()
        expect(hasattr(query_module, 'is_tracing_enabled')).to_be_true()
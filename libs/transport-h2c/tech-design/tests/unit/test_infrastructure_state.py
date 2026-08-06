from __future__ import annotations

import sys
import unittest
from dataclasses import replace
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from transport_h2c.domain.errors import (
    Connect,
    H2Protocol,
    InvalidRequest,
    Timeout,
)
from transport_h2c.infrastructure.config import (
    admission_permits,
    default_config,
    for_concurrency,
)
from transport_h2c.infrastructure.connection import (
    ConnectionState,
    idle_ms,
    mark_dead,
    record_send,
    release,
    reserve,
    touch,
)
from transport_h2c.infrastructure.stats import snapshot


class TestInfrastructureState(unittest.TestCase):
    def test_default_config_defaults(self) -> None:
        cfg = default_config(8)
        self.assertEqual(cfg.min_connections, 1)
        self.assertEqual(cfg.max_connections, 5)
        self.assertEqual(cfg.max_keepalive_connections, 16)
        self.assertEqual(cfg.max_in_flight_per_origin, 128)
        self.assertEqual(cfg.grow_threshold, 32)
        self.assertEqual(cfg.pool_timeout_seconds, 5.0)
        self.assertEqual(cfg.connect_timeout_seconds, 5.0)
        self.assertEqual(cfg.request_timeout_seconds, 30.0)
        self.assertEqual(cfg.ping_interval_seconds, 15.0)
        self.assertEqual(cfg.idle_timeout_seconds, 5.0)
        self.assertEqual(cfg.stream_window_bytes, 1048576)
        self.assertEqual(cfg.conn_window_bytes, 4194304)
        self.assertEqual(cfg.max_frame_bytes, 16384)

    def test_default_config_parallelism_caps(self) -> None:
        self.assertEqual(default_config(2).max_connections, 2)
        self.assertEqual(default_config(1).max_connections, 1)
        self.assertEqual(default_config(0).max_connections, 1)

    def test_for_concurrency_constructors(self) -> None:
        cfg256 = for_concurrency(256, 8)
        self.assertEqual(cfg256.max_connections, 6)
        self.assertEqual(cfg256.max_in_flight_per_origin, 256)
        self.assertEqual(cfg256.grow_threshold, 32)

        cfg1 = for_concurrency(1, 8)
        self.assertEqual(cfg1.max_connections, 1)
        self.assertEqual(cfg1.max_in_flight_per_origin, 1)

        cfg0 = for_concurrency(0, 8)
        self.assertEqual(cfg0.max_connections, 1)
        self.assertEqual(cfg0.max_in_flight_per_origin, 1)

        cfg_million = for_concurrency(1_000_000, 4)
        self.assertEqual(cfg_million.max_connections, 4)
        self.assertEqual(cfg_million.max_in_flight_per_origin, 1_000_000)

        cfg_low_core = for_concurrency(256, 2)
        self.assertEqual(cfg_low_core.max_connections, 2)
        self.assertEqual(cfg_low_core.max_in_flight_per_origin, 256)
        self.assertEqual(cfg_low_core.max_keepalive_connections, 16)

    def test_admission_permits(self) -> None:
        self.assertEqual(admission_permits(default_config(8)), 128)
        self.assertEqual(admission_permits(for_concurrency(0, 8)), 1)
        zero_max = replace(default_config(8), max_in_flight_per_origin=0)
        self.assertEqual(admission_permits(zero_max), 1)

    def test_connection_state_fresh(self) -> None:
        conn = ConnectionState(id=1)
        self.assertTrue(conn.healthy)
        self.assertEqual(conn.in_flight, 0)
        self.assertEqual(conn.total, 0)
        self.assertEqual(conn.errors, 0)
        self.assertEqual(conn.last_used_ms, 0)

    def test_reserve_and_release(self) -> None:
        conn = ConnectionState(id=1)
        reserve(conn)
        reserve(conn)
        reserve(conn)
        release(conn)
        self.assertEqual(conn.in_flight, 2)

    def test_mark_dead(self) -> None:
        conn = ConnectionState(id=1)
        mark_dead(conn)
        self.assertFalse(conn.healthy)
        mark_dead(conn)
        self.assertFalse(conn.healthy)
        self.assertEqual(conn.total, 0)
        self.assertEqual(conn.errors, 0)

    def test_record_send_success(self) -> None:
        conn = ConnectionState(id=1)
        record_send(conn, None)
        self.assertEqual(conn.total, 1)
        self.assertEqual(conn.errors, 0)
        self.assertTrue(conn.healthy)
        self.assertEqual(conn.in_flight, 0)

    def test_record_send_non_lost_errors(self) -> None:
        conn = ConnectionState(id=1)
        record_send(conn, Timeout(1.0))
        self.assertEqual(conn.total, 1)
        self.assertEqual(conn.errors, 1)
        self.assertTrue(conn.healthy)

        conn2 = ConnectionState(id=2)
        record_send(conn2, H2Protocol())
        self.assertEqual(conn2.total, 1)
        self.assertEqual(conn2.errors, 1)
        self.assertTrue(conn2.healthy)

        conn3 = ConnectionState(id=3)
        record_send(conn3, InvalidRequest("x"))
        self.assertEqual(conn3.total, 1)
        self.assertEqual(conn3.errors, 1)
        self.assertTrue(conn3.healthy)

    def test_record_send_lost_errors(self) -> None:
        conn = ConnectionState(id=1)
        record_send(conn, H2Protocol(go_away=True))
        self.assertEqual(conn.total, 1)
        self.assertEqual(conn.errors, 1)
        self.assertFalse(conn.healthy)

        conn2 = ConnectionState(id=2)
        record_send(conn2, Connect("a", "refused"))
        self.assertEqual(conn2.total, 1)
        self.assertEqual(conn2.errors, 1)
        self.assertFalse(conn2.healthy)

        conn3 = ConnectionState(id=3)
        record_send(conn3, H2Protocol(io=True))
        self.assertEqual(conn3.total, 1)
        self.assertEqual(conn3.errors, 1)
        self.assertFalse(conn3.healthy)

    def test_in_flight_non_interference(self) -> None:
        conn = ConnectionState(id=1)
        reserve(conn)
        record_send(conn, None)
        self.assertEqual(conn.in_flight, 1)

        conn2 = ConnectionState(id=2)
        reserve(conn2)
        record_send(conn2, H2Protocol(io=True))
        self.assertEqual(conn2.in_flight, 1)
        self.assertFalse(conn2.healthy)

    def test_touch_and_idle_ms(self) -> None:
        conn = ConnectionState(id=1)
        touch(conn, 1500)
        self.assertEqual(idle_ms(conn, 2000), 500)
        touch(conn, 2000)
        self.assertEqual(idle_ms(conn, 1500), 0)

    def test_snapshot_mixed_pool(self) -> None:
        conn_a = ConnectionState(id=1, healthy=True, in_flight=2, total=10, errors=1)
        conn_b = ConnectionState(id=2, healthy=True, in_flight=0, total=5, errors=0)
        conn_c = ConnectionState(id=3, healthy=False, in_flight=1, total=7, errors=3)

        st = snapshot([conn_a, conn_b, conn_c], 100, 9)
        self.assertEqual(st.connections, 3)
        self.assertEqual(st.healthy, 2)
        self.assertEqual(st.in_flight, 3)
        self.assertEqual(st.total_requests, 122)
        self.assertEqual(st.total_errors, 13)

    def test_snapshot_empty_pool(self) -> None:
        st0 = snapshot([], 0, 0)
        self.assertEqual(st0.connections, 0)
        self.assertEqual(st0.healthy, 0)
        self.assertEqual(st0.in_flight, 0)
        self.assertEqual(st0.total_requests, 0)
        self.assertEqual(st0.total_errors, 0)

        st_ret = snapshot([], 100, 9)
        self.assertEqual(st_ret.connections, 0)
        self.assertEqual(st_ret.healthy, 0)
        self.assertEqual(st_ret.in_flight, 0)
        self.assertEqual(st_ret.total_requests, 100)
        self.assertEqual(st_ret.total_errors, 9)


if __name__ == "__main__":
    unittest.main()

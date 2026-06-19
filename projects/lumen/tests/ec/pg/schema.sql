-- EC corpus database for the competitive search perf gate.
-- The gate (tests/perf_gate_vs_db.rs) creates its own table and inserts the
-- 1M-doc corpus at runtime; it only needs the `lumenbench` database to exist.
-- vat's postgres preset applies this seed once during cold-prepare, then caches
-- the populated cluster (clonefile-COW) so it is never rebuilt.
CREATE DATABASE lumenbench;

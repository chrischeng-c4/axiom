use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/tracemalloc/clear_traces_safe_when_not_tracing.py`.
#[test]
fn test_gen_behavior_std_libs_tracemalloc_clear_traces_safe_when_not_tracing() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "behavior"
# case = "clear_traces_safe_when_not_tracing"
# subject = "tracemalloc.clear_traces"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.clear_traces: clear_traces() is a safe no-op whether or not tracing is active and leaves is_tracing() False"""
import tracemalloc

if tracemalloc.is_tracing():
    tracemalloc.stop()

# clear_traces is callable whether or not tracing was ever started.
tracemalloc.clear_traces()
assert tracemalloc.is_tracing() is False, "not tracing after clear while stopped"

tracemalloc.start()
tracemalloc.clear_traces()
assert tracemalloc.is_tracing() is True, "still tracing after clear while started"
tracemalloc.stop()

print("clear_traces_safe_when_not_tracing OK")
"###);
    assert_output(&out, r###"clear_traces_safe_when_not_tracing OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tracemalloc/compare_to_statistic_diff.py`.
#[test]
fn test_gen_behavior_std_libs_tracemalloc_compare_to_statistic_diff() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "behavior"
# case = "compare_to_statistic_diff"
# subject = "tracemalloc.Snapshot"
# kind = "semantic"
# xfail = "mamba does not implement the tracemalloc.Snapshot class (#1261 long-tail stub batch)"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.Snapshot: compare_to yields StatisticDiff entries with size/count deltas (a.py:5 grew by 5000 B, count +1)"""
import tracemalloc

RAW1 = [
    (0, 10, (("a.py", 2), ("b.py", 4)), 3),
    (0, 10, (("a.py", 2), ("b.py", 4)), 3),
    (0, 10, (("a.py", 2), ("b.py", 4)), 3),
    (1, 2, (("a.py", 5), ("b.py", 4)), 3),
    (2, 66, (("b.py", 1),), 1),
    (3, 7, (("<unknown>", 0),), 1),
]
RAW2 = [
    (0, 10, (("a.py", 2), ("b.py", 4)), 3),
    (2, 2, (("a.py", 5), ("b.py", 4)), 3),
    (2, 5000, (("a.py", 5), ("b.py", 4)), 3),
    (4, 400, (("c.py", 578),), 1),
]
snap1 = tracemalloc.Snapshot(RAW1, 2)
snap2 = tracemalloc.Snapshot(RAW2, 2)

# compare_to yields StatisticDiff entries with size/count deltas.
diff = snap2.compare_to(snap1, "lineno")
grew = diff[0]
assert str(grew.traceback) == "a.py:5", "biggest growth is a.py:5"
assert grew.size == 5002, "a.py:5 new size"
assert grew.size_diff == 5000, "a.py:5 size delta"
assert grew.count_diff == 1, "a.py:5 count delta"

print("compare_to_statistic_diff OK")
"###);
    assert_output(&out, r###"compare_to_statistic_diff OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tracemalloc/domain_filter_fields.py`.
#[test]
fn test_gen_behavior_std_libs_tracemalloc_domain_filter_fields() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "behavior"
# case = "domain_filter_fields"
# subject = "tracemalloc.DomainFilter"
# kind = "semantic"
# xfail = "mamba does not implement the tracemalloc.DomainFilter class (#1261 long-tail stub batch)"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.DomainFilter: DomainFilter(True, 5) exposes inclusive True and domain 5"""
import tracemalloc

# DomainFilter carries inclusive + domain.
d = tracemalloc.DomainFilter(True, 5)
assert d.inclusive is True, "domain filter inclusive"
assert d.domain == 5, "domain value"

print("domain_filter_fields OK")
"###);
    assert_output(&out, r###"domain_filter_fields OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tracemalloc/filter_positional_and_keyword_match.py`.
#[test]
fn test_gen_behavior_std_libs_tracemalloc_filter_positional_and_keyword_match() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "behavior"
# case = "filter_positional_and_keyword_match"
# subject = "tracemalloc.Filter"
# kind = "semantic"
# xfail = "mamba does not implement the tracemalloc.Filter class (#1261 long-tail stub batch)"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.Filter: full positional Filter(False, 'test.py', 123, True) equals the keyword-built Filter field-for-field"""
import tracemalloc

# Full positional construction.
f = tracemalloc.Filter(False, "test.py", 123, True)
assert f.inclusive is False, "inclusive False"
assert f.filename_pattern == "test.py", "pattern test.py"
assert f.lineno == 123, "lineno 123"
assert f.all_frames is True, "all_frames True"

# Keyword construction matches positional.
g = tracemalloc.Filter(
    inclusive=False, filename_pattern="test.py", lineno=123, all_frames=True
)
assert (g.inclusive, g.filename_pattern, g.lineno, g.all_frames) == (
    False,
    "test.py",
    123,
    True,
), "keyword construction"

print("filter_positional_and_keyword_match OK")
"###);
    assert_output(&out, r###"filter_positional_and_keyword_match OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tracemalloc/filter_positional_defaults.py`.
#[test]
fn test_gen_behavior_std_libs_tracemalloc_filter_positional_defaults() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "behavior"
# case = "filter_positional_defaults"
# subject = "tracemalloc.Filter"
# kind = "semantic"
# xfail = "mamba does not implement the tracemalloc.Filter class (#1261 long-tail stub batch)"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.Filter: Filter(True, 'abc') defaults lineno to None and all_frames to False and exposes inclusive/filename_pattern"""
import tracemalloc

# Positional construction with defaults for lineno / all_frames.
f = tracemalloc.Filter(True, "abc")
assert f.inclusive is True, "inclusive"
assert f.filename_pattern == "abc", "filename_pattern"
assert f.lineno is None, "lineno default None"
assert f.all_frames is False, "all_frames default False"

print("filter_positional_defaults OK")
"###);
    assert_output(&out, r###"filter_positional_defaults OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tracemalloc/get_traceback_limit_retains_last_value.py`.
#[test]
fn test_gen_behavior_std_libs_tracemalloc_get_traceback_limit_retains_last_value() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "behavior"
# case = "get_traceback_limit_retains_last_value"
# subject = "tracemalloc.get_traceback_limit"
# kind = "semantic"
# xfail = "mamba tracemalloc is a GC-counter shim; start does not record the traceback limit, so get_traceback_limit does not reflect start(n) (#1261 long-tail stub batch)"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.get_traceback_limit: get_traceback_limit() defaults to 1 and after start(n)/stop retains the most recently configured limit n"""
import tracemalloc

if tracemalloc.is_tracing():
    tracemalloc.stop()

# Fresh in this process the limit defaults to 1.
assert tracemalloc.get_traceback_limit() == 1, "default limit is 1"

# start(n) sets the limit; it is observable while tracing.
tracemalloc.start(5)
assert tracemalloc.get_traceback_limit() == 5, "limit reflects start(5)"

# After stop the most recently configured limit is retained (not zeroed).
tracemalloc.stop()
assert tracemalloc.get_traceback_limit() == 5, "limit retained after stop"

print("get_traceback_limit_retains_last_value OK")
"###);
    assert_output(&out, r###"get_traceback_limit_retains_last_value OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tracemalloc/get_traced_memory_zero_before_start.py`.
#[test]
fn test_gen_behavior_std_libs_tracemalloc_get_traced_memory_zero_before_start() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "behavior"
# case = "get_traced_memory_zero_before_start"
# subject = "tracemalloc.get_traced_memory"
# kind = "semantic"
# xfail = "mamba tracemalloc is a GC-counter shim; get_traced_memory does not report the (0, 0) not-tracing contract (#1261 long-tail stub batch)"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.get_traced_memory: get_traced_memory() returns (0, 0) before start() rather than raising"""
import tracemalloc

if tracemalloc.is_tracing():
    tracemalloc.stop()

assert tracemalloc.get_traced_memory() == (0, 0), "zero before start"

print("get_traced_memory_zero_before_start OK")
"###);
    assert_output(&out, r###"get_traced_memory_zero_before_start OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tracemalloc/snapshot_filter_traces_non_mutating.py`.
#[test]
fn test_gen_behavior_std_libs_tracemalloc_snapshot_filter_traces_non_mutating() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "behavior"
# case = "snapshot_filter_traces_non_mutating"
# subject = "tracemalloc.Snapshot"
# kind = "semantic"
# xfail = "mamba does not implement the tracemalloc.Snapshot class (#1261 long-tail stub batch)"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.Snapshot: filter_traces with an exclusive Filter drops matching traces into a new snapshot and leaves the original intact; a DomainFilter keeps only the requested domain"""
import tracemalloc

RAW = [
    (0, 10, (("a.py", 2), ("b.py", 4)), 3),
    (1, 2, (("a.py", 5), ("b.py", 4)), 3),
    (2, 66, (("b.py", 1),), 1),
    (3, 7, (("<unknown>", 0),), 1),
]
snap = tracemalloc.Snapshot(RAW, 2)

# filter_traces with an exclusive Filter drops every frame matching b.py.
excl = tracemalloc.Filter(False, "b.py")
filtered = snap.filter_traces((excl,))
assert len(filtered.traces) == 3, "exclusive filter drops b.py-only trace"
# The original snapshot is untouched (filter_traces is non-mutating).
assert len(snap.traces) == 4, "original snapshot intact"

# DomainFilter keeps only the requested domain.
dom = tracemalloc.DomainFilter(True, domain=3)
only3 = snap.filter_traces((dom,))
assert len(only3.traces) == 1, "domain filter keeps one"

print("snapshot_filter_traces_non_mutating OK")
"###);
    assert_output(&out, r###"snapshot_filter_traces_non_mutating OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tracemalloc/snapshot_str_summaries_newest_frame.py`.
#[test]
fn test_gen_behavior_std_libs_tracemalloc_snapshot_str_summaries_newest_frame() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "behavior"
# case = "snapshot_str_summaries_newest_frame"
# subject = "tracemalloc.Snapshot"
# kind = "semantic"
# xfail = "mamba does not implement the tracemalloc.Snapshot class (#1261 long-tail stub batch)"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.Snapshot: str() of a trace, its traceback, and its newest frame all summarise the most-recent frame as 'b.py:4'"""
import tracemalloc

RAW = [
    (0, 10, (("a.py", 2), ("b.py", 4)), 3),
]
snap = tracemalloc.Snapshot(RAW, 2)
trace = snap.traces[0]

# str() of trace / traceback / frame all summarise the most-recent frame.
assert str(trace) == "b.py:4: 10 B", "trace str"
assert str(trace.traceback) == "b.py:4", "traceback str"
assert str(trace.traceback[0]) == "b.py:4", "frame str"

print("snapshot_str_summaries_newest_frame OK")
"###);
    assert_output(&out, r###"snapshot_str_summaries_newest_frame OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tracemalloc/snapshot_trace_fields_and_order.py`.
#[test]
fn test_gen_behavior_std_libs_tracemalloc_snapshot_trace_fields_and_order() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "behavior"
# case = "snapshot_trace_fields_and_order"
# subject = "tracemalloc.Snapshot"
# kind = "semantic"
# xfail = "mamba does not implement the tracemalloc.Snapshot class (#1261 long-tail stub batch)"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.Snapshot: a Snapshot built from raw traces exposes traceback_limit, trace count, per-trace size, and a traceback indexed newest-frame-first"""
import tracemalloc

# Raw traces: (domain, size, traceback_frames, total_nframe). A Snapshot can be
# built directly without live tracing, which makes the post-processing API
# deterministic.
RAW = [
    (0, 10, (("a.py", 2), ("b.py", 4)), 3),
    (1, 2, (("a.py", 5), ("b.py", 4)), 3),
    (2, 66, (("b.py", 1),), 1),
    (3, 7, (("<unknown>", 0),), 1),
]
snap = tracemalloc.Snapshot(RAW, 2)

# traceback_limit and number of recorded traces.
assert snap.traceback_limit == 2, "traceback_limit"
assert len(snap.traces) == 4, "trace count"

# A single trace exposes size + a traceback indexed most-recent-first
# (index 0 is the innermost / most-recent frame).
trace = snap.traces[0]
assert trace.size == 10, "trace.size"
assert trace.traceback.total_nframe == 3, "total_nframe"
assert len(trace.traceback) == 2, "frames in traceback"
assert trace.traceback[0].filename == "b.py", "newest frame filename"
assert trace.traceback[0].lineno == 4, "newest frame lineno"
assert trace.traceback[1].filename == "a.py", "older frame filename"

# Slicing a Traces sequence returns a plain tuple of traces.
assert snap.traces[:2] == (snap.traces[0], snap.traces[1]), "traces slice"

print("snapshot_trace_fields_and_order OK")
"###);
    assert_output(&out, r###"snapshot_trace_fields_and_order OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tracemalloc/start_enables_tracing_and_sets_limit.py`.
#[test]
fn test_gen_behavior_std_libs_tracemalloc_start_enables_tracing_and_sets_limit() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "behavior"
# case = "start_enables_tracing_and_sets_limit"
# subject = "tracemalloc.start"
# kind = "semantic"
# xfail = "mamba tracemalloc is a GC-counter shim; start/stop do not toggle is_tracing or record the traceback limit (#1261 long-tail stub batch)"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.start: start(1) makes is_tracing() True and get_traceback_limit() == 1; stop() makes is_tracing() False"""
import tracemalloc

if tracemalloc.is_tracing():
    tracemalloc.stop()

# start(nframe) enables tracing and sets the traceback depth.
tracemalloc.start(1)
assert tracemalloc.is_tracing() is True, "tracing after start"
assert tracemalloc.get_traceback_limit() == 1, "traceback limit"

# stop disables tracing.
tracemalloc.stop()
assert tracemalloc.is_tracing() is False, "not tracing after stop"

print("start_enables_tracing_and_sets_limit OK")
"###);
    assert_output(&out, r###"start_enables_tracing_and_sets_limit OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tracemalloc/statistic_str_formatting.py`.
#[test]
fn test_gen_behavior_std_libs_tracemalloc_statistic_str_formatting() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "behavior"
# case = "statistic_str_formatting"
# subject = "tracemalloc.Snapshot"
# kind = "semantic"
# xfail = "mamba does not implement the tracemalloc.Snapshot class (#1261 long-tail stub batch)"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.Snapshot: str() of a Statistic and a StatisticDiff render the human-readable size/count/average summary strings"""
import tracemalloc

RAW1 = [
    (0, 10, (("a.py", 2), ("b.py", 4)), 3),
    (0, 10, (("a.py", 2), ("b.py", 4)), 3),
    (0, 10, (("a.py", 2), ("b.py", 4)), 3),
    (1, 2, (("a.py", 5), ("b.py", 4)), 3),
    (2, 66, (("b.py", 1),), 1),
    (3, 7, (("<unknown>", 0),), 1),
]
RAW2 = [
    (0, 10, (("a.py", 2), ("b.py", 4)), 3),
    (2, 2, (("a.py", 5), ("b.py", 4)), 3),
    (2, 5000, (("a.py", 5), ("b.py", 4)), 3),
    (4, 400, (("c.py", 578),), 1),
]
snap1 = tracemalloc.Snapshot(RAW1, 2)
snap2 = tracemalloc.Snapshot(RAW2, 2)

top = snap1.statistics("lineno")[0]
grew = snap2.compare_to(snap1, "lineno")[0]

# str() formatting of Statistic and StatisticDiff.
assert str(top) == "b.py:1: size=66 B, count=1, average=66 B", "Statistic str"
assert (
    str(grew)
    == "a.py:5: size=5002 B (+5000 B), count=2 (+1), average=2501 B"
), "StatisticDiff str"

print("statistic_str_formatting OK")
"###);
    assert_output(&out, r###"statistic_str_formatting OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tracemalloc/statistics_filename_and_cumulative.py`.
#[test]
fn test_gen_behavior_std_libs_tracemalloc_statistics_filename_and_cumulative() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "behavior"
# case = "statistics_filename_and_cumulative"
# subject = "tracemalloc.Snapshot"
# kind = "semantic"
# xfail = "mamba does not implement the tracemalloc.Snapshot class (#1261 long-tail stub batch)"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.Snapshot: statistics('filename') collapses lines per file and cumulative=True counts a frame at every traceback depth (b.py cumulative is 98 B, count 5)"""
import tracemalloc

RAW1 = [
    (0, 10, (("a.py", 2), ("b.py", 4)), 3),
    (0, 10, (("a.py", 2), ("b.py", 4)), 3),
    (0, 10, (("a.py", 2), ("b.py", 4)), 3),
    (1, 2, (("a.py", 5), ("b.py", 4)), 3),
    (2, 66, (("b.py", 1),), 1),
    (3, 7, (("<unknown>", 0),), 1),
]
snap1 = tracemalloc.Snapshot(RAW1, 2)

# Group by filename collapses every line in a file (lineno reported as 0).
by_file = snap1.statistics("filename")
b_file = [s for s in by_file if str(s.traceback) == "b.py:0"][0]
assert b_file.size == 66, "b.py filename total"

# Cumulative aggregation counts a frame at every traceback depth.
cumulative = snap1.statistics("filename", cumulative=True)
b_cum = [s for s in cumulative if str(s.traceback) == "b.py:0"][0]
assert b_cum.size == 98, "b.py cumulative size"
assert b_cum.count == 5, "b.py cumulative count"

print("statistics_filename_and_cumulative OK")
"###);
    assert_output(&out, r###"statistics_filename_and_cumulative OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tracemalloc/statistics_group_by_lineno.py`.
#[test]
fn test_gen_behavior_std_libs_tracemalloc_statistics_group_by_lineno() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "behavior"
# case = "statistics_group_by_lineno"
# subject = "tracemalloc.Snapshot"
# kind = "semantic"
# xfail = "mamba does not implement the tracemalloc.Snapshot class (#1261 long-tail stub batch)"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.Snapshot: statistics('lineno') sorts groups by descending size and aggregates same-line traces (a.py:2 across 3 traces is 30 B, count 3)"""
import tracemalloc

RAW1 = [
    (0, 10, (("a.py", 2), ("b.py", 4)), 3),
    (0, 10, (("a.py", 2), ("b.py", 4)), 3),
    (0, 10, (("a.py", 2), ("b.py", 4)), 3),
    (1, 2, (("a.py", 5), ("b.py", 4)), 3),
    (2, 66, (("b.py", 1),), 1),
    (3, 7, (("<unknown>", 0),), 1),
]
snap1 = tracemalloc.Snapshot(RAW1, 2)

# Group by line number: results sorted by descending size.
by_line = snap1.statistics("lineno")
top = by_line[0]
assert str(top.traceback) == "b.py:1", "top group is b.py:1"
assert top.size == 66, "top group size"
assert top.count == 1, "top group count"
# Same line aggregated across traces: a.py:2 appears in 3 traces, 30 B.
a2 = [s for s in by_line if str(s.traceback) == "a.py:2"][0]
assert (a2.size, a2.count) == (30, 3), "a.py:2 aggregate"

print("statistics_group_by_lineno OK")
"###);
    assert_output(&out, r###"statistics_group_by_lineno OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tracemalloc/stop_zeroes_traced_memory.py`.
#[test]
fn test_gen_behavior_std_libs_tracemalloc_stop_zeroes_traced_memory() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "behavior"
# case = "stop_zeroes_traced_memory"
# subject = "tracemalloc.stop"
# kind = "semantic"
# xfail = "mamba tracemalloc is a GC-counter shim; stop does not zero the traced-memory counters (#1261 long-tail stub batch)"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.stop: after stop() get_traced_memory() returns (0, 0) and get_tracemalloc_memory() reports non-negative tracer overhead"""
import tracemalloc

if tracemalloc.is_tracing():
    tracemalloc.stop()

tracemalloc.start(1)
_blob = b"y" * 200000  # noqa: F841 - allocate something to trace

# get_tracemalloc_memory reports overhead used by the tracer itself.
overhead = tracemalloc.get_tracemalloc_memory()
assert overhead >= 0, "tracer overhead non-negative"

# stop disables tracing and zeroes the traced-memory counters.
tracemalloc.stop()
assert tracemalloc.is_tracing() is False, "not tracing after stop"
assert tracemalloc.get_traced_memory() == (0, 0), "counters zero after stop"

print("stop_zeroes_traced_memory OK")
"###);
    assert_output(&out, r###"stop_zeroes_traced_memory OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tracemalloc/traceback_format_orderings.py`.
#[test]
fn test_gen_behavior_std_libs_tracemalloc_traceback_format_orderings() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "behavior"
# case = "traceback_format_orderings"
# subject = "tracemalloc.Snapshot"
# kind = "semantic"
# xfail = "mamba does not implement the tracemalloc.Snapshot class (#1261 long-tail stub batch)"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.Snapshot: Traceback.format() renders newest-frame-first by default, honours limit, and reverses under most_recent_first"""
import tracemalloc

RAW = [
    (0, 10, (("a.py", 2), ("b.py", 4)), 3),
]
snap = tracemalloc.Snapshot(RAW, 2)
trace = snap.traces[0]

# Traceback.format() renders newest-frame-first by default.
fmt = trace.traceback.format()
assert fmt == ['  File "b.py", line 4', '  File "a.py", line 2'], "format default"
assert trace.traceback.format(limit=1) == ['  File "a.py", line 2'], "format limit"
assert trace.traceback.format(most_recent_first=True) == [
    '  File "a.py", line 2',
    '  File "b.py", line 4',
], "format most_recent_first"

print("traceback_format_orderings OK")
"###);
    assert_output(&out, r###"traceback_format_orderings OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tracemalloc/traceback_repr_frame_ordering.py`.
#[test]
fn test_gen_behavior_std_libs_tracemalloc_traceback_repr_frame_ordering() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "behavior"
# case = "traceback_repr_frame_ordering"
# subject = "tracemalloc.Traceback"
# kind = "semantic"
# xfail = "mamba does not implement the tracemalloc.Traceback class (#1261 long-tail stub batch)"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.Traceback: repr(Traceback) lists frames newest-first and only shows total_nframe when an explicit total is supplied"""
import tracemalloc

# Empty traceback.
assert repr(tracemalloc.Traceback(())) == "<Traceback ()>", "empty repr"
assert (
    repr(tracemalloc.Traceback((), 0)) == "<Traceback () total_nframe=0>"
), "empty repr with total"

# Frames render newest-first; constructor input is oldest-first.
frames = (("f1", 1), ("f2", 2))
exp_frames = "(<Frame filename='f2' lineno=2>, <Frame filename='f1' lineno=1>)"
assert repr(tracemalloc.Traceback(frames)) == f"<Traceback {exp_frames}>", "frames repr"
assert (
    repr(tracemalloc.Traceback(frames, 2))
    == f"<Traceback {exp_frames} total_nframe=2>"
), "frames repr with total"

print("traceback_repr_frame_ordering OK")
"###);
    assert_output(&out, r###"traceback_repr_frame_ordering OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tracemalloc/traced_memory_positive_peak_ge_current.py`.
#[test]
fn test_gen_behavior_std_libs_tracemalloc_traced_memory_positive_peak_ge_current() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "behavior"
# case = "traced_memory_positive_peak_ge_current"
# subject = "tracemalloc.get_traced_memory"
# kind = "semantic"
# xfail = "mamba tracemalloc is a GC-counter shim; get_traced_memory does not track per-allocation current/peak sizes (#1261 long-tail stub batch)"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.get_traced_memory: while tracing a large allocation get_traced_memory() reports current > 0 with peak >= current, and reset_peak keeps peak >= current"""
import tracemalloc

if tracemalloc.is_tracing():
    tracemalloc.stop()

tracemalloc.start(1)
tracemalloc.clear_traces()
blob = b"x" * 200000  # noqa: F841 - kept alive to count toward traced memory

# get_traced_memory reports a non-trivial current size and peak >= current.
current, peak = tracemalloc.get_traced_memory()
assert current > 0, "current traced memory positive"
assert peak >= current, "peak at least current"

# reset_peak keeps peak >= current but does not lose live allocations.
tracemalloc.reset_peak()
cur2, peak2 = tracemalloc.get_traced_memory()
assert peak2 >= cur2, "peak still >= current after reset"

tracemalloc.stop()

print("traced_memory_positive_peak_ge_current OK")
"###);
    assert_output(&out, r###"traced_memory_positive_peak_ge_current OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tracemalloc/traced_object_has_traceback_then_cleared.py`.
#[test]
fn test_gen_behavior_std_libs_tracemalloc_traced_object_has_traceback_then_cleared() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "behavior"
# case = "traced_object_has_traceback_then_cleared"
# subject = "tracemalloc.get_object_traceback"
# kind = "semantic"
# xfail = "mamba tracemalloc is a GC-counter shim; get_object_traceback does not record per-allocation tracebacks (#1261 long-tail stub batch)"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.get_object_traceback: a freshly allocated object has a recorded traceback while tracing, and clear_traces() drops it back to None"""
import tracemalloc

if tracemalloc.is_tracing():
    tracemalloc.stop()

tracemalloc.start(1)

# A freshly allocated object has a recorded traceback.
tracemalloc.clear_traces()
blob = b"x" * 200000
tb = tracemalloc.get_object_traceback(blob)
assert tb is not None, "object has traceback while tracing"

# clear_traces forgets recorded allocations.
tracemalloc.clear_traces()
assert tracemalloc.get_object_traceback(blob) is None, "traceback cleared"

tracemalloc.stop()

print("traced_object_has_traceback_then_cleared OK")
"###);
    assert_output(&out, r###"traced_object_has_traceback_then_cleared OK
"###);
}

#![cfg(test)]

/// Py3.12 behavioral conformance: stdlib module tests (T6-T12).
///
/// Part of py312-behavioral-conformance spec (mamba-conformance-p0 change):
///   T6:  math module (R6)
///   T7:  json module (R7)
///   T8:  re module (R8)
///   T9:  collections module (R9)
///   T10: datetime module (R10)
///   T11: itertools module (R12)
///   T12: functools module (R12)
///
/// Each test runs Python source through the full JIT pipeline:
///   parse -> type-check -> HIR -> MIR -> Cranelift JIT -> capture stdout -> verify
///
/// Tests marked `#[ignore]` require features not yet implemented (tracked as xfail
/// in the fixture-based harness). Remove `#[ignore]` as features land.

use crate::codegen::cranelift::jit::{CraneliftJitBackend, JIT_LOCK};
use crate::codegen::{CodegenBackend, CodegenOutput};
use crate::lower::{lower_hir_to_mir_with_symbols, lower_module};
use crate::parser;
use crate::runtime::cleanup_all_runtime_state;
use crate::runtime::output::{begin_capture, end_capture};
use crate::source::span::FileId;
use crate::types::TypeChecker;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

const TEST_TIMEOUT_SECS: u64 = 10;

/// Run Python source through the full JIT pipeline, capturing stdout.
fn jit_capture(src: &str) -> String {
    let _jit_guard = JIT_LOCK.lock().unwrap();

    let module = parser::parse(src, FileId(0)).expect("parse failed");
    let mut checker = TypeChecker::new();
    let errors = checker.check_module(&module);
    if !errors.is_empty() {
        panic!(
            "type errors: {:?}",
            errors.iter().map(|e| e.to_string()).collect::<Vec<_>>()
        );
    }

    let hir = lower_module(&module, &checker).expect("HIR lowering failed");
    let mir = lower_hir_to_mir_with_symbols(&hir, &checker.tcx, &checker.symbols);

    let mut backend = CraneliftJitBackend::new().expect("JIT init failed");
    let output = backend
        .codegen(&mir, &checker.tcx)
        .expect("JIT codegen failed");

    match output {
        CodegenOutput::Jit { entry } => {
            let entry_addr = entry as usize;
            let (tx, rx) = mpsc::sync_channel(1);

            let handle = thread::spawn(move || {
                let prev = begin_capture();
                let main_fn: fn() -> i64 = unsafe { std::mem::transmute(entry_addr) };
                let _result = main_fn();
                cleanup_all_runtime_state();
                let captured = end_capture(prev);
                let _ = tx.send(captured);
            });

            let result = match rx.recv_timeout(Duration::from_secs(TEST_TIMEOUT_SECS)) {
                Ok(captured) => captured,
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    panic!("JIT execution timed out after {TEST_TIMEOUT_SECS}s");
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    panic!("JIT execution thread panicked");
                }
            };

            let _ = handle.join();
            result
        }
        _ => panic!("expected JIT output"),
    }
}

/// Assert that captured output matches expected lines.
fn assert_output(actual: &str, expected: &str) {
    let actual_trimmed = actual.trim_end();
    let expected_trimmed = expected.trim_end();
    if actual_trimmed != expected_trimmed {
        let a_lines: Vec<&str> = actual_trimmed.lines().collect();
        let e_lines: Vec<&str> = expected_trimmed.lines().collect();
        let max = a_lines.len().max(e_lines.len());
        let mut diff = String::new();
        for i in 0..max {
            let a = a_lines.get(i).copied().unwrap_or("<missing>");
            let e = e_lines.get(i).copied().unwrap_or("<missing>");
            if a != e {
                diff.push_str(&format!("  line {}: expected {:?}, got {:?}\n", i + 1, e, a));
            }
        }
        panic!(
            "output mismatch:\n--- expected ---\n{expected_trimmed}\n--- actual ---\n{actual_trimmed}\n--- diff ---\n{diff}"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// T6: math Module (R6)
// ═══════════════════════════════════════════════════════════════════════════════

/// T6.1: math.floor(3.7) == 3.
#[test]
fn test_t6_1_math_floor() {
    let output = jit_capture("import math\nprint(math.floor(3.7))\n");
    assert_output(&output, "3\n");
}

/// T6.2: math.ceil(3.2) == 4.
#[test]
fn test_t6_2_math_ceil() {
    let output = jit_capture("import math\nprint(math.ceil(3.2))\n");
    assert_output(&output, "4\n");
}

/// T6.3: math.sqrt(16) == 4.0.
#[test]
fn test_t6_3_math_sqrt() {
    let output = jit_capture("import math\nprint(math.sqrt(16))\n");
    assert_output(&output, "4.0\n");
}

/// T6.4: math.factorial(5) == 120.
#[test]
fn test_t6_4_math_factorial() {
    let output = jit_capture("import math\nprint(math.factorial(5))\n");
    assert_output(&output, "120\n");
}

/// T6.5: math.gcd(12, 8) == 4.
#[test]
fn test_t6_5_math_gcd() {
    let output = jit_capture("import math\nprint(math.gcd(12, 8))\n");
    assert_output(&output, "4\n");
}

/// T6.6: math.isnan(float('nan')) == True.
#[test]
fn test_t6_6_math_isnan() {
    let output = jit_capture("import math\nprint(math.isnan(float('nan')))\n");
    assert_output(&output, "True\n");
}

/// T6.7: math.isinf(float('inf')) == True.
#[test]
fn test_t6_7_math_isinf() {
    let output = jit_capture("import math\nprint(math.isinf(float('inf')))\n");
    assert_output(&output, "True\n");
}

/// T6.8: math.comb(5, 2) == 10.
#[test]
fn test_t6_8_math_comb() {
    let output = jit_capture("import math\nprint(math.comb(5, 2))\n");
    assert_output(&output, "10\n");
}

/// T6.9: math.perm(5, 2) == 20.
#[test]
fn test_t6_9_math_perm() {
    let output = jit_capture("import math\nprint(math.perm(5, 2))\n");
    assert_output(&output, "20\n");
}

/// T6 supplemental: math.pi and math.e constants.
#[test]
fn test_t6_math_constants() {
    let output = jit_capture(
        r#"import math
print(math.pi)
print(math.e)
"#,
    );
    assert_output(&output, "3.141592653589793\n2.718281828459045\n");
}

// ═══════════════════════════════════════════════════════════════════════════════
// T7: json Module (R7)
// ═══════════════════════════════════════════════════════════════════════════════

/// T7.1: json.dumps({'a': 1}, sort_keys=True).
#[test]
fn test_t7_1_json_dumps_sort_keys() {
    let output = jit_capture(
        r#"import json
print(json.dumps({'a': 1}, sort_keys=True))
"#,
    );
    assert_output(&output, "{\"a\": 1}\n");
}

/// T7.2: json.loads round-trip.
#[test]
fn test_t7_2_json_loads() {
    let output = jit_capture(
        r#"import json
print(json.loads('{"x": [1, 2]}'))
"#,
    );
    assert_output(&output, "{'x': [1, 2]}\n");
}

/// T7.3: json.dumps with indent.
#[test]
fn test_t7_3_json_dumps_indent() {
    let output = jit_capture(
        r#"import json
print(json.dumps({'a': 1}, indent=2))
"#,
    );
    assert_output(&output, "{\n  \"a\": 1\n}\n");
}

/// T7.4: json round-trip equality.
#[test]
fn test_t7_4_json_roundtrip() {
    let output = jit_capture(
        r#"import json
d = {'name': 'test', 'values': [1, 2, 3]}
print(json.loads(json.dumps(d, sort_keys=True)) == d)
"#,
    );
    assert_output(&output, "True\n");
}

/// T7.5: hot-loop `json.loads` on a nested-dict payload with periodic
/// `print(i)` progress. Regression for #2109 — previously deadlocked the
/// main thread once the GC alloc threshold tripped *inside*
/// `json_to_mbvalue`'s recursive dict construction. The outer dict's
/// `RwLock::write` guard was held while a child `MbObject::new_dict`
/// allocation triggered `gc::collect()`, which then tried to take a
/// `RwLock::read` on the same outer dict — a single-thread self-deadlock
/// that surfaced as `_dispatch_semaphore_wait_slow` on macOS. Fix is to
/// build all child MbValues into a local Vec *before* acquiring the
/// outer write lock. See `json_mod.rs::json_to_mbvalue` for details.
#[test]
fn test_t7_5_json_loads_nested_dict_print_loop_2109() {
    let output = jit_capture(
        r#"import json
PAYLOAD = '{"service": "mamba-api", "version": "0.3.48", "enabled": true, "timeout_ms": 5000, "retries": 3, "endpoints": [{"path": "/v1/status", "method": "GET", "auth": false}, {"path": "/v1/run", "method": "POST", "auth": true}], "feature_flags": {"tracing": true, "metrics": false}, "tags": ["prod", "us-east-1"]}'
for i in range(2000):
    parsed = json.loads(PAYLOAD)
    out = json.dumps(parsed)
    if i % 100 == 0:
        print(i)
print("done")
"#,
    );
    // Must reach "done" without hanging.
    assert!(
        output.trim_end().ends_with("done"),
        "expected loop to complete; got:\n{output}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// T8: re Module (R8)
// ═══════════════════════════════════════════════════════════════════════════════

/// T8.1: re.match(r'\d+', '123abc').group() == '123'.
#[test]
fn test_t8_1_re_match() {
    let output = jit_capture(
        r#"import re
print(re.match(r'\d+', '123abc').group())
"#,
    );
    assert_output(&output, "123\n");
}

/// T8.2: re.findall(r'\d+', 'a1b2c3') == ['1', '2', '3'].
#[test]
fn test_t8_2_re_findall() {
    let output = jit_capture(
        r#"import re
print(re.findall(r'\d+', 'a1b2c3'))
"#,
    );
    assert_output(&output, "['1', '2', '3']\n");
}

/// T8.3: re.sub(r'\d', 'X', 'a1b2') == 'aXbX'.
#[test]
fn test_t8_3_re_sub() {
    let output = jit_capture(
        r#"import re
print(re.sub(r'\d', 'X', 'a1b2'))
"#,
    );
    assert_output(&output, "aXbX\n");
}

/// T8.4: re.split(r'[,;]', 'a,b;c') == ['a', 'b', 'c'].
#[test]
fn test_t8_4_re_split() {
    let output = jit_capture(
        r#"import re
print(re.split(r'[,;]', 'a,b;c'))
"#,
    );
    assert_output(&output, "['a', 'b', 'c']\n");
}

/// T8.5: named groups.
#[test]
fn test_t8_5_re_named_groups() {
    let output = jit_capture(
        r#"import re
m = re.match(r'(?P<name>\w+)', 'hello')
print(m.group('name'))
"#,
    );
    assert_output(&output, "hello\n");
}

// ═══════════════════════════════════════════════════════════════════════════════
// T9: collections Module (R9)
// ═══════════════════════════════════════════════════════════════════════════════

/// T9.1: Counter('abracadabra').most_common(3).
#[test]
fn test_t9_1_counter_most_common() {
    let output = jit_capture(
        r#"from collections import Counter
c = Counter('abracadabra')
print(c.most_common(3))
"#,
    );
    assert_output(&output, "[('a', 5), ('b', 2), ('r', 2)]\n");
}

/// T9.2: defaultdict(int)['x'] == 0.
#[test]
fn test_t9_2_defaultdict() {
    let output = jit_capture(
        r#"from collections import defaultdict
dd = defaultdict(int)
print(dd['x'])
"#,
    );
    assert_output(&output, "0\n");
}

/// T9.3: deque with maxlen.
#[test]
fn test_t9_3_deque_maxlen() {
    let output = jit_capture(
        r#"from collections import deque
d = deque([1, 2, 3], maxlen=3)
d.appendleft(0)
print(list(d))
"#,
    );
    assert_output(&output, "[0, 1, 2]\n");
}

/// T9.4: namedtuple construction and access.
#[test]
fn test_t9_4_namedtuple() {
    let output = jit_capture(
        r#"from collections import namedtuple
Point = namedtuple('Point', ['x', 'y'])
p = Point(1, 2)
print(p.x, p.y)
print(p)
"#,
    );
    assert_output(&output, "1 2\nPoint(x=1, y=2)\n");
}

// ═══════════════════════════════════════════════════════════════════════════════
// T10: datetime Module (R10)
// ═══════════════════════════════════════════════════════════════════════════════

/// T10.1: datetime strftime.
#[test]
fn test_t10_1_datetime_strftime() {
    let output = jit_capture(
        r#"from datetime import datetime
d = datetime(2024, 1, 15, 10, 30)
print(d.strftime('%Y-%m-%d'))
"#,
    );
    assert_output(&output, "2024-01-15\n");
}

/// T10.2: timedelta arithmetic.
#[test]
fn test_t10_2_timedelta_add() {
    let output = jit_capture(
        r#"from datetime import datetime, timedelta
d = datetime(2024, 1, 15, 10, 30)
d2 = d + timedelta(days=10)
print(d2.day)
"#,
    );
    assert_output(&output, "25\n");
}

/// T10.3: date comparison.
#[test]
fn test_t10_3_date_compare() {
    let output = jit_capture(
        r#"from datetime import date
d1 = date(2024, 1, 1)
d2 = date(2024, 12, 31)
print(d1 < d2)
"#,
    );
    assert_output(&output, "True\n");
}

// ═══════════════════════════════════════════════════════════════════════════════
// T11: itertools Module (R12)
// ═══════════════════════════════════════════════════════════════════════════════

/// T11.1: list(chain([1,2], [3,4])) == [1, 2, 3, 4].
#[test]
fn test_t11_1_itertools_chain() {
    let output = jit_capture(
        r#"from itertools import chain
print(list(chain([1, 2], [3, 4])))
"#,
    );
    assert_output(&output, "[1, 2, 3, 4]\n");
}

/// T11.2: list(islice(range(10), 2, 7, 2)) == [2, 4, 6].
#[test]
fn test_t11_2_itertools_islice() {
    let output = jit_capture(
        r#"from itertools import islice
print(list(islice(range(10), 2, 7, 2)))
"#,
    );
    assert_output(&output, "[2, 4, 6]\n");
}

/// T11.3: product('ab', '12') produces 4 tuples.
#[test]
fn test_t11_3_itertools_product() {
    let output = jit_capture(
        r#"from itertools import product
print(list(product('ab', '12')))
"#,
    );
    assert_output(
        &output,
        "[('a', '1'), ('a', '2'), ('b', '1'), ('b', '2')]\n",
    );
}

/// T11.4: combinations('abc', 2).
#[test]
fn test_t11_4_itertools_combinations() {
    let output = jit_capture(
        r#"from itertools import combinations
print(list(combinations('abc', 2)))
"#,
    );
    assert_output(&output, "[('a', 'b'), ('a', 'c'), ('b', 'c')]\n");
}

/// T11.5: permutations('ab').
#[test]
fn test_t11_5_itertools_permutations() {
    let output = jit_capture(
        r#"from itertools import permutations
print(list(permutations('ab')))
"#,
    );
    assert_output(&output, "[('a', 'b'), ('b', 'a')]\n");
}

/// T11.6 (#2182): `itertools.chain` must be lazy.
///
/// Reproducer for the cross-cutting "stdlib iterators ship as list-aliased"
/// gap. In CPython, `next(chain(a, b))` consumes exactly **one** element
/// from `a` and leaves the rest untouched; in mamba the shim at
/// `projects/mamba/src/runtime/stdlib/itertools_mod.rs::mb_itertools_chain`
/// (and ~20 sibling shims — `islice`, `count`, `cycle`, `repeat`,
/// `accumulate`, `takewhile`, `dropwhile`, `filterfalse`, `compress`,
/// `starmap`, `pairwise`, `batched`, `groupby`, `tee`, `zip_longest`,
/// `product`, `permutations`, `combinations`, `combinations_with_replacement`)
/// returns a fully materialized list because the runtime lacks an
/// `ObjKind::Generator` variant. The same gap shows up at
/// `csv.reader` / `csv.DictReader` (Wave-4), `xml.etree.ElementTree.iter`
/// (Wave-4), `glob.iglob` (Wave-5, aliased to `glob.glob`), `re.finditer`,
/// and `os.walk`.
///
/// The probe uses `itertools.count(0)` — an unbounded generator in CPython
/// — wrapped in `islice` to assert that *some* prefix is observable. Under
/// the current list-aliased shim, `count(0)` cannot return without an
/// upper bound, so the program fails at the `count` call rather than
/// stream the prefix. Remove `#[ignore]` once `ObjKind::Generator` lands
/// and `mb_itertools_count` is rewritten to construct a generator object
/// (closure-state or `runtime::generator::GenEntry` handle) instead of a
/// pre-allocated `Vec<MbValue>`.
///
/// Companion HANDWRITE carve-out: `projects/mamba/src/runtime/rc.rs`
/// (`ObjKind` + `ObjData` enums, #2182 reason).
#[test]
#[ignore = "#2182: itertools shims materialize lists eagerly; Generator runtime type not yet wired"]
fn test_itertools_chain_is_lazy_2182() {
    // Under a true generator runtime this prints `[0, 1, 2, 3, 4]`
    // without ever materializing the (infinite) tail of `count(0)`.
    // Under the current list-aliased shim the `count(0)` call cannot
    // terminate (it has no upper bound) — the JIT output therefore
    // diverges from CPython's `[0, 1, 2, 3, 4]\n`.
    let output = jit_capture(
        r#"from itertools import count, islice
print(list(islice(count(0), 5)))
"#,
    );
    assert_output(&output, "[0, 1, 2, 3, 4]\n");
}

// ═══════════════════════════════════════════════════════════════════════════════
// T12: functools Module (R12)
// ═══════════════════════════════════════════════════════════════════════════════

/// T12.1: reduce(lambda a,b: a+b, [1,2,3,4]) == 10.
#[test]
fn test_t12_1_functools_reduce() {
    let output = jit_capture(
        r#"from functools import reduce
print(reduce(lambda a, b: a + b, [1, 2, 3, 4]))
"#,
    );
    assert_output(&output, "10\n");
}

/// T12.2: partial(pow, 2)(10) == 1024.
#[test]
fn test_t12_2_functools_partial() {
    let output = jit_capture(
        r#"from functools import partial
pow2 = partial(pow, 2)
print(pow2(10))
"#,
    );
    assert_output(&output, "1024\n");
}

/// T12.3: lru_cache caches results (tests dynamic decorator dispatch).
#[test]
fn test_t12_3_functools_lru_cache() {
    let output = jit_capture(
        r#"from functools import lru_cache

call_count = 0

@lru_cache
def fib(n):
    global call_count
    call_count += 1
    if n < 2:
        return n
    return fib(n - 1) + fib(n - 2)

print(fib(10))
print(fib(10))
"#,
    );
    assert_output(&output, "55\n55\n");
}

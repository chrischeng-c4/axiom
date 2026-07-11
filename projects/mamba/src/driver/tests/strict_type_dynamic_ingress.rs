#![cfg(test)]

use crate::codegen::cranelift::jit::JIT_LOCK;
use crate::driver::{CompilerConfig, CompilerSession};
use crate::runtime::cleanup_all_runtime_state;
use crate::runtime::output::{begin_capture, end_capture};

fn run(source: &str) -> (Result<(), String>, String) {
    let _guard = JIT_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let previous = begin_capture();
    let mut session = CompilerSession::new(CompilerConfig::default());
    let result = session
        .run_source(source, "<strict-type-dynamic-ingress>")
        .map_err(|error| error.to_string());
    let output = end_capture(previous);
    cleanup_all_runtime_state();
    (result, output)
}

fn line_count(output: &str, expected: &str) -> usize {
    output.lines().filter(|line| *line == expected).count()
}

#[test]
fn dynamic_routes_reject_before_body_and_keep_bound_kw_default() {
    let (result, output) = run(
        r#"
from typing import Any

def reject_raw(value: int) -> int:
    print("BAD_RAW")
    return 0

raw_fn: Any = reject_raw
try:
    raw_fn("wrong")
except TypeError as exc:
    print("REJECT_RAW", str(exc))

def reject_keyword(value: int) -> int:
    print("BAD_KEYWORD")
    return 0

keyword_fn: Any = reject_keyword
try:
    keyword_fn(value="wrong")
except TypeError:
    print("REJECT_KEYWORD")

def reject_star(value: int) -> int:
    print("BAD_STAR")
    return 0

star_fn: Any = reject_star
try:
    star_fn(*["wrong"])
except TypeError:
    print("REJECT_STAR")

def reject_kwstar(value: int) -> int:
    print("BAD_KWSTAR")
    return 0

kwstar_fn: Any = reject_kwstar
try:
    kwstar_fn(**{"value": "wrong"})
except TypeError:
    print("REJECT_KWSTAR")

def reject_global(value: int) -> int:
    print("BAD_GLOBAL")
    return 0

global_fn: Any = globals()["reject_global"]
def call_global() -> int:
    try:
        global_fn("wrong")
    except TypeError:
        print("REJECT_GLOBAL")
    return 0

call_global()

wrong_default: Any = "wrong"
def reject_default(value: int = wrong_default) -> int:
    print("BAD_DEFAULT")
    return 0

default_fn: Any = reject_default
try:
    default_fn()
except TypeError:
    print("REJECT_DEFAULT")

def make_reject_closure() -> Any:
    captured = 7
    def inner(value: int) -> int:
        print("BAD_CLOSURE")
        return captured
    return inner

closure_fn: Any = make_reject_closure()
try:
    closure_fn("wrong")
except TypeError:
    print("REJECT_CLOSURE")

def identity(fn: Any) -> Any:
    return fn

@identity
def reject_decorated(value: int) -> int:
    print("BAD_DECORATED")
    return 0

decorated_fn: Any = reject_decorated
try:
    decorated_fn("wrong")
except TypeError:
    print("REJECT_DECORATED")

class Holder:
    def reject_bound(self, value: int) -> int:
        print("BAD_BOUND")
        return 0

    def keep_default(self, *, value: int = 4) -> int:
        print("BODY_BOUND_DEFAULT", value)
        return 0

holder = Holder()
bound_fn: Any = holder.reject_bound
try:
    bound_fn("wrong")
except TypeError:
    print("REJECT_BOUND")

def reject_keyword_only(*, value: int) -> int:
    print("BAD_KEYWORD_ONLY")
    return 0

keyword_only_fn: Any = reject_keyword_only
try:
    keyword_only_fn(value="wrong")
except TypeError:
    print("REJECT_KEYWORD_ONLY")

bound_default_fn: Any = holder.keep_default
bound_default_fn()
"#,
    );
    assert!(result.is_ok(), "unexpected session error: {result:?}\n{output}");
    assert!(!output.contains("BAD_"), "rejected body executed:\n{output}");
    assert_eq!(
        line_count(
            &output,
            "REJECT_RAW reject_raw() argument 'value' expected int, got str"
        ),
        1,
        "{output}"
    );
    assert_eq!(
        output
            .lines()
            .filter(|line| line.starts_with("REJECT_"))
            .count(),
        10,
        "{output}"
    );
    assert_eq!(line_count(&output, "BODY_BOUND_DEFAULT 4"), 1, "{output}");
}

#[test]
fn scalar_contracts_aliases_and_fail_open_annotations() {
    let (result, output) = run(
        r#"
from typing import Any

type Count = int
type Blob = bytes

def take_count(value: Count) -> int:
    print("BODY_COUNT", value)
    return 0

def take_bool(value: bool) -> int:
    print("BODY_BOOL", value)
    return 0

def take_float(value: float) -> int:
    print("BODY_FLOAT", value)
    return 0

def take_str(value: str) -> int:
    print("BODY_STR", value)
    return 0

def take_blob(value: Blob) -> int:
    print("BODY_BLOB", value)
    return 0

def take_none(value: None) -> int:
    print("BODY_NONE", value)
    return 0

count_fn: Any = take_count
bool_fn: Any = take_bool
float_fn: Any = take_float
str_fn: Any = take_str
blob_fn: Any = take_blob
none_fn: Any = take_none

count_fn(True)
bool_fn(True)
float_fn(True)
float_fn(1)
str_fn("ok")
blob_fn(b"ok")
none_fn(None)

try:
    count_fn("wrong")
except TypeError:
    print("REJECT_COUNT")
try:
    bool_fn(1)
except TypeError:
    print("REJECT_BOOL")
try:
    float_fn("wrong")
except TypeError:
    print("REJECT_FLOAT")
try:
    str_fn(1)
except TypeError:
    print("REJECT_STR")
try:
    blob_fn("wrong")
except TypeError:
    print("REJECT_BLOB")
try:
    none_fn(1)
except TypeError:
    print("REJECT_NONE")

def loose(value) -> int:
    print("BODY_LOOSE")
    return 0

loose(1)
loose_fn: Any = loose
try:
    loose_fn("wrong")
except TypeError:
    print("BAD_REJECT_LOOSE")

def take_any(value: Any) -> int:
    print("BODY_ANY")
    return 0

any_fn: Any = take_any
try:
    any_fn("wrong")
except TypeError:
    print("BAD_REJECT_ANY")

def take_sequence(value: list[int]) -> int:
    print("BODY_SEQUENCE")
    return 0

sequence_fn: Any = take_sequence
try:
    sequence_fn("wrong")
except TypeError:
    print("BAD_REJECT_SEQUENCE")

def take_variadic(*values: int, **named: int) -> int:
    print("BODY_VARIADIC")
    return 0

variadic_fn: Any = take_variadic
try:
    variadic_fn("wrong", named="wrong")
except TypeError:
    print("BAD_REJECT_VARIADIC")
"#,
    );
    assert!(result.is_ok(), "unexpected session error: {result:?}\n{output}");
    assert!(!output.contains("BAD_REJECT"), "fail-open annotation rejected:\n{output}");
    assert_eq!(line_count(&output, "BODY_COUNT 1"), 1, "{output}");
    assert_eq!(line_count(&output, "BODY_BOOL True"), 1, "{output}");
    assert_eq!(line_count(&output, "BODY_FLOAT 1.0"), 2, "{output}");
    assert_eq!(line_count(&output, "BODY_STR ok"), 1, "{output}");
    assert_eq!(line_count(&output, "BODY_BLOB b'ok'"), 1, "{output}");
    assert_eq!(line_count(&output, "BODY_NONE None"), 1, "{output}");
    assert_eq!(line_count(&output, "BODY_LOOSE"), 2, "{output}");
    assert_eq!(line_count(&output, "BODY_ANY"), 1, "{output}");
    assert_eq!(line_count(&output, "BODY_SEQUENCE"), 1, "{output}");
    assert_eq!(line_count(&output, "BODY_VARIADIC"), 1, "{output}");
    assert_eq!(
        output
            .lines()
            .filter(|line| line.starts_with("REJECT_"))
            .count(),
        6,
        "{output}"
    );
}

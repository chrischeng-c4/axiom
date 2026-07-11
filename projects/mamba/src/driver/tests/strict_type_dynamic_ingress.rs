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

def take_variadic(*values: list[int], **named: list[int]) -> int:
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

#[test]
fn dynamic_variadic_elements_reject_before_body_and_preserve_packed_values() {
    let (result, output) = run(
        r#"
from typing import Any

type Count = int

def take_ints(*values: Count) -> int:
    print("INTS", len(values), values[0], values[1])
    return 0

def take_bools(*values: bool) -> int:
    print("BOOLS", len(values), values[0])
    return 0

def take_floats(*values: float) -> int:
    print("FLOATS", len(values), values[0], values[1])
    return 0

def take_strs(*values: str) -> int:
    print("STRS", len(values), values[0])
    return 0

def take_bytes(*values: bytes) -> int:
    print("BYTES", len(values), values[0])
    return 0

def take_nones(*values: None) -> int:
    print("NONES", len(values), values[0])
    return 0

def take_named(**values: Count) -> int:
    print("NAMED", len(values), values["plain"], values["spread"])
    return 0

def take_empty(*values: int, **named: int) -> int:
    print("EMPTY", len(values), len(named))
    return 0

ints: Any = take_ints
bools: Any = take_bools
floats: Any = take_floats
strs: Any = take_strs
blobs: Any = take_bytes
nones: Any = take_nones
named: Any = take_named
empty: Any = take_empty

empty()
ints(True, 2)
ints(*[3, 4])
bools(True)
floats(1, True)
strs("ok")
blobs(b"ok")
nones(None)
named(plain=True, **{"spread": 2})

def reject_alias(*values: Count) -> int:
    print("BAD_ALIAS")
    return 0

def reject_values(*values: int) -> int:
    print("BAD_VALUES")
    return 0

def reject_named(**values: int) -> int:
    print("BAD_NAMED")
    return 0

bad_alias: Any = reject_alias
bad_values: Any = reject_values
bad_named: Any = reject_named
try:
    bad_alias("wrong")
except TypeError as exc:
    print("REJECT_ALIAS", str(exc))
try:
    bad_values(1, "wrong", 3)
except TypeError as exc:
    print("REJECT_VALUES", str(exc))
try:
    bad_values(*["wrong"])
except TypeError:
    print("REJECT_VALUES_STAR")
try:
    bad_named(first=1, wrong="wrong")
except TypeError as exc:
    print("REJECT_NAMED", str(exc))
try:
    bad_named(**{"wrong": "wrong"})
except TypeError:
    print("REJECT_NAMED_STAR")

def mixed(head: int, *values: int, flag: int = 7, **named: int) -> int:
    print("MIXED", head, values[0], flag, named["item"])
    return 0

mixed_fn: Any = mixed
mixed_fn(1, 2, flag=3, item=4)
try:
    mixed_fn(1, "wrong", flag=3, item=4)
except TypeError as exc:
    print("REJECT_MIXED_VALUES", str(exc))
try:
    mixed_fn(1, 2, flag=3, item="wrong")
except TypeError as exc:
    print("REJECT_MIXED_NAMED", str(exc))

def loose(*values, **named) -> int:
    print("LOOSE", len(values), len(named))
    return 0

def any_values(*values: Any, **named: Any) -> int:
    print("ANY", len(values), len(named))
    return 0

def generic_values(*values: list[int], **named: list[int]) -> int:
    print("GENERIC", len(values), len(named))
    return 0

loose_fn: Any = loose
any_fn: Any = any_values
generic_fn: Any = generic_values
loose_fn("wrong", named="wrong")
any_fn("wrong", named="wrong")
generic_fn("wrong", named="wrong")
"#,
    );
    assert!(result.is_ok(), "unexpected session error: {result:?}\n{output}");
    assert!(!output.contains("BAD_"), "rejected body executed:\n{output}");
    assert_eq!(line_count(&output, "INTS 2 True 2"), 1, "{output}");
    assert_eq!(line_count(&output, "INTS 2 3 4"), 1, "{output}");
    assert_eq!(line_count(&output, "BOOLS 1 True"), 1, "{output}");
    assert_eq!(line_count(&output, "FLOATS 2 1 True"), 1, "{output}");
    assert_eq!(line_count(&output, "STRS 1 ok"), 1, "{output}");
    assert_eq!(line_count(&output, "BYTES 1 b'ok'"), 1, "{output}");
    assert_eq!(line_count(&output, "NONES 1 None"), 1, "{output}");
    assert_eq!(line_count(&output, "NAMED 2 True 2"), 1, "{output}");
    assert_eq!(line_count(&output, "EMPTY 0 0"), 1, "{output}");
    assert_eq!(line_count(&output, "MIXED 1 2 3 4"), 1, "{output}");
    assert_eq!(
        line_count(
            &output,
            "REJECT_ALIAS reject_alias() variadic argument 'values' at index 0 expected Count, got str"
        ),
        1,
        "{output}"
    );
    assert_eq!(
        line_count(
            &output,
            "REJECT_VALUES reject_values() variadic argument 'values' at index 1 expected int, got str"
        ),
        1,
        "{output}"
    );
    assert_eq!(
        line_count(
            &output,
            "REJECT_NAMED reject_named() variadic argument 'values' at key 'wrong' expected int, got str"
        ),
        1,
        "{output}"
    );
    assert_eq!(line_count(&output, "REJECT_VALUES_STAR"), 1, "{output}");
    assert_eq!(line_count(&output, "REJECT_NAMED_STAR"), 1, "{output}");
    assert_eq!(
        line_count(
            &output,
            "REJECT_MIXED_VALUES mixed() variadic argument 'values' at index 0 expected int, got str"
        ),
        1,
        "{output}"
    );
    assert_eq!(
        line_count(
            &output,
            "REJECT_MIXED_NAMED mixed() variadic argument 'named' at key 'item' expected int, got str"
        ),
        1,
        "{output}"
    );
    assert_eq!(line_count(&output, "LOOSE 1 1"), 1, "{output}");
    assert_eq!(line_count(&output, "ANY 1 1"), 1, "{output}");
    assert_eq!(line_count(&output, "GENERIC 1 1"), 1, "{output}");
}

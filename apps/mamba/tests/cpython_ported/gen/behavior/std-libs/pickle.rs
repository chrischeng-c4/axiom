use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/pickle/container_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_pickle_container_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "behavior"
# case = "container_roundtrip"
# subject = "pickle.loads"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pickle.py"
# status = "filled"
# ///
"""pickle.loads: list/tuple/dict (incl. nested) round-trip through dumps+loads equal to the original, including a list-of-lists with mixed depth"""
import pickle

containers = [
    [1, 2, 3],
    (1, 2, 3),
    {"a": 1, "b": [2, 3]},
    [[1, 2], [3, [4, 5]]],
]
for c in containers:
    rt = pickle.loads(pickle.dumps(c))
    assert rt == c, f"container round-trip {type(c).__name__}: {rt!r}"

print("container_roundtrip OK")
"###);
    assert_output(&out, r###"container_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pickle/custom_class_default_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_pickle_custom_class_default_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "behavior"
# case = "custom_class_default_roundtrip"
# subject = "pickle.loads"
# kind = "semantic"
# xfail = "pickle shim serializes Instance objects to the 'N' sentinel; default user-class pickling is unsupported (src/runtime/stdlib/pickle_mod.rs:220)"
# mem_carveout = ""
# source = "Lib/test/test_pickle.py"
# status = "filled"
# ///
"""pickle.loads: a plain user class with instance attributes round-trips via default pickling: the reconstructed object is an instance of the class and compares equal by attributes"""
import pickle


class Point:
    def __init__(self, x, y):
        self.x = x
        self.y = y

    def __eq__(self, other):
        return isinstance(other, Point) and self.x == other.x and self.y == other.y


p = Point(3, 4)
rt = pickle.loads(pickle.dumps(p))
assert isinstance(rt, Point), f"custom class type = {type(rt)!r}"
assert rt == p, "custom class equality"

print("custom_class_default_roundtrip OK")
"###);
    assert_output(&out, r###"custom_class_default_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pickle/custom_reduce_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_pickle_custom_reduce_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "behavior"
# case = "custom_reduce_roundtrip"
# subject = "pickle.loads"
# kind = "semantic"
# xfail = "pickle shim does not consult __reduce__ and serializes Instance objects to the 'N' sentinel (src/runtime/stdlib/pickle_mod.rs:220)"
# mem_carveout = ""
# source = "Lib/test/test_pickle.py"
# status = "filled"
# ///
"""pickle.loads: a class defining __reduce__ controls its own reconstruction: the round-tripped instance equals the original via the reduce-provided constructor args"""
import pickle


class Custom:
    def __init__(self, val):
        self.val = val

    def __reduce__(self):
        return (Custom, (self.val,))

    def __eq__(self, other):
        return isinstance(other, Custom) and self.val == other.val


c = Custom(99)
rt = pickle.loads(pickle.dumps(c))
assert rt == c, f"custom __reduce__ round-trip = {rt.val!r}"

print("custom_reduce_roundtrip OK")
"###);
    assert_output(&out, r###"custom_reduce_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pickle/deep_nested_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_pickle_deep_nested_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "behavior"
# case = "deep_nested_roundtrip"
# subject = "pickle.loads"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pickle.py"
# status = "filled"
# ///
"""pickle.loads: a deeply nested dict-of-dict-of-list-of-dict round-trips through dumps+loads equal to the original"""
import pickle

nested = {"a": {"b": {"c": [1, 2, {"d": 3}]}}}
rt = pickle.loads(pickle.dumps(nested))
assert rt == nested, f"deep nested round-trip = {rt!r}"

print("deep_nested_roundtrip OK")
"###);
    assert_output(&out, r###"deep_nested_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pickle/dump_load_file_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_pickle_dump_load_file_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "behavior"
# case = "dump_load_file_roundtrip"
# subject = "pickle.dump"
# kind = "semantic"
# xfail = "pickle.dump is a stub that discards its output and pickle.load returns None (src/runtime/stdlib/pickle_mod.rs:346-353)"
# mem_carveout = ""
# source = "Lib/test/test_pickle.py"
# status = "filled"
# ///
"""pickle.dump: pickle.dump writes a value into a BytesIO file object and pickle.load reads it back from the rewound buffer equal to the original"""
import io
import pickle

data = {"key": [1, 2, 3], "value": "hello"}
buf = io.BytesIO()
pickle.dump(data, buf)
buf.seek(0)
loaded = pickle.load(buf)
assert loaded == data, f"dump/load file round-trip = {loaded!r}"

print("dump_load_file_roundtrip OK")
"###);
    assert_output(&out, r###"dump_load_file_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pickle/dumps_returns_bytes_with_protocol_header.py`.
#[test]
fn test_gen_behavior_std_libs_pickle_dumps_returns_bytes_with_protocol_header() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "behavior"
# case = "dumps_returns_bytes_with_protocol_header"
# subject = "pickle.dumps"
# kind = "semantic"
# xfail = "pickle shim emits a non-CPython text format with no b'\\x80' header byte (src/runtime/stdlib/pickle_mod.rs:318)"
# mem_carveout = ""
# source = "Lib/test/test_pickle.py"
# status = "filled"
# ///
"""pickle.dumps: pickle.dumps returns a bytes object whose first byte is the protocol-2+ opcode marker b'\\x80', and loads round-trips the value"""
import pickle

data = {"key": [1, 2, 3], "value": "hello"}
blob = pickle.dumps(data)
assert isinstance(blob, bytes), f"dumps type = {type(blob)!r}"
assert blob[0:1] == b"\x80", f"pickle header = {blob[0:1]!r}"
assert pickle.loads(blob) == data, "round-trip through dumps/loads"

print("dumps_returns_bytes_with_protocol_header OK")
"###);
    assert_output(&out, r###"dumps_returns_bytes_with_protocol_header OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pickle/error_class_hierarchy.py`.
#[test]
fn test_gen_behavior_std_libs_pickle_error_class_hierarchy() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "behavior"
# case = "error_class_hierarchy"
# subject = "pickle.PickleError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pickle.py"
# status = "filled"
# ///
"""pickle.PickleError: PicklingError and UnpicklingError are both subclasses of pickle.PickleError"""
import pickle

assert issubclass(pickle.PicklingError, pickle.PickleError), "PicklingError <: PickleError"
assert issubclass(pickle.UnpicklingError, pickle.PickleError), "UnpicklingError <: PickleError"

print("error_class_hierarchy OK")
"###);
    assert_output(&out, r###"error_class_hierarchy OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pickle/frozenset_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_pickle_frozenset_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "behavior"
# case = "frozenset_roundtrip"
# subject = "pickle.loads"
# kind = "semantic"
# xfail = "pickle shim has no set/frozenset serialization branch; sets serialize to the 'N' sentinel (src/runtime/stdlib/pickle_mod.rs:220)"
# mem_carveout = ""
# source = "Lib/test/test_pickle.py"
# status = "filled"
# ///
"""pickle.loads: a frozenset round-trips through dumps+loads equal to the original frozenset"""
import pickle

fs = frozenset([1, 2, 3])
rt = pickle.loads(pickle.dumps(fs))
assert rt == fs, f"frozenset round-trip = {rt!r}"
assert isinstance(rt, frozenset), f"frozenset type preserved = {type(rt)!r}"

print("frozenset_roundtrip OK")
"###);
    assert_output(&out, r###"frozenset_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pickle/pickler_unpickler_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_pickle_pickler_unpickler_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "behavior"
# case = "pickler_unpickler_roundtrip"
# subject = "pickle.Pickler"
# kind = "semantic"
# xfail = "pickle.Pickler/Unpickler are class shells; construction is out of scope (src/runtime/stdlib/pickle_mod.rs:50-54)"
# mem_carveout = ""
# source = "Lib/test/test_pickle.py"
# status = "filled"
# ///
"""pickle.Pickler: the streaming API round-trips a dict: a Pickler over a BytesIO writes via dump, and an Unpickler over the rewound buffer reconstructs it via load equal to the original"""
import io
import pickle

buf = io.BytesIO()
pickler = pickle.Pickler(buf)
assert hasattr(pickler, "dump"), "Pickler has dump"
pickler.dump({"x": 1})

buf.seek(0)
unpickler = pickle.Unpickler(buf)
assert hasattr(unpickler, "load"), "Unpickler has load"
assert unpickler.load() == {"x": 1}, "Pickler/Unpickler round-trip"

print("pickler_unpickler_roundtrip OK")
"###);
    assert_output(&out, r###"pickler_unpickler_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pickle/primitive_roundtrip_preserves_type.py`.
#[test]
fn test_gen_behavior_std_libs_pickle_primitive_roundtrip_preserves_type() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "behavior"
# case = "primitive_roundtrip_preserves_type"
# subject = "pickle.loads"
# kind = "semantic"
# xfail = "pickle shim has no bytes serialization branch; bytes serialize to the 'N' sentinel (src/runtime/stdlib/pickle_mod.rs:220)"
# mem_carveout = ""
# source = "Lib/test/test_pickle.py"
# status = "filled"
# ///
"""pickle.loads: every primitive (int/float/bool/None/str including unicode/bytes including empty) round-trips through dumps+loads equal to itself and with its exact type preserved"""
import pickle

primitives = [
    42, -1, 0, 3.14, -2.5, 1e100,
    True, False, None,
    "hello", "", "unicode: 中文",
    b"bytes", b"",
]
for v in primitives:
    rt = pickle.loads(pickle.dumps(v))
    assert rt == v, f"prim round-trip {type(v).__name__}: {v!r}"
    assert type(rt) == type(v), f"prim type preserved for {v!r}: {type(rt)!r}"

print("primitive_roundtrip_preserves_type OK")
"###);
    assert_output(&out, r###"primitive_roundtrip_preserves_type OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pickle/protocol_selection_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_pickle_protocol_selection_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "behavior"
# case = "protocol_selection_roundtrip"
# subject = "pickle.dumps"
# kind = "semantic"
# xfail = "pickle shim ignores the protocol kwarg and emits one fixed text format (src/runtime/stdlib/pickle_mod.rs:318)"
# mem_carveout = ""
# source = "Lib/test/test_pickle.py"
# status = "filled"
# ///
"""pickle.dumps: dumping with an explicit protocol (0 ASCII and 2 binary) and loading back reconstructs the original dict equal to itself"""
import pickle

data = {"test": [1, 2, 3]}
for proto in (0, 2):
    blob = pickle.dumps(data, protocol=proto)
    assert isinstance(blob, bytes), f"protocol {proto} returns bytes"
    assert pickle.loads(blob) == data, f"protocol {proto} round-trip"

print("protocol_selection_roundtrip OK")
"###);
    assert_output(&out, r###"protocol_selection_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pickle/shared_object_memo.py`.
#[test]
fn test_gen_behavior_std_libs_pickle_shared_object_memo() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "behavior"
# case = "shared_object_memo"
# subject = "pickle.loads"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pickle.py"
# status = "filled"
# ///
"""pickle.loads: a structure that references the same list object twice survives a dumps+loads round-trip: both positions reconstruct equal to the shared value"""
import pickle

shared = [1, 2, 3]
container = [shared, shared]
rt = pickle.loads(pickle.dumps(container))
# Both positions must reconstruct equal to the shared value; CPython's memo
# additionally makes them identical, but equality is the portable contract.
assert rt[0] == [1, 2, 3], f"shared[0] = {rt[0]!r}"
assert rt[1] == [1, 2, 3], f"shared[1] = {rt[1]!r}"
assert rt[0] is rt[1], "memo preserves shared identity across both positions"

print("shared_object_memo OK")
"###);
    assert_output(&out, r###"shared_object_memo OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pickle/tuple_type_preserved.py`.
#[test]
fn test_gen_behavior_std_libs_pickle_tuple_type_preserved() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "behavior"
# case = "tuple_type_preserved"
# subject = "pickle.loads"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pickle.py"
# status = "filled"
# ///
"""pickle.loads: a tuple round-trips as a tuple (not a list): pickle.loads(pickle.dumps((1,'a',True))) is a tuple and equals the original"""
import pickle

t = (1, "a", True)
rt = pickle.loads(pickle.dumps(t))
assert isinstance(rt, tuple), f"tuple preserved = {type(rt)!r}"
assert rt == t, f"tuple equality = {rt!r}"

print("tuple_type_preserved OK")
"###);
    assert_output(&out, r###"tuple_type_preserved OK
"###);
}

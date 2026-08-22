use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/uuid/bytes_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_uuid_bytes_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "bytes_roundtrip"
# subject = "uuid.UUID"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.UUID: UUID(bytes=u.bytes) reconstructs an equal UUID (16-byte round-trip)"""
import uuid

u = uuid.uuid4()
assert uuid.UUID(bytes=u.bytes) == u, "UUID bytes round-trip"
print("bytes_roundtrip OK")
"###);
    assert_output(&out, r###"bytes_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/uuid/copy_and_deepcopy_preserve_equality.py`.
#[test]
fn test_gen_behavior_std_libs_uuid_copy_and_deepcopy_preserve_equality() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "copy_and_deepcopy_preserve_equality"
# subject = "uuid.UUID"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.UUID: copy.copy and copy.deepcopy of a UUID both compare equal to the original"""
import copy
import uuid

canon = uuid.UUID("12345678-1234-5678-1234-567812345678")
assert copy.copy(canon) == canon, "copy equal"
assert copy.deepcopy(canon) == canon, "deepcopy equal"
print("copy_and_deepcopy_preserve_equality OK")
"###);
    assert_output(&out, r###"copy_and_deepcopy_preserve_equality OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/uuid/equal_uuids_hash_and_dedup.py`.
#[test]
fn test_gen_behavior_std_libs_uuid_equal_uuids_hash_and_dedup() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "equal_uuids_hash_and_dedup"
# subject = "uuid.UUID"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.UUID: two UUIDs built from the same string hash equal and collapse to one element in a set"""
import uuid

canon = uuid.UUID("12345678-1234-5678-1234-567812345678")
dup = uuid.UUID("12345678-1234-5678-1234-567812345678")
assert hash(dup) == hash(canon), "equal UUIDs hash equal"
assert len({canon, dup}) == 1, "set dedup"
print("equal_uuids_hash_and_dedup OK")
"###);
    assert_output(&out, r###"equal_uuids_hash_and_dedup OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/uuid/fields_decompose_128_bit.py`.
#[test]
fn test_gen_behavior_std_libs_uuid_fields_decompose_128_bit() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "fields_decompose_128_bit"
# subject = "uuid.UUID"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.UUID: UUID('12345678-1234-5678-1234-567812345678').fields decomposes into (time_low, time_mid, time_hi_version, clock_seq_hi_variant, clock_seq_low, node) matching each named accessor"""
import uuid

u = uuid.UUID("12345678-1234-5678-1234-567812345678")
assert u.fields == (0x12345678, 0x1234, 0x5678, 0x12, 0x34, 0x567812345678), \
    f"fields = {u.fields!r}"
assert u.time_low == 0x12345678, f"time_low = {u.time_low!r}"
assert u.time_mid == 0x1234, f"time_mid = {u.time_mid!r}"
assert u.time_hi_version == 0x5678, f"time_hi_version = {u.time_hi_version!r}"
assert u.clock_seq_hi_variant == 0x12, f"clock_seq_hi_variant = {u.clock_seq_hi_variant!r}"
assert u.clock_seq_low == 0x34, f"clock_seq_low = {u.clock_seq_low!r}"
assert u.node == 0x567812345678, f"node = {u.node!r}"
print("fields_decompose_128_bit OK")
"###);
    assert_output(&out, r###"fields_decompose_128_bit OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/uuid/getnode_is_stable_48_bit.py`.
#[test]
fn test_gen_behavior_std_libs_uuid_getnode_is_stable_48_bit() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "getnode_is_stable_48_bit"
# subject = "uuid.getnode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.getnode: getnode() returns a stable 48-bit unsigned node id (repeat calls equal) that feeds uuid1().node within range"""
import uuid

node = uuid.getnode()
assert isinstance(node, int), f"node type = {type(node)!r}"
assert 0 < node < (1 << 48), f"node out of 48-bit range: {node!r}"
assert uuid.getnode() == node, "getnode not stable across calls"

u = uuid.uuid1()
assert 0 < u.node < (1 << 48), f"uuid1 node out of range: {u.node!r}"
print("getnode_is_stable_48_bit OK")
"###);
    assert_output(&out, r###"getnode_is_stable_48_bit OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/uuid/hex_int_bytes_urn_shapes.py`.
#[test]
fn test_gen_behavior_std_libs_uuid_hex_int_bytes_urn_shapes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "hex_int_bytes_urn_shapes"
# subject = "uuid.UUID"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.UUID: a v4 UUID exposes .hex (32 chars), .int (positive int), .bytes (16 bytes), and .urn ('urn:uuid:' prefix)"""
import uuid

u = uuid.uuid4()
assert isinstance(u.hex, str) and len(u.hex) == 32, f"hex = {u.hex!r}"
assert isinstance(u.int, int) and u.int > 0, f"int = {u.int!r}"
assert isinstance(u.bytes, bytes) and len(u.bytes) == 16, f"bytes len = {len(u.bytes)!r}"
assert u.urn.startswith("urn:uuid:"), f"urn = {u.urn!r}"
print("hex_int_bytes_urn_shapes OK")
"###);
    assert_output(&out, r###"hex_int_bytes_urn_shapes OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/uuid/hex_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_uuid_hex_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "hex_roundtrip"
# subject = "uuid.UUID"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.UUID: UUID(hex=u.hex) reconstructs an equal UUID (32-char hex round-trip)"""
import uuid

u = uuid.uuid4()
assert uuid.UUID(hex=u.hex) == u, "UUID hex round-trip"
print("hex_roundtrip OK")
"###);
    assert_output(&out, r###"hex_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/uuid/int_matches_hex_value.py`.
#[test]
fn test_gen_behavior_std_libs_uuid_int_matches_hex_value() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "int_matches_hex_value"
# subject = "uuid.UUID"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.UUID: UUID.int equals int(UUID.hex, 16) for a fixed UUID"""
import uuid

u = uuid.UUID("550e8400-e29b-41d4-a716-446655440000")
assert u.int == int(u.hex, 16), f"int {u.int!r} != int(hex, 16) {int(u.hex, 16)!r}"
print("int_matches_hex_value OK")
"###);
    assert_output(&out, r###"int_matches_hex_value OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/uuid/int_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_uuid_int_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "int_roundtrip"
# subject = "uuid.UUID"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.UUID: UUID(int=u.int) reconstructs an equal UUID (128-bit int round-trip)"""
import uuid

u = uuid.uuid4()
assert uuid.UUID(int=u.int) == u, "UUID int round-trip"
print("int_roundtrip OK")
"###);
    assert_output(&out, r###"int_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/uuid/ordering_lexicographic.py`.
#[test]
fn test_gen_behavior_std_libs_uuid_ordering_lexicographic() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "ordering_lexicographic"
# subject = "uuid.UUID"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.UUID: UUIDs order by their 128-bit int: the all-zero-but-one value is less than the all-ones value"""
import uuid

small = uuid.UUID("00000000-0000-0000-0000-000000000001")
large = uuid.UUID("ffffffff-ffff-ffff-ffff-ffffffffffff")
assert small < large, "UUID ordering"
assert large > small, "UUID ordering (reverse)"
print("ordering_lexicographic OK")
"###);
    assert_output(&out, r###"ordering_lexicographic OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/uuid/repr_is_reconstructable.py`.
#[test]
fn test_gen_behavior_std_libs_uuid_repr_is_reconstructable() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "repr_is_reconstructable"
# subject = "uuid.UUID"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.UUID: repr(UUID('12345678-1234-5678-1234-567812345678')) is the reconstructable "UUID('12345678-1234-5678-1234-567812345678')" """
import uuid

u = uuid.UUID("12345678-1234-5678-1234-567812345678")
assert repr(u) == "UUID('12345678-1234-5678-1234-567812345678')", f"repr = {repr(u)!r}"
print("repr_is_reconstructable OK")
"###);
    assert_output(&out, r###"repr_is_reconstructable OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/uuid/safe_uuid_enum_members.py`.
#[test]
fn test_gen_behavior_std_libs_uuid_safe_uuid_enum_members() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "safe_uuid_enum_members"
# subject = "uuid.SafeUUID"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.SafeUUID: SafeUUID is an Enum with members safe=0, unsafe=-1, unknown=None; value lookup and identity hold, and a plain UUID reports is_safe is SafeUUID.unknown"""
import enum
import uuid

assert issubclass(uuid.SafeUUID, enum.Enum), "SafeUUID is not an Enum"
assert uuid.SafeUUID.safe.value == 0, f"safe = {uuid.SafeUUID.safe.value!r}"
assert uuid.SafeUUID.unsafe.value == -1, f"unsafe = {uuid.SafeUUID.unsafe.value!r}"
assert uuid.SafeUUID.unknown.value is None, f"unknown = {uuid.SafeUUID.unknown.value!r}"
assert uuid.SafeUUID.safe is not uuid.SafeUUID.unsafe, "members not distinct"
assert uuid.SafeUUID(0) is uuid.SafeUUID.safe, "lookup by value failed"

plain = uuid.UUID("12345678-1234-5678-1234-567812345678")
assert plain.is_safe is uuid.SafeUUID.unknown, f"plain.is_safe = {plain.is_safe!r}"
print("safe_uuid_enum_members OK")
"###);
    assert_output(&out, r###"safe_uuid_enum_members OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/uuid/string_form_8_4_4_4_12.py`.
#[test]
fn test_gen_behavior_std_libs_uuid_string_form_8_4_4_4_12() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "string_form_8_4_4_4_12"
# subject = "uuid.UUID"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.UUID: str(UUID) is the canonical 8-4-4-4-12 dash-grouped 36-char form"""
import uuid

s = str(uuid.uuid4())
assert len(s) == 36, f"str len = {len(s)!r}"
parts = s.split("-")
assert len(parts) == 5, f"UUID parts = {len(parts)!r}"
assert [len(p) for p in parts] == [8, 4, 4, 4, 12], f"part lens = {[len(p) for p in parts]!r}"
print("string_form_8_4_4_4_12 OK")
"###);
    assert_output(&out, r###"string_form_8_4_4_4_12 OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/uuid/string_form_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_uuid_string_form_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "string_form_roundtrip"
# subject = "uuid.UUID"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.UUID: UUID(str(u)) reconstructs an equal UUID (canonical-string round-trip)"""
import uuid

u = uuid.uuid4()
assert uuid.UUID(str(u)) == u, "UUID str round-trip"
print("string_form_roundtrip OK")
"###);
    assert_output(&out, r###"string_form_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/uuid/string_parsing_tolerates_forms.py`.
#[test]
fn test_gen_behavior_std_libs_uuid_string_parsing_tolerates_forms() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "string_parsing_tolerates_forms"
# subject = "uuid.UUID"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.UUID: the constructor accepts brace, urn:uuid:, dash-free, and upper-case spellings of one UUID, all equal to the canonical form"""
import uuid

canon = uuid.UUID("12345678-1234-5678-1234-567812345678")
assert uuid.UUID("{12345678-1234-5678-1234-567812345678}") == canon, "brace form"
assert uuid.UUID("urn:uuid:12345678-1234-5678-1234-567812345678") == canon, "urn form"
assert uuid.UUID("12345678123456781234567812345678") == canon, "no-dash form"
assert uuid.UUID("12345678-1234-5678-1234-567812345678".upper()) == canon, "upper case"
print("string_parsing_tolerates_forms OK")
"###);
    assert_output(&out, r###"string_parsing_tolerates_forms OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/uuid/test_uuid_with_ext_module__test_getnode.py`.
#[test]
fn test_gen_behavior_std_libs_uuid_test_uuid_with_ext_module__test_getnode() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "test_uuid_with_ext_module__test_getnode"
# subject = "cpython.test_uuid.TestUUIDWithExtModule.test_getnode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_uuid.py::TestUUIDWithExtModule::test_getnode
"""Auto-ported test: TestUUIDWithExtModule::test_getnode (CPython 3.12 oracle)."""


import unittest
from test import support
from test.support import import_helper
import builtins
import contextlib
import copy
import enum
import io
import os
import pickle
import sys
import weakref
from unittest import mock


py_uuid = import_helper.import_fresh_module('uuid', blocked=['_uuid'])

c_uuid = import_helper.import_fresh_module('uuid', fresh=['_uuid'])

def importable(name):
    try:
        __import__(name)
        return True
    except ModuleNotFoundError:
        return False

def mock_get_command_stdout(data):

    def get_command_stdout(command, args):
        return io.BytesIO(data.encode())
    return get_command_stdout


# --- test body ---
uuid = None
uuid = c_uuid
node1 = uuid.getnode()

assert 0 < node1 < 1 << 48
node2 = uuid.getnode()

assert node1 == node2
print("TestUUIDWithExtModule::test_getnode: ok")
"###);
    assert_output(&out, r###"TestUUIDWithExtModule::test_getnode: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/uuid/test_uuid_with_ext_module__test_uuid_weakref.py`.
#[test]
fn test_gen_behavior_std_libs_uuid_test_uuid_with_ext_module__test_uuid_weakref() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "test_uuid_with_ext_module__test_uuid_weakref"
# subject = "cpython.test_uuid.TestUUIDWithExtModule.test_uuid_weakref"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_uuid.py::TestUUIDWithExtModule::test_uuid_weakref
"""Auto-ported test: TestUUIDWithExtModule::test_uuid_weakref (CPython 3.12 oracle)."""


import unittest
from test import support
from test.support import import_helper
import builtins
import contextlib
import copy
import enum
import io
import os
import pickle
import sys
import weakref
from unittest import mock


py_uuid = import_helper.import_fresh_module('uuid', blocked=['_uuid'])

c_uuid = import_helper.import_fresh_module('uuid', fresh=['_uuid'])

def importable(name):
    try:
        __import__(name)
        return True
    except ModuleNotFoundError:
        return False

def mock_get_command_stdout(data):

    def get_command_stdout(command, args):
        return io.BytesIO(data.encode())
    return get_command_stdout


# --- test body ---
uuid = None
uuid = c_uuid
strong = uuid.uuid4()
weak = weakref.ref(strong)

assert strong is weak()
print("TestUUIDWithExtModule::test_uuid_weakref: ok")
"###);
    assert_output(&out, r###"TestUUIDWithExtModule::test_uuid_weakref: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/uuid/test_uuid_without_ext_module__test_getnode.py`.
#[test]
fn test_gen_behavior_std_libs_uuid_test_uuid_without_ext_module__test_getnode() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "test_uuid_without_ext_module__test_getnode"
# subject = "cpython.test_uuid.TestUUIDWithoutExtModule.test_getnode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_uuid.py::TestUUIDWithoutExtModule::test_getnode
"""Auto-ported test: TestUUIDWithoutExtModule::test_getnode (CPython 3.12 oracle)."""


import unittest
from test import support
from test.support import import_helper
import builtins
import contextlib
import copy
import enum
import io
import os
import pickle
import sys
import weakref
from unittest import mock


py_uuid = import_helper.import_fresh_module('uuid', blocked=['_uuid'])

c_uuid = import_helper.import_fresh_module('uuid', fresh=['_uuid'])

def importable(name):
    try:
        __import__(name)
        return True
    except ModuleNotFoundError:
        return False

def mock_get_command_stdout(data):

    def get_command_stdout(command, args):
        return io.BytesIO(data.encode())
    return get_command_stdout


# --- test body ---
uuid = None
uuid = py_uuid
node1 = uuid.getnode()

assert 0 < node1 < 1 << 48
node2 = uuid.getnode()

assert node1 == node2
print("TestUUIDWithoutExtModule::test_getnode: ok")
"###);
    assert_output(&out, r###"TestUUIDWithoutExtModule::test_getnode: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/uuid/test_uuid_without_ext_module__test_uuid_weakref.py`.
#[test]
fn test_gen_behavior_std_libs_uuid_test_uuid_without_ext_module__test_uuid_weakref() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "test_uuid_without_ext_module__test_uuid_weakref"
# subject = "cpython.test_uuid.TestUUIDWithoutExtModule.test_uuid_weakref"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_uuid.py::TestUUIDWithoutExtModule::test_uuid_weakref
"""Auto-ported test: TestUUIDWithoutExtModule::test_uuid_weakref (CPython 3.12 oracle)."""


import unittest
from test import support
from test.support import import_helper
import builtins
import contextlib
import copy
import enum
import io
import os
import pickle
import sys
import weakref
from unittest import mock


py_uuid = import_helper.import_fresh_module('uuid', blocked=['_uuid'])

c_uuid = import_helper.import_fresh_module('uuid', fresh=['_uuid'])

def importable(name):
    try:
        __import__(name)
        return True
    except ModuleNotFoundError:
        return False

def mock_get_command_stdout(data):

    def get_command_stdout(command, args):
        return io.BytesIO(data.encode())
    return get_command_stdout


# --- test body ---
uuid = None
uuid = py_uuid
strong = uuid.uuid4()
weak = weakref.ref(strong)

assert strong is weak()
print("TestUUIDWithoutExtModule::test_uuid_weakref: ok")
"###);
    assert_output(&out, r###"TestUUIDWithoutExtModule::test_uuid_weakref: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/uuid/urn_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_uuid_urn_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "urn_roundtrip"
# subject = "uuid.UUID"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.UUID: UUID(u.urn) reconstructs an equal UUID (urn:uuid: round-trip through the constructor)"""
import uuid

u = uuid.UUID("12345678-1234-5678-1234-567812345678")
assert uuid.UUID(u.urn) == u, "urn round-trip"
print("urn_roundtrip OK")
"###);
    assert_output(&out, r###"urn_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/uuid/uuid1_default_is_version_1.py`.
#[test]
fn test_gen_behavior_std_libs_uuid_uuid1_default_is_version_1() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "uuid1_default_is_version_1"
# subject = "uuid.uuid1"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.uuid1: uuid1() with no args is still a version-1 RFC 4122 UUID and accepts the live getnode() value without raising"""
import uuid

node = uuid.getnode()
assert 0 < node < (1 << 48), f"node out of range: {node!r}"
try:
    uuid.uuid1(node=node)
except ValueError as e:
    raise AssertionError(f"uuid1 rejected a valid node: {e}")

d = uuid.uuid1()
assert d.version == 1, f"default version = {d.version!r}"
assert d.variant == uuid.RFC_4122, f"default variant = {d.variant!r}"
print("uuid1_default_is_version_1 OK")
"###);
    assert_output(&out, r###"uuid1_default_is_version_1 OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/uuid/uuid1_node_clock_seq_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_uuid_uuid1_node_clock_seq_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "uuid1_node_clock_seq_roundtrip"
# subject = "uuid.uuid1"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.uuid1: uuid1(node, clock_seq) with explicit values round-trips both fields, reports version 1, RFC 4122 variant, and a positive 60-bit time"""
import uuid

NODE = 93328246233727       # a valid 48-bit EUI-64 node
CLOCK_SEQ = 5317            # 14-bit clock sequence

u = uuid.uuid1(node=NODE, clock_seq=CLOCK_SEQ)
assert u.version == 1, f"version = {u.version!r}"
assert u.node == NODE, f"node = {u.node!r}"
assert u.clock_seq == CLOCK_SEQ, f"clock_seq = {u.clock_seq!r}"
assert u.variant == uuid.RFC_4122, f"variant = {u.variant!r}"
assert isinstance(u.time, int) and u.time > 0, f"time = {u.time!r}"
print("uuid1_node_clock_seq_roundtrip OK")
"###);
    assert_output(&out, r###"uuid1_node_clock_seq_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/uuid/uuid3_is_deterministic_md5.py`.
#[test]
fn test_gen_behavior_std_libs_uuid_uuid3_is_deterministic_md5() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "uuid3_is_deterministic_md5"
# subject = "uuid.uuid3"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.uuid3: uuid3(NAMESPACE_URL, 'http://test.com') is MD5-deterministic (two calls equal) and reports version 3"""
import uuid

a = uuid.uuid3(uuid.NAMESPACE_URL, "http://test.com")
b = uuid.uuid3(uuid.NAMESPACE_URL, "http://test.com")
assert a == b, f"uuid3 not deterministic: {a} vs {b}"
assert a.version == 3, f"uuid3 version = {a.version!r}"
print("uuid3_is_deterministic_md5 OK")
"###);
    assert_output(&out, r###"uuid3_is_deterministic_md5 OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/uuid/uuid4_is_version_4_random.py`.
#[test]
fn test_gen_behavior_std_libs_uuid_uuid4_is_version_4_random() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "uuid4_is_version_4_random"
# subject = "uuid.uuid4"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.uuid4: uuid4() returns a version-4 UUID with the RFC 4122 variant, and two successive calls differ (random)"""
import uuid

a = uuid.uuid4()
b = uuid.uuid4()
assert isinstance(a, uuid.UUID), f"uuid4 type = {type(a)!r}"
assert a.version == 4, f"uuid4 version = {a.version!r}"
assert a.variant == uuid.RFC_4122, f"uuid4 variant = {a.variant!r}"
assert a != b, f"uuid4 not unique: {a} vs {b}"
print("uuid4_is_version_4_random OK")
"###);
    assert_output(&out, r###"uuid4_is_version_4_random OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/uuid/uuid5_is_deterministic.py`.
#[test]
fn test_gen_behavior_std_libs_uuid_uuid5_is_deterministic() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "uuid5_is_deterministic"
# subject = "uuid.uuid5"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.uuid5: uuid5(NAMESPACE_DNS, 'example.com') is deterministic: two calls with the same name yield equal version-5 UUIDs"""
import uuid

a = uuid.uuid5(uuid.NAMESPACE_DNS, "example.com")
b = uuid.uuid5(uuid.NAMESPACE_DNS, "example.com")
assert isinstance(a, uuid.UUID), f"uuid5 type = {type(a)!r}"
assert a.version == 5, f"uuid5 version = {a.version!r}"
assert a == b, f"uuid5 not deterministic: {a} vs {b}"
print("uuid5_is_deterministic OK")
"###);
    assert_output(&out, r###"uuid5_is_deterministic OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/uuid/uuid5_python_org_rfc_vector.py`.
#[test]
fn test_gen_behavior_std_libs_uuid_uuid5_python_org_rfc_vector() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "uuid5_python_org_rfc_vector"
# subject = "uuid.uuid5"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.uuid5: uuid5(NAMESPACE_DNS, 'python.org') equals the known SHA-1 vector '886313e1-3b8a-5372-9b90-0c9aee199e5d'"""
import uuid

known = uuid.uuid5(uuid.NAMESPACE_DNS, "python.org")
assert str(known) == "886313e1-3b8a-5372-9b90-0c9aee199e5d", \
    f"uuid5 known value = {str(known)!r}"
assert known.version == 5, f"uuid5 version = {known.version!r}"
print("uuid5_python_org_rfc_vector OK")
"###);
    assert_output(&out, r###"uuid5_python_org_rfc_vector OK
"###);
}

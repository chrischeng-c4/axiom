use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/uuid/UUID____ge____other_as_UUID_wrong.py`.
#[test]
fn test_gen_type_std_libs_uuid_UUID____ge____other_as_UUID_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "type"
# case = "UUID____ge____other_as_UUID_wrong"
# subject = "uuid.UUID.__ge__(other: UUID)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/uuid.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: uuid.UUID.__ge__(other: UUID); call it with the wrong type.

typeshed contract: other is UUID. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from uuid import UUID
obj = object.__new__(UUID)
try:
    obj.__ge__(_W())  # other: UUID <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/uuid/UUID____gt____other_as_UUID_wrong.py`.
#[test]
fn test_gen_type_std_libs_uuid_UUID____gt____other_as_UUID_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "type"
# case = "UUID____gt____other_as_UUID_wrong"
# subject = "uuid.UUID.__gt__(other: UUID)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/uuid.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: uuid.UUID.__gt__(other: UUID); call it with the wrong type.

typeshed contract: other is UUID. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from uuid import UUID
obj = object.__new__(UUID)
try:
    obj.__gt__(_W())  # other: UUID <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/uuid/UUID____le____other_as_UUID_wrong.py`.
#[test]
fn test_gen_type_std_libs_uuid_UUID____le____other_as_UUID_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "type"
# case = "UUID____le____other_as_UUID_wrong"
# subject = "uuid.UUID.__le__(other: UUID)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/uuid.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: uuid.UUID.__le__(other: UUID); call it with the wrong type.

typeshed contract: other is UUID. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from uuid import UUID
obj = object.__new__(UUID)
try:
    obj.__le__(_W())  # other: UUID <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/uuid/UUID____lt____other_as_UUID_wrong.py`.
#[test]
fn test_gen_type_std_libs_uuid_UUID____lt____other_as_UUID_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "type"
# case = "UUID____lt____other_as_UUID_wrong"
# subject = "uuid.UUID.__lt__(other: UUID)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/uuid.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: uuid.UUID.__lt__(other: UUID); call it with the wrong type.

typeshed contract: other is UUID. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from uuid import UUID
obj = object.__new__(UUID)
try:
    obj.__lt__(_W())  # other: UUID <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/uuid/UUID__init__hex_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_uuid_UUID__init__hex_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "type"
# case = "UUID__init__hex_as_typed_wrong"
# subject = "uuid.UUID.__init__(hex: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/uuid.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: uuid.UUID.__init__(hex: typed); call it with the wrong type.

typeshed contract: hex is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from uuid import UUID
try:
    UUID(_W())  # hex: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/uuid/uuid1__node_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_uuid_uuid1__node_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "type"
# case = "uuid1__node_as_typed_wrong"
# subject = "uuid.uuid1(node: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/uuid.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: uuid.uuid1(node: typed); call it with the wrong type.

typeshed contract: node is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from uuid import uuid1
try:
    uuid1(_W())  # node: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/uuid/uuid3__namespace_as_UUID_wrong.py`.
#[test]
fn test_gen_type_std_libs_uuid_uuid3__namespace_as_UUID_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "type"
# case = "uuid3__namespace_as_UUID_wrong"
# subject = "uuid.uuid3(namespace: UUID)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/uuid.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: uuid.uuid3(namespace: UUID); call it with the wrong type.

typeshed contract: namespace is UUID. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from uuid import uuid3
try:
    uuid3(_W(), None)  # namespace: UUID <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/uuid/uuid5__namespace_as_UUID_wrong.py`.
#[test]
fn test_gen_type_std_libs_uuid_uuid5__namespace_as_UUID_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "type"
# case = "uuid5__namespace_as_UUID_wrong"
# subject = "uuid.uuid5(namespace: UUID)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/uuid.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: uuid.uuid5(namespace: UUID); call it with the wrong type.

typeshed contract: namespace is UUID. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from uuid import uuid5
try:
    uuid5(_W(), None)  # namespace: UUID <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/uuid/uuid6__node_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_uuid_uuid6__node_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "type"
# case = "uuid6__node_as_typed_wrong"
# subject = "uuid.uuid6(node: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/uuid.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: uuid.uuid6(node: typed); call it with the wrong type.

typeshed contract: node is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from uuid import uuid6
try:
    uuid6(_W())  # node: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/uuid/uuid8__a_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_uuid_uuid8__a_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "type"
# case = "uuid8__a_as_typed_wrong"
# subject = "uuid.uuid8(a: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/uuid.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: uuid.uuid8(a: typed); call it with the wrong type.

typeshed contract: a is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from uuid import uuid8
try:
    uuid8(_W())  # a: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

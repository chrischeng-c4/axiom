use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/_interpchannels/ChannelID____ge____other_as_ChannelID_wrong.py`.
#[test]
fn test_gen_type_std_libs__interpchannels_ChannelID____ge____other_as_ChannelID_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_interpchannels"
# dimension = "type"
# case = "ChannelID____ge____other_as_ChannelID_wrong"
# subject = "_interpchannels.ChannelID.__ge__(other: ChannelID)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_interpchannels.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _interpchannels.ChannelID.__ge__(other: ChannelID); call it with the wrong type.

typeshed contract: other is ChannelID. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _interpchannels import ChannelID
obj = object.__new__(ChannelID)
try:
    obj.__ge__(_W())  # other: ChannelID <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_interpchannels/ChannelID____gt____other_as_ChannelID_wrong.py`.
#[test]
fn test_gen_type_std_libs__interpchannels_ChannelID____gt____other_as_ChannelID_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_interpchannels"
# dimension = "type"
# case = "ChannelID____gt____other_as_ChannelID_wrong"
# subject = "_interpchannels.ChannelID.__gt__(other: ChannelID)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_interpchannels.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _interpchannels.ChannelID.__gt__(other: ChannelID); call it with the wrong type.

typeshed contract: other is ChannelID. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _interpchannels import ChannelID
obj = object.__new__(ChannelID)
try:
    obj.__gt__(_W())  # other: ChannelID <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_interpchannels/ChannelID____le____other_as_ChannelID_wrong.py`.
#[test]
fn test_gen_type_std_libs__interpchannels_ChannelID____le____other_as_ChannelID_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_interpchannels"
# dimension = "type"
# case = "ChannelID____le____other_as_ChannelID_wrong"
# subject = "_interpchannels.ChannelID.__le__(other: ChannelID)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_interpchannels.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _interpchannels.ChannelID.__le__(other: ChannelID); call it with the wrong type.

typeshed contract: other is ChannelID. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _interpchannels import ChannelID
obj = object.__new__(ChannelID)
try:
    obj.__le__(_W())  # other: ChannelID <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_interpchannels/ChannelID____lt____other_as_ChannelID_wrong.py`.
#[test]
fn test_gen_type_std_libs__interpchannels_ChannelID____lt____other_as_ChannelID_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_interpchannels"
# dimension = "type"
# case = "ChannelID____lt____other_as_ChannelID_wrong"
# subject = "_interpchannels.ChannelID.__lt__(other: ChannelID)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_interpchannels.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _interpchannels.ChannelID.__lt__(other: ChannelID); call it with the wrong type.

typeshed contract: other is ChannelID. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _interpchannels import ChannelID
obj = object.__new__(ChannelID)
try:
    obj.__lt__(_W())  # other: ChannelID <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_interpchannels/close__cid_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs__interpchannels_close__cid_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_interpchannels"
# dimension = "type"
# case = "close__cid_as_SupportsIndex_wrong"
# subject = "_interpchannels.close(cid: SupportsIndex)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_interpchannels.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _interpchannels.close(cid: SupportsIndex); call it with the wrong type.

typeshed contract: cid is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _interpchannels import close
try:
    close(_W())  # cid: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_interpchannels/create__unboundop_as_Literal_wrong.py`.
#[test]
fn test_gen_type_std_libs__interpchannels_create__unboundop_as_Literal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_interpchannels"
# dimension = "type"
# case = "create__unboundop_as_Literal_wrong"
# subject = "_interpchannels.create(unboundop: Literal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_interpchannels.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _interpchannels.create(unboundop: Literal); call it with the wrong type.

typeshed contract: unboundop is Literal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _interpchannels import create
try:
    create(_W())  # unboundop: Literal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_interpchannels/destroy__cid_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs__interpchannels_destroy__cid_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_interpchannels"
# dimension = "type"
# case = "destroy__cid_as_SupportsIndex_wrong"
# subject = "_interpchannels.destroy(cid: SupportsIndex)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_interpchannels.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _interpchannels.destroy(cid: SupportsIndex); call it with the wrong type.

typeshed contract: cid is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _interpchannels import destroy
try:
    destroy(_W())  # cid: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_interpchannels/get_channel_defaults__cid_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs__interpchannels_get_channel_defaults__cid_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_interpchannels"
# dimension = "type"
# case = "get_channel_defaults__cid_as_SupportsIndex_wrong"
# subject = "_interpchannels.get_channel_defaults(cid: SupportsIndex)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_interpchannels.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _interpchannels.get_channel_defaults(cid: SupportsIndex); call it with the wrong type.

typeshed contract: cid is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _interpchannels import get_channel_defaults
try:
    get_channel_defaults(_W())  # cid: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_interpchannels/get_count__cid_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs__interpchannels_get_count__cid_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_interpchannels"
# dimension = "type"
# case = "get_count__cid_as_SupportsIndex_wrong"
# subject = "_interpchannels.get_count(cid: SupportsIndex)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_interpchannels.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _interpchannels.get_count(cid: SupportsIndex); call it with the wrong type.

typeshed contract: cid is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _interpchannels import get_count
try:
    get_count(_W())  # cid: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_interpchannels/get_info__cid_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs__interpchannels_get_info__cid_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_interpchannels"
# dimension = "type"
# case = "get_info__cid_as_SupportsIndex_wrong"
# subject = "_interpchannels.get_info(cid: SupportsIndex)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_interpchannels.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _interpchannels.get_info(cid: SupportsIndex); call it with the wrong type.

typeshed contract: cid is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _interpchannels import get_info
try:
    get_info(_W())  # cid: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_interpchannels/list_interpreters__cid_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs__interpchannels_list_interpreters__cid_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_interpchannels"
# dimension = "type"
# case = "list_interpreters__cid_as_SupportsIndex_wrong"
# subject = "_interpchannels.list_interpreters(cid: SupportsIndex)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_interpchannels.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _interpchannels.list_interpreters(cid: SupportsIndex); call it with the wrong type.

typeshed contract: cid is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _interpchannels import list_interpreters
try:
    list_interpreters(_W())  # cid: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_interpchannels/recv__cid_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs__interpchannels_recv__cid_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_interpchannels"
# dimension = "type"
# case = "recv__cid_as_SupportsIndex_wrong"
# subject = "_interpchannels.recv(cid: SupportsIndex)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_interpchannels.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _interpchannels.recv(cid: SupportsIndex); call it with the wrong type.

typeshed contract: cid is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _interpchannels import recv
try:
    recv(_W())  # cid: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_interpchannels/release__cid_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs__interpchannels_release__cid_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_interpchannels"
# dimension = "type"
# case = "release__cid_as_SupportsIndex_wrong"
# subject = "_interpchannels.release(cid: SupportsIndex)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_interpchannels.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _interpchannels.release(cid: SupportsIndex); call it with the wrong type.

typeshed contract: cid is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _interpchannels import release
try:
    release(_W())  # cid: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_interpchannels/send__cid_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs__interpchannels_send__cid_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_interpchannels"
# dimension = "type"
# case = "send__cid_as_SupportsIndex_wrong"
# subject = "_interpchannels.send(cid: SupportsIndex)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_interpchannels.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _interpchannels.send(cid: SupportsIndex); call it with the wrong type.

typeshed contract: cid is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _interpchannels import send
try:
    send(_W(), None)  # cid: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_interpchannels/send_buffer__cid_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs__interpchannels_send_buffer__cid_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_interpchannels"
# dimension = "type"
# case = "send_buffer__cid_as_SupportsIndex_wrong"
# subject = "_interpchannels.send_buffer(cid: SupportsIndex)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_interpchannels.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _interpchannels.send_buffer(cid: SupportsIndex); call it with the wrong type.

typeshed contract: cid is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _interpchannels import send_buffer
try:
    send_buffer(_W(), None)  # cid: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

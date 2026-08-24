use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/asynchat/async_chat__push_with_producer__producer_as_simple_producer_wrong.py`.
#[test]
fn test_gen_type_std_libs_asynchat_async_chat__push_with_producer__producer_as_simple_producer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asynchat"
# dimension = "type"
# case = "async_chat__push_with_producer__producer_as_simple_producer_wrong"
# subject = "asynchat.async_chat.push_with_producer(producer: simple_producer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asynchat.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asynchat.async_chat.push_with_producer(producer: simple_producer); call it with the wrong type.

typeshed contract: producer is simple_producer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asynchat import async_chat
obj = object.__new__(async_chat)
try:
    obj.push_with_producer(_W())  # producer: simple_producer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asynchat/async_chat__set_terminator__term_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_asynchat_async_chat__set_terminator__term_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asynchat"
# dimension = "type"
# case = "async_chat__set_terminator__term_as_typed_wrong"
# subject = "asynchat.async_chat.set_terminator(term: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asynchat.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asynchat.async_chat.set_terminator(term: typed); call it with the wrong type.

typeshed contract: term is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asynchat import async_chat
obj = object.__new__(async_chat)
try:
    obj.set_terminator(_W())  # term: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

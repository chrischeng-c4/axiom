use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/lib2to3_pgen2_pgen/DFAState____eq____other_as_DFAState_wrong.py`.
#[test]
fn test_gen_type_std_libs_lib2to3_pgen2_pgen_DFAState____eq____other_as_DFAState_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_pgen2_pgen"
# dimension = "type"
# case = "DFAState____eq____other_as_DFAState_wrong"
# subject = "lib2to3.pgen2.pgen.DFAState.__eq__(other: DFAState)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/pgen2/pgen.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.pgen2.pgen.DFAState.__eq__(other: DFAState); call it with the wrong type.

typeshed contract: other is DFAState. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lib2to3.pgen2.pgen import DFAState
obj = object.__new__(DFAState)
try:
    obj.__eq__(_W())  # other: DFAState <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lib2to3_pgen2_pgen/DFAState__addarc__next_as_DFAState_wrong.py`.
#[test]
fn test_gen_type_std_libs_lib2to3_pgen2_pgen_DFAState__addarc__next_as_DFAState_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_pgen2_pgen"
# dimension = "type"
# case = "DFAState__addarc__next_as_DFAState_wrong"
# subject = "lib2to3.pgen2.pgen.DFAState.addarc(next: DFAState)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/pgen2/pgen.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.pgen2.pgen.DFAState.addarc(next: DFAState); call it with the wrong type.

typeshed contract: next is DFAState. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lib2to3.pgen2.pgen import DFAState
obj = object.__new__(DFAState)
try:
    obj.addarc(_W(), "")  # next: DFAState <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lib2to3_pgen2_pgen/DFAState__init__nfaset_as_dict_wrong.py`.
#[test]
fn test_gen_type_std_libs_lib2to3_pgen2_pgen_DFAState__init__nfaset_as_dict_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_pgen2_pgen"
# dimension = "type"
# case = "DFAState__init__nfaset_as_dict_wrong"
# subject = "lib2to3.pgen2.pgen.DFAState.__init__(nfaset: dict)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/pgen2/pgen.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.pgen2.pgen.DFAState.__init__(nfaset: dict); call it with the wrong type.

typeshed contract: nfaset is dict. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from lib2to3.pgen2.pgen import DFAState
try:
    DFAState(12345, None)  # nfaset: dict <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lib2to3_pgen2_pgen/DFAState__unifystate__old_as_DFAState_wrong.py`.
#[test]
fn test_gen_type_std_libs_lib2to3_pgen2_pgen_DFAState__unifystate__old_as_DFAState_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_pgen2_pgen"
# dimension = "type"
# case = "DFAState__unifystate__old_as_DFAState_wrong"
# subject = "lib2to3.pgen2.pgen.DFAState.unifystate(old: DFAState)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/pgen2/pgen.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.pgen2.pgen.DFAState.unifystate(old: DFAState); call it with the wrong type.

typeshed contract: old is DFAState. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lib2to3.pgen2.pgen import DFAState
obj = object.__new__(DFAState)
try:
    obj.unifystate(_W(), None)  # old: DFAState <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lib2to3_pgen2_pgen/NFAState__addarc__next_as_NFAState_wrong.py`.
#[test]
fn test_gen_type_std_libs_lib2to3_pgen2_pgen_NFAState__addarc__next_as_NFAState_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_pgen2_pgen"
# dimension = "type"
# case = "NFAState__addarc__next_as_NFAState_wrong"
# subject = "lib2to3.pgen2.pgen.NFAState.addarc(next: NFAState)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/pgen2/pgen.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.pgen2.pgen.NFAState.addarc(next: NFAState); call it with the wrong type.

typeshed contract: next is NFAState. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lib2to3.pgen2.pgen import NFAState
obj = object.__new__(NFAState)
try:
    obj.addarc(_W())  # next: NFAState <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lib2to3_pgen2_pgen/ParserGenerator__calcfirst__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_lib2to3_pgen2_pgen_ParserGenerator__calcfirst__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_pgen2_pgen"
# dimension = "type"
# case = "ParserGenerator__calcfirst__name_as_str_wrong"
# subject = "lib2to3.pgen2.pgen.ParserGenerator.calcfirst(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/pgen2/pgen.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.pgen2.pgen.ParserGenerator.calcfirst(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from lib2to3.pgen2.pgen import ParserGenerator
obj = object.__new__(ParserGenerator)
try:
    obj.calcfirst(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lib2to3_pgen2_pgen/ParserGenerator__dump_dfa__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_lib2to3_pgen2_pgen_ParserGenerator__dump_dfa__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_pgen2_pgen"
# dimension = "type"
# case = "ParserGenerator__dump_dfa__name_as_str_wrong"
# subject = "lib2to3.pgen2.pgen.ParserGenerator.dump_dfa(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/pgen2/pgen.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.pgen2.pgen.ParserGenerator.dump_dfa(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from lib2to3.pgen2.pgen import ParserGenerator
obj = object.__new__(ParserGenerator)
try:
    obj.dump_dfa(12345, None)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lib2to3_pgen2_pgen/ParserGenerator__dump_nfa__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_lib2to3_pgen2_pgen_ParserGenerator__dump_nfa__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_pgen2_pgen"
# dimension = "type"
# case = "ParserGenerator__dump_nfa__name_as_str_wrong"
# subject = "lib2to3.pgen2.pgen.ParserGenerator.dump_nfa(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/pgen2/pgen.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.pgen2.pgen.ParserGenerator.dump_nfa(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from lib2to3.pgen2.pgen import ParserGenerator
obj = object.__new__(ParserGenerator)
try:
    obj.dump_nfa(12345, None, None)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lib2to3_pgen2_pgen/ParserGenerator__expect__type_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_lib2to3_pgen2_pgen_ParserGenerator__expect__type_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_pgen2_pgen"
# dimension = "type"
# case = "ParserGenerator__expect__type_as_int_wrong"
# subject = "lib2to3.pgen2.pgen.ParserGenerator.expect(type: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/pgen2/pgen.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.pgen2.pgen.ParserGenerator.expect(type: int); call it with the wrong type.

typeshed contract: type is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from lib2to3.pgen2.pgen import ParserGenerator
obj = object.__new__(ParserGenerator)
try:
    obj.expect("not_an_int")  # type: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lib2to3_pgen2_pgen/ParserGenerator__init__filename_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_lib2to3_pgen2_pgen_ParserGenerator__init__filename_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_pgen2_pgen"
# dimension = "type"
# case = "ParserGenerator__init__filename_as_StrPath_wrong"
# subject = "lib2to3.pgen2.pgen.ParserGenerator.__init__(filename: StrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/pgen2/pgen.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.pgen2.pgen.ParserGenerator.__init__(filename: StrPath); call it with the wrong type.

typeshed contract: filename is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lib2to3.pgen2.pgen import ParserGenerator
try:
    ParserGenerator(_W())  # filename: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lib2to3_pgen2_pgen/ParserGenerator__make_dfa__start_as_NFAState_wrong.py`.
#[test]
fn test_gen_type_std_libs_lib2to3_pgen2_pgen_ParserGenerator__make_dfa__start_as_NFAState_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_pgen2_pgen"
# dimension = "type"
# case = "ParserGenerator__make_dfa__start_as_NFAState_wrong"
# subject = "lib2to3.pgen2.pgen.ParserGenerator.make_dfa(start: NFAState)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/pgen2/pgen.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.pgen2.pgen.ParserGenerator.make_dfa(start: NFAState); call it with the wrong type.

typeshed contract: start is NFAState. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lib2to3.pgen2.pgen import ParserGenerator
obj = object.__new__(ParserGenerator)
try:
    obj.make_dfa(_W(), None)  # start: NFAState <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lib2to3_pgen2_pgen/ParserGenerator__make_first__c_as_PgenGrammar_wrong.py`.
#[test]
fn test_gen_type_std_libs_lib2to3_pgen2_pgen_ParserGenerator__make_first__c_as_PgenGrammar_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_pgen2_pgen"
# dimension = "type"
# case = "ParserGenerator__make_first__c_as_PgenGrammar_wrong"
# subject = "lib2to3.pgen2.pgen.ParserGenerator.make_first(c: PgenGrammar)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/pgen2/pgen.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.pgen2.pgen.ParserGenerator.make_first(c: PgenGrammar); call it with the wrong type.

typeshed contract: c is PgenGrammar. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lib2to3.pgen2.pgen import ParserGenerator
obj = object.__new__(ParserGenerator)
try:
    obj.make_first(_W(), "")  # c: PgenGrammar <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lib2to3_pgen2_pgen/ParserGenerator__make_label__c_as_PgenGrammar_wrong.py`.
#[test]
fn test_gen_type_std_libs_lib2to3_pgen2_pgen_ParserGenerator__make_label__c_as_PgenGrammar_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_pgen2_pgen"
# dimension = "type"
# case = "ParserGenerator__make_label__c_as_PgenGrammar_wrong"
# subject = "lib2to3.pgen2.pgen.ParserGenerator.make_label(c: PgenGrammar)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/pgen2/pgen.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.pgen2.pgen.ParserGenerator.make_label(c: PgenGrammar); call it with the wrong type.

typeshed contract: c is PgenGrammar. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lib2to3.pgen2.pgen import ParserGenerator
obj = object.__new__(ParserGenerator)
try:
    obj.make_label(_W(), "")  # c: PgenGrammar <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lib2to3_pgen2_pgen/ParserGenerator__simplify_dfa__dfa_as_list_wrong.py`.
#[test]
fn test_gen_type_std_libs_lib2to3_pgen2_pgen_ParserGenerator__simplify_dfa__dfa_as_list_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_pgen2_pgen"
# dimension = "type"
# case = "ParserGenerator__simplify_dfa__dfa_as_list_wrong"
# subject = "lib2to3.pgen2.pgen.ParserGenerator.simplify_dfa(dfa: list)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/pgen2/pgen.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.pgen2.pgen.ParserGenerator.simplify_dfa(dfa: list); call it with the wrong type.

typeshed contract: dfa is list. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from lib2to3.pgen2.pgen import ParserGenerator
obj = object.__new__(ParserGenerator)
try:
    obj.simplify_dfa(12345)  # dfa: list <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lib2to3_pgen2_pgen/generate_grammar__filename_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_lib2to3_pgen2_pgen_generate_grammar__filename_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_pgen2_pgen"
# dimension = "type"
# case = "generate_grammar__filename_as_StrPath_wrong"
# subject = "lib2to3.pgen2.pgen.generate_grammar(filename: StrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/pgen2/pgen.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.pgen2.pgen.generate_grammar(filename: StrPath); call it with the wrong type.

typeshed contract: filename is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lib2to3.pgen2.pgen import generate_grammar
try:
    generate_grammar(_W())  # filename: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

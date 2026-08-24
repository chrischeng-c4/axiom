use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/pipes/Template__append__cmd_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_pipes_Template__append__cmd_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pipes"
# dimension = "type"
# case = "Template__append__cmd_as_str_wrong"
# subject = "pipes.Template.append(cmd: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pipes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pipes.Template.append(cmd: str); call it with the wrong type.

typeshed contract: cmd is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pipes import Template
obj = object.__new__(Template)
try:
    obj.append(12345, "")  # cmd: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pipes/Template__copy__infile_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_pipes_Template__copy__infile_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pipes"
# dimension = "type"
# case = "Template__copy__infile_as_str_wrong"
# subject = "pipes.Template.copy(infile: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pipes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pipes.Template.copy(infile: str); call it with the wrong type.

typeshed contract: infile is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pipes import Template
obj = object.__new__(Template)
try:
    obj.copy(12345, "")  # infile: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pipes/Template__debug__flag_as_bool_wrong.py`.
#[test]
fn test_gen_type_std_libs_pipes_Template__debug__flag_as_bool_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pipes"
# dimension = "type"
# case = "Template__debug__flag_as_bool_wrong"
# subject = "pipes.Template.debug(flag: bool)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pipes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pipes.Template.debug(flag: bool); call it with the wrong type.

typeshed contract: flag is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pipes import Template
obj = Template()
try:
    obj.debug("not_a_bool")  # flag: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pipes/Template__open__file_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_pipes_Template__open__file_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pipes"
# dimension = "type"
# case = "Template__open__file_as_str_wrong"
# subject = "pipes.Template.open(file: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pipes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pipes.Template.open(file: str); call it with the wrong type.

typeshed contract: file is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pipes import Template
obj = object.__new__(Template)
try:
    obj.open(12345, "")  # file: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pipes/Template__prepend__cmd_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_pipes_Template__prepend__cmd_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pipes"
# dimension = "type"
# case = "Template__prepend__cmd_as_str_wrong"
# subject = "pipes.Template.prepend(cmd: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pipes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pipes.Template.prepend(cmd: str); call it with the wrong type.

typeshed contract: cmd is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pipes import Template
obj = object.__new__(Template)
try:
    obj.prepend(12345, "")  # cmd: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pipes/quote__s_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_pipes_quote__s_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pipes"
# dimension = "type"
# case = "quote__s_as_str_wrong"
# subject = "pipes.quote(s: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pipes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pipes.quote(s: str); call it with the wrong type.

typeshed contract: s is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pipes import quote
try:
    quote(12345)  # s: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

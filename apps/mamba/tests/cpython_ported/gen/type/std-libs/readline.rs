use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/readline/add_history__string_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_readline_add_history__string_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "readline"
# dimension = "type"
# case = "add_history__string_as_str_wrong"
# subject = "readline.add_history(string: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/readline.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: readline.add_history(string: str); call it with the wrong type.

typeshed contract: string is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from readline import add_history
try:
    add_history(12345)  # string: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/readline/append_history_file__nelements_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_readline_append_history_file__nelements_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "readline"
# dimension = "type"
# case = "append_history_file__nelements_as_int_wrong"
# subject = "readline.append_history_file(nelements: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/readline.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: readline.append_history_file(nelements: int); call it with the wrong type.

typeshed contract: nelements is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from readline import append_history_file
try:
    append_history_file("not_an_int")  # nelements: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/readline/get_history_item__index_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_readline_get_history_item__index_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "readline"
# dimension = "type"
# case = "get_history_item__index_as_int_wrong"
# subject = "readline.get_history_item(index: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/readline.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: readline.get_history_item(index: int); call it with the wrong type.

typeshed contract: index is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from readline import get_history_item
try:
    get_history_item("not_an_int")  # index: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/readline/insert_text__string_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_readline_insert_text__string_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "readline"
# dimension = "type"
# case = "insert_text__string_as_str_wrong"
# subject = "readline.insert_text(string: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/readline.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: readline.insert_text(string: str); call it with the wrong type.

typeshed contract: string is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from readline import insert_text
try:
    insert_text(12345)  # string: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/readline/parse_and_bind__string_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_readline_parse_and_bind__string_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "readline"
# dimension = "type"
# case = "parse_and_bind__string_as_str_wrong"
# subject = "readline.parse_and_bind(string: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/readline.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: readline.parse_and_bind(string: str); call it with the wrong type.

typeshed contract: string is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from readline import parse_and_bind
try:
    parse_and_bind(12345)  # string: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/readline/read_history_file__filename_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_readline_read_history_file__filename_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "readline"
# dimension = "type"
# case = "read_history_file__filename_as_typed_wrong"
# subject = "readline.read_history_file(filename: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/readline.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: readline.read_history_file(filename: typed); call it with the wrong type.

typeshed contract: filename is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from readline import read_history_file
try:
    read_history_file(_W())  # filename: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/readline/read_init_file__filename_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_readline_read_init_file__filename_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "readline"
# dimension = "type"
# case = "read_init_file__filename_as_typed_wrong"
# subject = "readline.read_init_file(filename: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/readline.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: readline.read_init_file(filename: typed); call it with the wrong type.

typeshed contract: filename is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from readline import read_init_file
try:
    read_init_file(_W())  # filename: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/readline/remove_history_item__pos_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_readline_remove_history_item__pos_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "readline"
# dimension = "type"
# case = "remove_history_item__pos_as_int_wrong"
# subject = "readline.remove_history_item(pos: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/readline.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: readline.remove_history_item(pos: int); call it with the wrong type.

typeshed contract: pos is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from readline import remove_history_item
try:
    remove_history_item("not_an_int")  # pos: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/readline/replace_history_item__pos_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_readline_replace_history_item__pos_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "readline"
# dimension = "type"
# case = "replace_history_item__pos_as_int_wrong"
# subject = "readline.replace_history_item(pos: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/readline.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: readline.replace_history_item(pos: int); call it with the wrong type.

typeshed contract: pos is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from readline import replace_history_item
try:
    replace_history_item("not_an_int", "")  # pos: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/readline/set_completer__function_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_readline_set_completer__function_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "readline"
# dimension = "type"
# case = "set_completer__function_as_typed_wrong"
# subject = "readline.set_completer(function: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/readline.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: readline.set_completer(function: typed); call it with the wrong type.

typeshed contract: function is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from readline import set_completer
try:
    set_completer(_W())  # function: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/readline/set_completer_delims__string_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_readline_set_completer_delims__string_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "readline"
# dimension = "type"
# case = "set_completer_delims__string_as_str_wrong"
# subject = "readline.set_completer_delims(string: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/readline.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: readline.set_completer_delims(string: str); call it with the wrong type.

typeshed contract: string is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from readline import set_completer_delims
try:
    set_completer_delims(12345)  # string: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/readline/set_completion_display_matches_hook__function_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_readline_set_completion_display_matches_hook__function_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "readline"
# dimension = "type"
# case = "set_completion_display_matches_hook__function_as_typed_wrong"
# subject = "readline.set_completion_display_matches_hook(function: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/readline.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: readline.set_completion_display_matches_hook(function: typed); call it with the wrong type.

typeshed contract: function is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from readline import set_completion_display_matches_hook
try:
    set_completion_display_matches_hook(_W())  # function: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/readline/set_history_length__length_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_readline_set_history_length__length_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "readline"
# dimension = "type"
# case = "set_history_length__length_as_int_wrong"
# subject = "readline.set_history_length(length: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/readline.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: readline.set_history_length(length: int); call it with the wrong type.

typeshed contract: length is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from readline import set_history_length
try:
    set_history_length("not_an_int")  # length: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/readline/write_history_file__filename_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_readline_write_history_file__filename_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "readline"
# dimension = "type"
# case = "write_history_file__filename_as_typed_wrong"
# subject = "readline.write_history_file(filename: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/readline.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: readline.write_history_file(filename: typed); call it with the wrong type.

typeshed contract: filename is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from readline import write_history_file
try:
    write_history_file(_W())  # filename: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

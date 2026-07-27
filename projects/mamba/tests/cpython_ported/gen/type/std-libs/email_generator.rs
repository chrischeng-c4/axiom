use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/email_generator/BytesGenerator__init__outfp_as_SupportsWrite_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_generator_BytesGenerator__init__outfp_as_SupportsWrite_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_generator"
# dimension = "type"
# case = "BytesGenerator__init__outfp_as_SupportsWrite_wrong"
# subject = "email.generator.BytesGenerator.__init__(outfp: SupportsWrite)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/generator.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.generator.BytesGenerator.__init__(outfp: SupportsWrite); call it with the wrong type.

typeshed contract: outfp is SupportsWrite. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.generator import BytesGenerator
try:
    BytesGenerator(_W())  # outfp: SupportsWrite <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_generator/DecodedGenerator__init__outfp_as_SupportsWrite_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_generator_DecodedGenerator__init__outfp_as_SupportsWrite_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_generator"
# dimension = "type"
# case = "DecodedGenerator__init__outfp_as_SupportsWrite_wrong"
# subject = "email.generator.DecodedGenerator.__init__(outfp: SupportsWrite)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/generator.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.generator.DecodedGenerator.__init__(outfp: SupportsWrite); call it with the wrong type.

typeshed contract: outfp is SupportsWrite. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.generator import DecodedGenerator
try:
    DecodedGenerator(_W())  # outfp: SupportsWrite <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_generator/Generator__clone__fp_as_SupportsWrite_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_generator_Generator__clone__fp_as_SupportsWrite_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_generator"
# dimension = "type"
# case = "Generator__clone__fp_as_SupportsWrite_wrong"
# subject = "email.generator.Generator.clone(fp: SupportsWrite)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/generator.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.generator.Generator.clone(fp: SupportsWrite); call it with the wrong type.

typeshed contract: fp is SupportsWrite. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.generator import Generator
obj = object.__new__(Generator)
try:
    obj.clone(_W())  # fp: SupportsWrite <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_generator/Generator__flatten__msg_as__MessageT_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_generator_Generator__flatten__msg_as__MessageT_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_generator"
# dimension = "type"
# case = "Generator__flatten__msg_as__MessageT_wrong"
# subject = "email.generator.Generator.flatten(msg: _MessageT)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/generator.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.generator.Generator.flatten(msg: _MessageT); call it with the wrong type.

typeshed contract: msg is _MessageT. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.generator import Generator
obj = object.__new__(Generator)
try:
    obj.flatten(_W())  # msg: _MessageT <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_generator/Generator__init__outfp_as_SupportsWrite_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_generator_Generator__init__outfp_as_SupportsWrite_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_generator"
# dimension = "type"
# case = "Generator__init__outfp_as_SupportsWrite_wrong"
# subject = "email.generator.Generator.__init__(outfp: SupportsWrite)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/generator.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.generator.Generator.__init__(outfp: SupportsWrite); call it with the wrong type.

typeshed contract: outfp is SupportsWrite. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.generator import Generator
try:
    Generator(_W())  # outfp: SupportsWrite <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_generator/Generator__write__s_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_generator_Generator__write__s_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_generator"
# dimension = "type"
# case = "Generator__write__s_as_str_wrong"
# subject = "email.generator.Generator.write(s: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/generator.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.generator.Generator.write(s: str); call it with the wrong type.

typeshed contract: s is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.generator import Generator
obj = object.__new__(Generator)
try:
    obj.write(12345)  # s: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

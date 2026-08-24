use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/wsgiref_validate/ErrorWrapper__init__wsgi_errors_as_ErrorStream_wrong.py`.
#[test]
fn test_gen_type_std_libs_wsgiref_validate_ErrorWrapper__init__wsgi_errors_as_ErrorStream_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "wsgiref_validate"
# dimension = "type"
# case = "ErrorWrapper__init__wsgi_errors_as_ErrorStream_wrong"
# subject = "wsgiref.validate.ErrorWrapper.__init__(wsgi_errors: ErrorStream)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/wsgiref/validate.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: wsgiref.validate.ErrorWrapper.__init__(wsgi_errors: ErrorStream); call it with the wrong type.

typeshed contract: wsgi_errors is ErrorStream. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from wsgiref.validate import ErrorWrapper
try:
    ErrorWrapper(_W())  # wsgi_errors: ErrorStream <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/wsgiref_validate/ErrorWrapper__write__s_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_wsgiref_validate_ErrorWrapper__write__s_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "wsgiref_validate"
# dimension = "type"
# case = "ErrorWrapper__write__s_as_str_wrong"
# subject = "wsgiref.validate.ErrorWrapper.write(s: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/wsgiref/validate.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: wsgiref.validate.ErrorWrapper.write(s: str); call it with the wrong type.

typeshed contract: s is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from wsgiref.validate import ErrorWrapper
obj = object.__new__(ErrorWrapper)
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

/// Ported from `tests/cpython/type/std-libs/wsgiref_validate/ErrorWrapper__writelines__seq_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_wsgiref_validate_ErrorWrapper__writelines__seq_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "wsgiref_validate"
# dimension = "type"
# case = "ErrorWrapper__writelines__seq_as_Iterable_wrong"
# subject = "wsgiref.validate.ErrorWrapper.writelines(seq: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/wsgiref/validate.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: wsgiref.validate.ErrorWrapper.writelines(seq: Iterable); call it with the wrong type.

typeshed contract: seq is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from wsgiref.validate import ErrorWrapper
obj = object.__new__(ErrorWrapper)
try:
    obj.writelines(_W())  # seq: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/wsgiref_validate/InputWrapper__init__wsgi_input_as_InputStream_wrong.py`.
#[test]
fn test_gen_type_std_libs_wsgiref_validate_InputWrapper__init__wsgi_input_as_InputStream_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "wsgiref_validate"
# dimension = "type"
# case = "InputWrapper__init__wsgi_input_as_InputStream_wrong"
# subject = "wsgiref.validate.InputWrapper.__init__(wsgi_input: InputStream)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/wsgiref/validate.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: wsgiref.validate.InputWrapper.__init__(wsgi_input: InputStream); call it with the wrong type.

typeshed contract: wsgi_input is InputStream. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from wsgiref.validate import InputWrapper
try:
    InputWrapper(_W())  # wsgi_input: InputStream <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/wsgiref_validate/InputWrapper__read__size_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_wsgiref_validate_InputWrapper__read__size_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "wsgiref_validate"
# dimension = "type"
# case = "InputWrapper__read__size_as_int_wrong"
# subject = "wsgiref.validate.InputWrapper.read(size: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/wsgiref/validate.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: wsgiref.validate.InputWrapper.read(size: int); call it with the wrong type.

typeshed contract: size is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from wsgiref.validate import InputWrapper
obj = object.__new__(InputWrapper)
try:
    obj.read("not_an_int")  # size: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/wsgiref_validate/InputWrapper__readline__size_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_wsgiref_validate_InputWrapper__readline__size_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "wsgiref_validate"
# dimension = "type"
# case = "InputWrapper__readline__size_as_int_wrong"
# subject = "wsgiref.validate.InputWrapper.readline(size: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/wsgiref/validate.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: wsgiref.validate.InputWrapper.readline(size: int); call it with the wrong type.

typeshed contract: size is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from wsgiref.validate import InputWrapper
obj = object.__new__(InputWrapper)
try:
    obj.readline("not_an_int")  # size: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/wsgiref_validate/InputWrapper__readlines__hint_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_wsgiref_validate_InputWrapper__readlines__hint_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "wsgiref_validate"
# dimension = "type"
# case = "InputWrapper__readlines__hint_as_int_wrong"
# subject = "wsgiref.validate.InputWrapper.readlines(hint: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/wsgiref/validate.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: wsgiref.validate.InputWrapper.readlines(hint: int); call it with the wrong type.

typeshed contract: hint is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from wsgiref.validate import InputWrapper
obj = object.__new__(InputWrapper)
try:
    obj.readlines("not_an_int")  # hint: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/wsgiref_validate/IteratorWrapper__init__wsgi_iterator_as_Iterator_wrong.py`.
#[test]
fn test_gen_type_std_libs_wsgiref_validate_IteratorWrapper__init__wsgi_iterator_as_Iterator_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "wsgiref_validate"
# dimension = "type"
# case = "IteratorWrapper__init__wsgi_iterator_as_Iterator_wrong"
# subject = "wsgiref.validate.IteratorWrapper.__init__(wsgi_iterator: Iterator)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/wsgiref/validate.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: wsgiref.validate.IteratorWrapper.__init__(wsgi_iterator: Iterator); call it with the wrong type.

typeshed contract: wsgi_iterator is Iterator. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from wsgiref.validate import IteratorWrapper
try:
    IteratorWrapper(_W(), None)  # wsgi_iterator: Iterator <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/wsgiref_validate/PartialIteratorWrapper__init__wsgi_iterator_as_Iterator_wrong.py`.
#[test]
fn test_gen_type_std_libs_wsgiref_validate_PartialIteratorWrapper__init__wsgi_iterator_as_Iterator_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "wsgiref_validate"
# dimension = "type"
# case = "PartialIteratorWrapper__init__wsgi_iterator_as_Iterator_wrong"
# subject = "wsgiref.validate.PartialIteratorWrapper.__init__(wsgi_iterator: Iterator)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/wsgiref/validate.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: wsgiref.validate.PartialIteratorWrapper.__init__(wsgi_iterator: Iterator); call it with the wrong type.

typeshed contract: wsgi_iterator is Iterator. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from wsgiref.validate import PartialIteratorWrapper
try:
    PartialIteratorWrapper(_W())  # wsgi_iterator: Iterator <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/wsgiref_validate/WriteWrapper__init__wsgi_writer_as__WriterCallback_wrong.py`.
#[test]
fn test_gen_type_std_libs_wsgiref_validate_WriteWrapper__init__wsgi_writer_as__WriterCallback_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "wsgiref_validate"
# dimension = "type"
# case = "WriteWrapper__init__wsgi_writer_as__WriterCallback_wrong"
# subject = "wsgiref.validate.WriteWrapper.__init__(wsgi_writer: _WriterCallback)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/wsgiref/validate.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: wsgiref.validate.WriteWrapper.__init__(wsgi_writer: _WriterCallback); call it with the wrong type.

typeshed contract: wsgi_writer is _WriterCallback. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from wsgiref.validate import WriteWrapper
try:
    WriteWrapper(_W())  # wsgi_writer: _WriterCallback <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/wsgiref_validate/validator__application_as_WSGIApplication_wrong.py`.
#[test]
fn test_gen_type_std_libs_wsgiref_validate_validator__application_as_WSGIApplication_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "wsgiref_validate"
# dimension = "type"
# case = "validator__application_as_WSGIApplication_wrong"
# subject = "wsgiref.validate.validator(application: WSGIApplication)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/wsgiref/validate.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: wsgiref.validate.validator(application: WSGIApplication); call it with the wrong type.

typeshed contract: application is WSGIApplication. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from wsgiref.validate import validator
try:
    validator(_W())  # application: WSGIApplication <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

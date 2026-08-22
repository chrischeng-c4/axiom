use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/wsgiref_handlers/BaseHandler__error_output__environ_as_WSGIEnvironment_wrong.py`.
#[test]
fn test_gen_type_std_libs_wsgiref_handlers_BaseHandler__error_output__environ_as_WSGIEnvironment_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "wsgiref_handlers"
# dimension = "type"
# case = "BaseHandler__error_output__environ_as_WSGIEnvironment_wrong"
# subject = "wsgiref.handlers.BaseHandler.error_output(environ: WSGIEnvironment)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/wsgiref/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: wsgiref.handlers.BaseHandler.error_output(environ: WSGIEnvironment); call it with the wrong type.

typeshed contract: environ is WSGIEnvironment. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from wsgiref.handlers import BaseHandler
obj = object.__new__(BaseHandler)
try:
    obj.error_output(_W(), None)  # environ: WSGIEnvironment <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/wsgiref_handlers/BaseHandler__log_exception__exc_info_as_OptExcInfo_wrong.py`.
#[test]
fn test_gen_type_std_libs_wsgiref_handlers_BaseHandler__log_exception__exc_info_as_OptExcInfo_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "wsgiref_handlers"
# dimension = "type"
# case = "BaseHandler__log_exception__exc_info_as_OptExcInfo_wrong"
# subject = "wsgiref.handlers.BaseHandler.log_exception(exc_info: OptExcInfo)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/wsgiref/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: wsgiref.handlers.BaseHandler.log_exception(exc_info: OptExcInfo); call it with the wrong type.

typeshed contract: exc_info is OptExcInfo. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from wsgiref.handlers import BaseHandler
obj = object.__new__(BaseHandler)
try:
    obj.log_exception(_W())  # exc_info: OptExcInfo <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/wsgiref_handlers/BaseHandler__run__application_as_WSGIApplication_wrong.py`.
#[test]
fn test_gen_type_std_libs_wsgiref_handlers_BaseHandler__run__application_as_WSGIApplication_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "wsgiref_handlers"
# dimension = "type"
# case = "BaseHandler__run__application_as_WSGIApplication_wrong"
# subject = "wsgiref.handlers.BaseHandler.run(application: WSGIApplication)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/wsgiref/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: wsgiref.handlers.BaseHandler.run(application: WSGIApplication); call it with the wrong type.

typeshed contract: application is WSGIApplication. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from wsgiref.handlers import BaseHandler
obj = object.__new__(BaseHandler)
try:
    obj.run(_W())  # application: WSGIApplication <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/wsgiref_handlers/BaseHandler__start_response__status_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_wsgiref_handlers_BaseHandler__start_response__status_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "wsgiref_handlers"
# dimension = "type"
# case = "BaseHandler__start_response__status_as_str_wrong"
# subject = "wsgiref.handlers.BaseHandler.start_response(status: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/wsgiref/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: wsgiref.handlers.BaseHandler.start_response(status: str); call it with the wrong type.

typeshed contract: status is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from wsgiref.handlers import BaseHandler
obj = object.__new__(BaseHandler)
try:
    obj.start_response(12345, None)  # status: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/wsgiref_handlers/BaseHandler__write__data_as_bytes_wrong.py`.
#[test]
fn test_gen_type_std_libs_wsgiref_handlers_BaseHandler__write__data_as_bytes_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "wsgiref_handlers"
# dimension = "type"
# case = "BaseHandler__write__data_as_bytes_wrong"
# subject = "wsgiref.handlers.BaseHandler.write(data: bytes)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/wsgiref/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: wsgiref.handlers.BaseHandler.write(data: bytes); call it with the wrong type.

typeshed contract: data is bytes. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from wsgiref.handlers import BaseHandler
obj = object.__new__(BaseHandler)
try:
    obj.write(12345)  # data: bytes <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/wsgiref_handlers/SimpleHandler__init__stdin_as_InputStream_wrong.py`.
#[test]
fn test_gen_type_std_libs_wsgiref_handlers_SimpleHandler__init__stdin_as_InputStream_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "wsgiref_handlers"
# dimension = "type"
# case = "SimpleHandler__init__stdin_as_InputStream_wrong"
# subject = "wsgiref.handlers.SimpleHandler.__init__(stdin: InputStream)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/wsgiref/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: wsgiref.handlers.SimpleHandler.__init__(stdin: InputStream); call it with the wrong type.

typeshed contract: stdin is InputStream. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from wsgiref.handlers import SimpleHandler
try:
    SimpleHandler(_W(), None, None, None)  # stdin: InputStream <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/wsgiref_handlers/format_date_time__timestamp_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_wsgiref_handlers_format_date_time__timestamp_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "wsgiref_handlers"
# dimension = "type"
# case = "format_date_time__timestamp_as_typed_wrong"
# subject = "wsgiref.handlers.format_date_time(timestamp: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/wsgiref/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: wsgiref.handlers.format_date_time(timestamp: typed); call it with the wrong type.

typeshed contract: timestamp is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from wsgiref.handlers import format_date_time
try:
    format_date_time(_W())  # timestamp: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

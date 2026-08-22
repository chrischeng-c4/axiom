use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/logging_handlers/BaseRotatingHandler__init__filename_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_handlers_BaseRotatingHandler__init__filename_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "type"
# case = "BaseRotatingHandler__init__filename_as_StrPath_wrong"
# subject = "logging.handlers.BaseRotatingHandler.__init__(filename: StrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.handlers.BaseRotatingHandler.__init__(filename: StrPath); call it with the wrong type.

typeshed contract: filename is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging.handlers import BaseRotatingHandler
try:
    BaseRotatingHandler(_W(), "")  # filename: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_handlers/BaseRotatingHandler__rotate__source_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_handlers_BaseRotatingHandler__rotate__source_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "type"
# case = "BaseRotatingHandler__rotate__source_as_str_wrong"
# subject = "logging.handlers.BaseRotatingHandler.rotate(source: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.handlers.BaseRotatingHandler.rotate(source: str); call it with the wrong type.

typeshed contract: source is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from logging.handlers import BaseRotatingHandler
obj = object.__new__(BaseRotatingHandler)
try:
    obj.rotate(12345, "")  # source: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_handlers/BaseRotatingHandler__rotation_filename__default_name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_handlers_BaseRotatingHandler__rotation_filename__default_name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "type"
# case = "BaseRotatingHandler__rotation_filename__default_name_as_str_wrong"
# subject = "logging.handlers.BaseRotatingHandler.rotation_filename(default_name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.handlers.BaseRotatingHandler.rotation_filename(default_name: str); call it with the wrong type.

typeshed contract: default_name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from logging.handlers import BaseRotatingHandler
obj = object.__new__(BaseRotatingHandler)
try:
    obj.rotation_filename(12345)  # default_name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_handlers/BufferingHandler__init__capacity_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_handlers_BufferingHandler__init__capacity_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "type"
# case = "BufferingHandler__init__capacity_as_int_wrong"
# subject = "logging.handlers.BufferingHandler.__init__(capacity: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.handlers.BufferingHandler.__init__(capacity: int); call it with the wrong type.

typeshed contract: capacity is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from logging.handlers import BufferingHandler
try:
    BufferingHandler("not_an_int")  # capacity: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_handlers/BufferingHandler__shouldFlush__record_as_LogRecord_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_handlers_BufferingHandler__shouldFlush__record_as_LogRecord_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "type"
# case = "BufferingHandler__shouldFlush__record_as_LogRecord_wrong"
# subject = "logging.handlers.BufferingHandler.shouldFlush(record: LogRecord)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.handlers.BufferingHandler.shouldFlush(record: LogRecord); call it with the wrong type.

typeshed contract: record is LogRecord. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging.handlers import BufferingHandler
obj = object.__new__(BufferingHandler)
try:
    obj.shouldFlush(_W())  # record: LogRecord <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_handlers/HTTPHandler__getConnection__host_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_handlers_HTTPHandler__getConnection__host_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "type"
# case = "HTTPHandler__getConnection__host_as_str_wrong"
# subject = "logging.handlers.HTTPHandler.getConnection(host: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.handlers.HTTPHandler.getConnection(host: str); call it with the wrong type.

typeshed contract: host is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from logging.handlers import HTTPHandler
obj = object.__new__(HTTPHandler)
try:
    obj.getConnection(12345, True)  # host: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_handlers/HTTPHandler__init__host_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_handlers_HTTPHandler__init__host_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "type"
# case = "HTTPHandler__init__host_as_str_wrong"
# subject = "logging.handlers.HTTPHandler.__init__(host: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.handlers.HTTPHandler.__init__(host: str); call it with the wrong type.

typeshed contract: host is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from logging.handlers import HTTPHandler
try:
    HTTPHandler(12345, "")  # host: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_handlers/HTTPHandler__mapLogRecord__record_as_LogRecord_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_handlers_HTTPHandler__mapLogRecord__record_as_LogRecord_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "type"
# case = "HTTPHandler__mapLogRecord__record_as_LogRecord_wrong"
# subject = "logging.handlers.HTTPHandler.mapLogRecord(record: LogRecord)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.handlers.HTTPHandler.mapLogRecord(record: LogRecord); call it with the wrong type.

typeshed contract: record is LogRecord. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging.handlers import HTTPHandler
obj = object.__new__(HTTPHandler)
try:
    obj.mapLogRecord(_W())  # record: LogRecord <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_handlers/MemoryHandler__init__capacity_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_handlers_MemoryHandler__init__capacity_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "type"
# case = "MemoryHandler__init__capacity_as_int_wrong"
# subject = "logging.handlers.MemoryHandler.__init__(capacity: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.handlers.MemoryHandler.__init__(capacity: int); call it with the wrong type.

typeshed contract: capacity is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from logging.handlers import MemoryHandler
try:
    MemoryHandler("not_an_int")  # capacity: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_handlers/MemoryHandler__setTarget__target_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_handlers_MemoryHandler__setTarget__target_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "type"
# case = "MemoryHandler__setTarget__target_as_typed_wrong"
# subject = "logging.handlers.MemoryHandler.setTarget(target: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.handlers.MemoryHandler.setTarget(target: typed); call it with the wrong type.

typeshed contract: target is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging.handlers import MemoryHandler
obj = object.__new__(MemoryHandler)
try:
    obj.setTarget(_W())  # target: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_handlers/NTEventLogHandler__getEventCategory__record_as_LogRecord_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_handlers_NTEventLogHandler__getEventCategory__record_as_LogRecord_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "type"
# case = "NTEventLogHandler__getEventCategory__record_as_LogRecord_wrong"
# subject = "logging.handlers.NTEventLogHandler.getEventCategory(record: LogRecord)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.handlers.NTEventLogHandler.getEventCategory(record: LogRecord); call it with the wrong type.

typeshed contract: record is LogRecord. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging.handlers import NTEventLogHandler
obj = object.__new__(NTEventLogHandler)
try:
    obj.getEventCategory(_W())  # record: LogRecord <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_handlers/NTEventLogHandler__getEventType__record_as_LogRecord_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_handlers_NTEventLogHandler__getEventType__record_as_LogRecord_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "type"
# case = "NTEventLogHandler__getEventType__record_as_LogRecord_wrong"
# subject = "logging.handlers.NTEventLogHandler.getEventType(record: LogRecord)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.handlers.NTEventLogHandler.getEventType(record: LogRecord); call it with the wrong type.

typeshed contract: record is LogRecord. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging.handlers import NTEventLogHandler
obj = object.__new__(NTEventLogHandler)
try:
    obj.getEventType(_W())  # record: LogRecord <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_handlers/NTEventLogHandler__getMessageID__record_as_LogRecord_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_handlers_NTEventLogHandler__getMessageID__record_as_LogRecord_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "type"
# case = "NTEventLogHandler__getMessageID__record_as_LogRecord_wrong"
# subject = "logging.handlers.NTEventLogHandler.getMessageID(record: LogRecord)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.handlers.NTEventLogHandler.getMessageID(record: LogRecord); call it with the wrong type.

typeshed contract: record is LogRecord. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging.handlers import NTEventLogHandler
obj = object.__new__(NTEventLogHandler)
try:
    obj.getMessageID(_W())  # record: LogRecord <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_handlers/NTEventLogHandler__init__appname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_handlers_NTEventLogHandler__init__appname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "type"
# case = "NTEventLogHandler__init__appname_as_str_wrong"
# subject = "logging.handlers.NTEventLogHandler.__init__(appname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.handlers.NTEventLogHandler.__init__(appname: str); call it with the wrong type.

typeshed contract: appname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from logging.handlers import NTEventLogHandler
try:
    NTEventLogHandler(12345)  # appname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_handlers/QueueHandler__enqueue__record_as_LogRecord_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_handlers_QueueHandler__enqueue__record_as_LogRecord_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "type"
# case = "QueueHandler__enqueue__record_as_LogRecord_wrong"
# subject = "logging.handlers.QueueHandler.enqueue(record: LogRecord)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.handlers.QueueHandler.enqueue(record: LogRecord); call it with the wrong type.

typeshed contract: record is LogRecord. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging.handlers import QueueHandler
obj = object.__new__(QueueHandler)
try:
    obj.enqueue(_W())  # record: LogRecord <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_handlers/QueueHandler__init__queue_as__QueueLike_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_handlers_QueueHandler__init__queue_as__QueueLike_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "type"
# case = "QueueHandler__init__queue_as__QueueLike_wrong"
# subject = "logging.handlers.QueueHandler.__init__(queue: _QueueLike)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.handlers.QueueHandler.__init__(queue: _QueueLike); call it with the wrong type.

typeshed contract: queue is _QueueLike. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging.handlers import QueueHandler
try:
    QueueHandler(_W())  # queue: _QueueLike <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_handlers/QueueHandler__prepare__record_as_LogRecord_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_handlers_QueueHandler__prepare__record_as_LogRecord_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "type"
# case = "QueueHandler__prepare__record_as_LogRecord_wrong"
# subject = "logging.handlers.QueueHandler.prepare(record: LogRecord)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.handlers.QueueHandler.prepare(record: LogRecord); call it with the wrong type.

typeshed contract: record is LogRecord. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging.handlers import QueueHandler
obj = object.__new__(QueueHandler)
try:
    obj.prepare(_W())  # record: LogRecord <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_handlers/QueueListener____exit____exc_type_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_handlers_QueueListener____exit____exc_type_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "type"
# case = "QueueListener____exit____exc_type_as_typed_wrong"
# subject = "logging.handlers.QueueListener.__exit__(exc_type: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.handlers.QueueListener.__exit__(exc_type: typed); call it with the wrong type.

typeshed contract: exc_type is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging.handlers import QueueListener
obj = object.__new__(QueueListener)
try:
    obj.__exit__(_W(), None, None)  # exc_type: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_handlers/QueueListener__dequeue__block_as_bool_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_handlers_QueueListener__dequeue__block_as_bool_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "type"
# case = "QueueListener__dequeue__block_as_bool_wrong"
# subject = "logging.handlers.QueueListener.dequeue(block: bool)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.handlers.QueueListener.dequeue(block: bool); call it with the wrong type.

typeshed contract: block is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from logging.handlers import QueueListener
obj = object.__new__(QueueListener)
try:
    obj.dequeue("not_a_bool")  # block: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_handlers/QueueListener__handle__record_as_LogRecord_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_handlers_QueueListener__handle__record_as_LogRecord_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "type"
# case = "QueueListener__handle__record_as_LogRecord_wrong"
# subject = "logging.handlers.QueueListener.handle(record: LogRecord)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.handlers.QueueListener.handle(record: LogRecord); call it with the wrong type.

typeshed contract: record is LogRecord. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging.handlers import QueueListener
obj = object.__new__(QueueListener)
try:
    obj.handle(_W())  # record: LogRecord <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_handlers/QueueListener__init__queue_as__QueueLike_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_handlers_QueueListener__init__queue_as__QueueLike_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "type"
# case = "QueueListener__init__queue_as__QueueLike_wrong"
# subject = "logging.handlers.QueueListener.__init__(queue: _QueueLike)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.handlers.QueueListener.__init__(queue: _QueueLike); call it with the wrong type.

typeshed contract: queue is _QueueLike. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging.handlers import QueueListener
try:
    QueueListener(_W())  # queue: _QueueLike <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_handlers/QueueListener__prepare__record_as_LogRecord_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_handlers_QueueListener__prepare__record_as_LogRecord_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "type"
# case = "QueueListener__prepare__record_as_LogRecord_wrong"
# subject = "logging.handlers.QueueListener.prepare(record: LogRecord)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.handlers.QueueListener.prepare(record: LogRecord); call it with the wrong type.

typeshed contract: record is LogRecord. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging.handlers import QueueListener
obj = object.__new__(QueueListener)
try:
    obj.prepare(_W())  # record: LogRecord <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_handlers/RotatingFileHandler__init__filename_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_handlers_RotatingFileHandler__init__filename_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "type"
# case = "RotatingFileHandler__init__filename_as_StrPath_wrong"
# subject = "logging.handlers.RotatingFileHandler.__init__(filename: StrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.handlers.RotatingFileHandler.__init__(filename: StrPath); call it with the wrong type.

typeshed contract: filename is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging.handlers import RotatingFileHandler
try:
    RotatingFileHandler(_W())  # filename: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_handlers/RotatingFileHandler__shouldRollover__record_as_LogRecord_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_handlers_RotatingFileHandler__shouldRollover__record_as_LogRecord_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "type"
# case = "RotatingFileHandler__shouldRollover__record_as_LogRecord_wrong"
# subject = "logging.handlers.RotatingFileHandler.shouldRollover(record: LogRecord)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.handlers.RotatingFileHandler.shouldRollover(record: LogRecord); call it with the wrong type.

typeshed contract: record is LogRecord. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging.handlers import RotatingFileHandler
obj = object.__new__(RotatingFileHandler)
try:
    obj.shouldRollover(_W())  # record: LogRecord <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_handlers/SMTPHandler__getSubject__record_as_LogRecord_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_handlers_SMTPHandler__getSubject__record_as_LogRecord_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "type"
# case = "SMTPHandler__getSubject__record_as_LogRecord_wrong"
# subject = "logging.handlers.SMTPHandler.getSubject(record: LogRecord)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.handlers.SMTPHandler.getSubject(record: LogRecord); call it with the wrong type.

typeshed contract: record is LogRecord. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging.handlers import SMTPHandler
obj = object.__new__(SMTPHandler)
try:
    obj.getSubject(_W())  # record: LogRecord <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_handlers/SMTPHandler__init__mailhost_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_handlers_SMTPHandler__init__mailhost_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "type"
# case = "SMTPHandler__init__mailhost_as_typed_wrong"
# subject = "logging.handlers.SMTPHandler.__init__(mailhost: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.handlers.SMTPHandler.__init__(mailhost: typed); call it with the wrong type.

typeshed contract: mailhost is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging.handlers import SMTPHandler
try:
    SMTPHandler(_W(), "", None, "")  # mailhost: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_handlers/SocketHandler__init__host_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_handlers_SocketHandler__init__host_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "type"
# case = "SocketHandler__init__host_as_str_wrong"
# subject = "logging.handlers.SocketHandler.__init__(host: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.handlers.SocketHandler.__init__(host: str); call it with the wrong type.

typeshed contract: host is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from logging.handlers import SocketHandler
try:
    SocketHandler(12345, None)  # host: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_handlers/SocketHandler__makePickle__record_as_LogRecord_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_handlers_SocketHandler__makePickle__record_as_LogRecord_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "type"
# case = "SocketHandler__makePickle__record_as_LogRecord_wrong"
# subject = "logging.handlers.SocketHandler.makePickle(record: LogRecord)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.handlers.SocketHandler.makePickle(record: LogRecord); call it with the wrong type.

typeshed contract: record is LogRecord. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging.handlers import SocketHandler
obj = object.__new__(SocketHandler)
try:
    obj.makePickle(_W())  # record: LogRecord <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_handlers/SocketHandler__makeSocket__timeout_as_float_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_handlers_SocketHandler__makeSocket__timeout_as_float_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "type"
# case = "SocketHandler__makeSocket__timeout_as_float_wrong"
# subject = "logging.handlers.SocketHandler.makeSocket(timeout: float)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.handlers.SocketHandler.makeSocket(timeout: float); call it with the wrong type.

typeshed contract: timeout is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from logging.handlers import SocketHandler
obj = object.__new__(SocketHandler)
try:
    obj.makeSocket("not_a_float")  # timeout: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_handlers/SocketHandler__send__s_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_handlers_SocketHandler__send__s_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "type"
# case = "SocketHandler__send__s_as_ReadableBuffer_wrong"
# subject = "logging.handlers.SocketHandler.send(s: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.handlers.SocketHandler.send(s: ReadableBuffer); call it with the wrong type.

typeshed contract: s is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging.handlers import SocketHandler
obj = object.__new__(SocketHandler)
try:
    obj.send(_W())  # s: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_handlers/SysLogHandler__encodePriority__facility_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_handlers_SysLogHandler__encodePriority__facility_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "type"
# case = "SysLogHandler__encodePriority__facility_as_typed_wrong"
# subject = "logging.handlers.SysLogHandler.encodePriority(facility: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.handlers.SysLogHandler.encodePriority(facility: typed); call it with the wrong type.

typeshed contract: facility is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging.handlers import SysLogHandler
obj = object.__new__(SysLogHandler)
try:
    obj.encodePriority(_W(), None)  # facility: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_handlers/SysLogHandler__init__address_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_handlers_SysLogHandler__init__address_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "type"
# case = "SysLogHandler__init__address_as_typed_wrong"
# subject = "logging.handlers.SysLogHandler.__init__(address: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.handlers.SysLogHandler.__init__(address: typed); call it with the wrong type.

typeshed contract: address is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging.handlers import SysLogHandler
try:
    SysLogHandler(_W())  # address: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_handlers/SysLogHandler__mapPriority__levelName_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_handlers_SysLogHandler__mapPriority__levelName_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "type"
# case = "SysLogHandler__mapPriority__levelName_as_str_wrong"
# subject = "logging.handlers.SysLogHandler.mapPriority(levelName: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.handlers.SysLogHandler.mapPriority(levelName: str); call it with the wrong type.

typeshed contract: levelName is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from logging.handlers import SysLogHandler
obj = object.__new__(SysLogHandler)
try:
    obj.mapPriority(12345)  # levelName: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_handlers/TimedRotatingFileHandler__computeRollover__currentTime_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_handlers_TimedRotatingFileHandler__computeRollover__currentTime_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "type"
# case = "TimedRotatingFileHandler__computeRollover__currentTime_as_int_wrong"
# subject = "logging.handlers.TimedRotatingFileHandler.computeRollover(currentTime: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.handlers.TimedRotatingFileHandler.computeRollover(currentTime: int); call it with the wrong type.

typeshed contract: currentTime is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from logging.handlers import TimedRotatingFileHandler
obj = object.__new__(TimedRotatingFileHandler)
try:
    obj.computeRollover("not_an_int")  # currentTime: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_handlers/TimedRotatingFileHandler__init__filename_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_handlers_TimedRotatingFileHandler__init__filename_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "type"
# case = "TimedRotatingFileHandler__init__filename_as_StrPath_wrong"
# subject = "logging.handlers.TimedRotatingFileHandler.__init__(filename: StrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.handlers.TimedRotatingFileHandler.__init__(filename: StrPath); call it with the wrong type.

typeshed contract: filename is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging.handlers import TimedRotatingFileHandler
try:
    TimedRotatingFileHandler(_W())  # filename: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_handlers/TimedRotatingFileHandler__shouldRollover__record_as_LogRecord_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_handlers_TimedRotatingFileHandler__shouldRollover__record_as_LogRecord_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "type"
# case = "TimedRotatingFileHandler__shouldRollover__record_as_LogRecord_wrong"
# subject = "logging.handlers.TimedRotatingFileHandler.shouldRollover(record: LogRecord)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.handlers.TimedRotatingFileHandler.shouldRollover(record: LogRecord); call it with the wrong type.

typeshed contract: record is LogRecord. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging.handlers import TimedRotatingFileHandler
obj = object.__new__(TimedRotatingFileHandler)
try:
    obj.shouldRollover(_W())  # record: LogRecord <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_handlers/WatchedFileHandler__init__filename_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_handlers_WatchedFileHandler__init__filename_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "type"
# case = "WatchedFileHandler__init__filename_as_StrPath_wrong"
# subject = "logging.handlers.WatchedFileHandler.__init__(filename: StrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.handlers.WatchedFileHandler.__init__(filename: StrPath); call it with the wrong type.

typeshed contract: filename is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging.handlers import WatchedFileHandler
try:
    WatchedFileHandler(_W())  # filename: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

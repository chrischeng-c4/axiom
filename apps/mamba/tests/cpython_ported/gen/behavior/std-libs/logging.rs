use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/logging/base_filter_passes_all.py`.
#[test]
fn test_gen_behavior_std_libs_logging_base_filter_passes_all() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "base_filter_passes_all"
# subject = "logging.Filter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.Filter: a bare logging.Filter() passes every record (filter() returns truthy with no name restriction configured)"""
import logging

flt = logging.Filter()
assert flt.filter(logging.makeLogRecord({"name": "spam.eggs"})), "empty filter passes"
print("base_filter_passes_all OK")
"###);
    assert_output(&out, r###"base_filter_passes_all OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/logging/bufferingformatter_joins_batch.py`.
#[test]
fn test_gen_behavior_std_libs_logging_bufferingformatter_joins_batch() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "bufferingformatter_joins_batch"
# subject = "logging.BufferingFormatter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.BufferingFormatter: BufferingFormatter.format joins a record batch in order ('one','two' -> 'onetwo') and returns '' for an empty batch"""
import logging

records = [
    logging.makeLogRecord({"msg": "one"}),
    logging.makeLogRecord({"msg": "two"}),
]
bf = logging.BufferingFormatter()
assert bf.format([]) == "", "empty batch -> empty string"
assert bf.format(records) == "onetwo", "batch joined in order"
print("bufferingformatter_joins_batch OK")
"###);
    assert_output(&out, r###"bufferingformatter_joins_batch OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/logging/child_propagates_to_parent.py`.
#[test]
fn test_gen_behavior_std_libs_logging_child_propagates_to_parent() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "child_propagates_to_parent"
# subject = "logging.Logger"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.Logger: a record on a dotted child logger propagates up to a handler installed on its ancestor"""
import logging

import io

_stream = io.StringIO()
_h = logging.StreamHandler(_stream)
_h.setLevel(logging.DEBUG)
_parent = logging.getLogger("test.behavior.parent")
_parent.setLevel(logging.DEBUG)
_parent.addHandler(_h)
_child = logging.getLogger("test.behavior.parent.child")
_child.debug("child msg")
_out = _stream.getvalue()
assert "child msg" in _out, f"propagation = {_out!r}"
_parent.removeHandler(_h)
print("child_propagates_to_parent OK")
"###);
    assert_output(&out, r###"child_propagates_to_parent OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/logging/configurator_resolves_cfg_refs.py`.
#[test]
fn test_gen_behavior_std_libs_logging_configurator_resolves_cfg_refs() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "configurator_resolves_cfg_refs"
# subject = "logging.config.BaseConfigurator"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.config.BaseConfigurator: BaseConfigurator.convert resolves cfg:// references by index and key: tuple/list indices, nested index, dotted key, and bracketed key all resolve into the backing dict"""
import logging.config

data = {
    "atuple": (1, 2, 3),
    "alist": ["a", "b", "c"],
    "adict": {"d": "e", "f": 3},
    "nest": ("g", ("h", "i"), "j"),
}
bc = logging.config.BaseConfigurator(data)
assert bc.convert("cfg://atuple[1]") == 2, "tuple index"
assert bc.convert("cfg://alist[1]") == "b", "list index"
assert bc.convert("cfg://nest[1][0]") == "h", "nested index"
assert bc.convert("cfg://adict.d") == "e", "dotted key"
assert bc.convert("cfg://adict[f]") == 3, "bracketed key"
print("configurator_resolves_cfg_refs OK")
"###);
    assert_output(&out, r###"configurator_resolves_cfg_refs OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/logging/exception_includes_traceback.py`.
#[test]
fn test_gen_behavior_std_libs_logging_exception_includes_traceback() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "exception_includes_traceback"
# subject = "logging.Logger"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.Logger: Logger.exception() inside an except block emits both the supplied message and the active exception's type name"""
import logging

import io

_stream = io.StringIO()
_h = logging.StreamHandler(_stream)
_h.setLevel(logging.DEBUG)
_log = logging.getLogger("test.behavior.r6")
_log.setLevel(logging.DEBUG)
_log.addHandler(_h)
try:
    raise ValueError("test_exc")
except ValueError:
    _log.exception("caught it")
_out = _stream.getvalue()
assert "caught it" in _out, f"exception message missing: {_out!r}"
assert "ValueError" in _out, f"exception type missing: {_out!r}"
_log.removeHandler(_h)
print("exception_includes_traceback OK")
"###);
    assert_output(&out, r###"exception_includes_traceback OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/logging/fatal_aliases_critical.py`.
#[test]
fn test_gen_behavior_std_libs_logging_fatal_aliases_critical() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "fatal_aliases_critical"
# subject = "logging.FATAL"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.FATAL: FATAL is an alias for CRITICAL: getLevelName('FATAL') == FATAL == CRITICAL == 50 (issue 27935)"""
import logging

assert logging.getLevelName("FATAL") == logging.FATAL, "FATAL resolves"
assert logging.FATAL == logging.CRITICAL == 50, "FATAL == CRITICAL == 50"
print("fatal_aliases_critical OK")
"###);
    assert_output(&out, r###"fatal_aliases_critical OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/logging/formatter_brace_style.py`.
#[test]
fn test_gen_behavior_std_libs_logging_formatter_brace_style() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "formatter_brace_style"
# subject = "logging.Formatter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.Formatter: Formatter(style='{') formats with str.format placeholders: '{levelname}|{message}' renders 'INFO|hi'"""
import logging

brace = logging.Formatter("{levelname}|{message}", style="{")
out = brace.format(logging.makeLogRecord({"levelname": "INFO", "msg": "hi"}))
assert out == "INFO|hi", f"brace style: {out!r}"
print("formatter_brace_style OK")
"###);
    assert_output(&out, r###"formatter_brace_style OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/logging/formatter_controls_output.py`.
#[test]
fn test_gen_behavior_std_libs_logging_formatter_controls_output() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "formatter_controls_output"
# subject = "logging.Formatter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.Formatter: a Formatter('%(levelname)s|%(message)s') makes an ERROR record render as 'ERROR|formatted_msg' in the handler stream"""
import logging

import io

_stream = io.StringIO()
_h = logging.StreamHandler(_stream)
_fmt = logging.Formatter("%(levelname)s|%(message)s")
_h.setFormatter(_fmt)
_h.setLevel(logging.DEBUG)
_log = logging.getLogger("test.behavior.r3")
_log.setLevel(logging.DEBUG)
_log.addHandler(_h)
_log.error("formatted_msg")
_out = _stream.getvalue()
assert "ERROR|formatted_msg" in _out, f"formatted output = {_out!r}"
_log.removeHandler(_h)
print("formatter_controls_output OK")
"###);
    assert_output(&out, r###"formatter_controls_output OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/logging/getchild_matches_getlogger.py`.
#[test]
fn test_gen_behavior_std_libs_logging_getchild_matches_getlogger() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "getchild_matches_getlogger"
# subject = "logging.Logger"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.Logger: getChild appends to the dotted name; top.getChild('mod') is the same cached instance as getLogger('top.mod'), and chained/dotted-suffix getChild agree"""
import logging

top = logging.getLogger("hier_pkg")
child = top.getChild("mod")
assert child is logging.getLogger("hier_pkg.mod"), "getChild == getLogger(full)"
deep = top.getChild("mod").getChild("sub")
also_deep = top.getChild("mod.sub")
assert deep is logging.getLogger("hier_pkg.mod.sub"), "chained getChild"
assert deep is also_deep, "dotted suffix == chained"
print("getchild_matches_getlogger OK")
"###);
    assert_output(&out, r###"getchild_matches_getlogger OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/logging/getchildren_immediate_only.py`.
#[test]
fn test_gen_behavior_std_libs_logging_getchildren_immediate_only() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "getchildren_immediate_only"
# subject = "logging.Logger"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.Logger: getChildren reports only immediate children (direct child listed, grandchild excluded) and a leaf logger has an empty getChildren set"""
import logging

a = logging.getLogger("hier_tree")
b = logging.getLogger("hier_tree.b")
_grand = logging.getLogger("hier_tree.b.c")
assert b in a.getChildren(), "direct child listed"
direct = {lg.name for lg in a.getChildren()}
assert "hier_tree.b.c" not in direct, "grandchild not listed"
assert _grand.getChildren() == set(), "leaf has empty getChildren"
print("getchildren_immediate_only OK")
"###);
    assert_output(&out, r###"getchildren_immediate_only OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/logging/getlevelname_name_to_number.py`.
#[test]
fn test_gen_behavior_std_libs_logging_getlevelname_name_to_number() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "getlevelname_name_to_number"
# subject = "logging.getLevelName"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.getLevelName: getLevelName resolves a level name to its numeric level: 'INFO' -> INFO, 'WARNING' -> 30"""
import logging

assert logging.getLevelName("INFO") == logging.INFO, "name -> number"
assert logging.getLevelName("WARNING") == logging.WARNING, "WARNING -> 30"
print("getlevelname_name_to_number OK")
"###);
    assert_output(&out, r###"getlevelname_name_to_number OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/logging/getlevelname_number_to_name.py`.
#[test]
fn test_gen_behavior_std_libs_logging_getlevelname_number_to_name() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "getlevelname_number_to_name"
# subject = "logging.getLevelName"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.getLevelName: getLevelName resolves a numeric level back to its name: INFO -> 'INFO', ERROR (40) -> 'ERROR'"""
import logging

assert logging.getLevelName(logging.INFO) == "INFO", "number -> name"
assert logging.getLevelName(logging.ERROR) == "ERROR", "40 -> ERROR"
print("getlevelname_number_to_name OK")
"###);
    assert_output(&out, r###"getlevelname_number_to_name OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/logging/getlevelnamesmapping_fresh_copy.py`.
#[test]
fn test_gen_behavior_std_libs_logging_getlevelnamesmapping_fresh_copy() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "getlevelnamesmapping_fresh_copy"
# subject = "logging.getLevelNamesMapping"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.getLevelNamesMapping: getLevelNamesMapping returns a name->number dict containing the builtins, and hands back a fresh independent copy on every call"""
import logging

mapping = logging.getLevelNamesMapping()
assert mapping["DEBUG"] == logging.DEBUG, "mapping has DEBUG"
assert mapping["CRITICAL"] == logging.CRITICAL, "mapping has CRITICAL"
again = logging.getLevelNamesMapping()
assert mapping is not again, "fresh dict per call"
assert mapping == again, "equal contents"
mapping["BOGUS"] = 999
assert "BOGUS" not in logging.getLevelNamesMapping(), "copy is independent"
print("getlevelnamesmapping_fresh_copy OK")
"###);
    assert_output(&out, r###"getlevelnamesmapping_fresh_copy OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/logging/getlogger_caches_by_name.py`.
#[test]
fn test_gen_behavior_std_libs_logging_getlogger_caches_by_name() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "getlogger_caches_by_name"
# subject = "logging.getLogger"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.getLogger: getLogger(name) returns a logging.Logger, and a second call with the same name returns the identical cached instance"""
import logging

_log = logging.getLogger("test.surface")
assert isinstance(_log, logging.Logger), f"getLogger type = {type(_log)!r}"
_log2 = logging.getLogger("test.surface")
assert _log is _log2, "getLogger caches by name"
print("getlogger_caches_by_name OK")
"###);
    assert_output(&out, r###"getlogger_caches_by_name OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/logging/handler_name_settable.py`.
#[test]
fn test_gen_behavior_std_libs_logging_handler_name_settable() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "handler_name_settable"
# subject = "logging.Handler"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.Handler: Handler.name is a settable attribute that round-trips through assignment and re-assignment"""
import logging

h = logging.Handler()
h.name = "generic"
assert h.name == "generic", "name set once"
h.name = "renamed"
assert h.name == "renamed", "name re-set"
print("handler_name_settable OK")
"###);
    assert_output(&out, r###"handler_name_settable OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/logging/level_constants_canonical_values.py`.
#[test]
fn test_gen_behavior_std_libs_logging_level_constants_canonical_values() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "level_constants_canonical_values"
# subject = "logging.DEBUG"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.DEBUG: the six built-in level constants have their canonical numeric values: DEBUG=10, INFO=20, WARNING=30, ERROR=40, CRITICAL=50, NOTSET=0"""
import logging

assert logging.DEBUG == 10, f"DEBUG = {logging.DEBUG!r}"
assert logging.INFO == 20, f"INFO = {logging.INFO!r}"
assert logging.WARNING == 30, f"WARNING = {logging.WARNING!r}"
assert logging.ERROR == 40, f"ERROR = {logging.ERROR!r}"
assert logging.CRITICAL == 50, f"CRITICAL = {logging.CRITICAL!r}"
assert logging.NOTSET == 0, f"NOTSET = {logging.NOTSET!r}"
print("level_constants_canonical_values OK")
"###);
    assert_output(&out, r###"level_constants_canonical_values OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/logging/logger_drops_below_level.py`.
#[test]
fn test_gen_behavior_std_libs_logging_logger_drops_below_level() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "logger_drops_below_level"
# subject = "logging.Logger"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.Logger: a logger set to WARNING drops DEBUG/INFO records but emits WARNING records to its handler's stream"""
import logging

import io

_stream = io.StringIO()
_h = logging.StreamHandler(_stream)
_h.setLevel(logging.DEBUG)
_log = logging.getLogger("test.behavior.r2")
_log.setLevel(logging.WARNING)
_log.addHandler(_h)
_log.debug("should be dropped")
_log.info("also dropped")
_log.warning("should appear")
_out = _stream.getvalue()
assert "should be dropped" not in _out, "DEBUG dropped at WARNING level"
assert "also dropped" not in _out, "INFO dropped at WARNING level"
assert "should appear" in _out, "WARNING appears"
_log.removeHandler(_h)
print("logger_drops_below_level OK")
"###);
    assert_output(&out, r###"logger_drops_below_level OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/logging/logger_name_attribute.py`.
#[test]
fn test_gen_behavior_std_libs_logging_logger_name_attribute() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "logger_name_attribute"
# subject = "logging.Logger"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.Logger: a logger's .name attribute echoes the dotted name it was fetched with"""
import logging

_named = logging.getLogger("my.module.path")
assert _named.name == "my.module.path", f"Logger.name = {_named.name!r}"
print("logger_name_attribute OK")
"###);
    assert_output(&out, r###"logger_name_attribute OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/logging/makelogrecord_repr_shape.py`.
#[test]
fn test_gen_behavior_std_libs_logging_makelogrecord_repr_shape() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "makelogrecord_repr_shape"
# subject = "logging.makeLogRecord"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.makeLogRecord: str(makeLogRecord({})) has the documented '<LogRecord: ...>' repr shape"""
import logging

rec = logging.makeLogRecord({})
text = str(rec)
assert text.startswith("<LogRecord: "), f"repr prefix: {text!r}"
assert text.endswith(">"), f"repr suffix: {text!r}"
print("makelogrecord_repr_shape OK")
"###);
    assert_output(&out, r###"makelogrecord_repr_shape OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/logging/manager_uses_custom_class.py`.
#[test]
fn test_gen_behavior_std_libs_logging_manager_uses_custom_class() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "manager_uses_custom_class"
# subject = "logging.Manager"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.Manager: a Manager.setLoggerClass(custom) routes getLogger through the custom Logger subclass, and its overridden _log receives the message; setLogRecordFactory stores the handed-in factory"""
import logging

captured = []


class RecordingLogger(logging.Logger):
    def _log(self, level, msg, args, exc_info=None, extra=None, **kw):
        captured.append(msg)


man = logging.Manager(None)
man.setLoggerClass(RecordingLogger)
made = man.getLogger("logger_class_test")
assert type(made) is RecordingLogger, "Manager used custom class"
made.warning("captured-msg")
assert captured == ["captured-msg"], "custom _log received the message"

# setLogRecordFactory stores whatever factory it is handed.
sentinel = object()
man.setLogRecordFactory(sentinel)
assert man.logRecordFactory is sentinel, "factory stored"
print("manager_uses_custom_class OK")
"###);
    assert_output(&out, r###"manager_uses_custom_class OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/logging/nullhandler_discards.py`.
#[test]
fn test_gen_behavior_std_libs_logging_nullhandler_discards() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "nullhandler_discards"
# subject = "logging.NullHandler"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.NullHandler: a logger whose only handler is NullHandler discards records silently without raising"""
import logging

_log = logging.getLogger("test.behavior.null")
_log.setLevel(logging.DEBUG)
_log.addHandler(logging.NullHandler())
_log.info("silent")  # should not raise, just discarded
print("nullhandler_discards OK")
"###);
    assert_output(&out, r###"nullhandler_discards OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/logging/propagate_false_stops.py`.
#[test]
fn test_gen_behavior_std_libs_logging_propagate_false_stops() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "propagate_false_stops"
# subject = "logging.Logger"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.Logger: setting child.propagate=False stops a child record from reaching a handler on the parent"""
import logging

import io

_stream = io.StringIO()
_h = logging.StreamHandler(_stream)
_h.setLevel(logging.DEBUG)
_parent = logging.getLogger("test.behavior.parent5")
_parent.setLevel(logging.DEBUG)
_parent.addHandler(_h)
_child = logging.getLogger("test.behavior.parent5.child")
_child.propagate = False
_child.debug("no propagate")
_out = _stream.getvalue()
assert "no propagate" not in _out, f"propagate=False stops: {_out!r}"
_parent.removeHandler(_h)
_child.propagate = True  # reset shared module state
print("propagate_false_stops OK")
"###);
    assert_output(&out, r###"propagate_false_stops OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/logging/root_child_is_bare_named.py`.
#[test]
fn test_gen_behavior_std_libs_logging_root_child_is_bare_named() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "root_child_is_bare_named"
# subject = "logging.Logger"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.Logger: a child of the root logger has the bare name with no leading dot: root.getChild('x') is getLogger('x')"""
import logging

root = logging.getLogger()
xyz = root.getChild("hier_top")
assert xyz is logging.getLogger("hier_top"), "root child is bare-named"
print("root_child_is_bare_named OK")
"###);
    assert_output(&out, r###"root_child_is_bare_named OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/logging/setlevel_isenabledfor.py`.
#[test]
fn test_gen_behavior_std_libs_logging_setlevel_isenabledfor() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "setlevel_isenabledfor"
# subject = "logging.Logger"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.Logger: after setLevel(WARNING) the logger.level reads WARNING; isEnabledFor is False for DEBUG and True for WARNING and ERROR"""
import logging

_log = logging.getLogger("test.setlevel")
_log.setLevel(logging.WARNING)
assert _log.level == logging.WARNING, f"setLevel = {_log.level!r}"
assert not _log.isEnabledFor(logging.DEBUG), "DEBUG disabled at WARNING"
assert _log.isEnabledFor(logging.WARNING), "WARNING enabled"
assert _log.isEnabledFor(logging.ERROR), "ERROR enabled"
print("setlevel_isenabledfor OK")
"###);
    assert_output(&out, r###"setlevel_isenabledfor OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/logging/setloggerclass_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_logging_setloggerclass_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "setloggerclass_roundtrip"
# subject = "logging.setLoggerClass"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.setLoggerClass: setLoggerClass(MyLogger subclass) round-trips through getLoggerClass and restores the default Logger class"""
import logging

class MyLogger(logging.Logger):
    pass


logging.setLoggerClass(MyLogger)
assert logging.getLoggerClass() is MyLogger, "custom class installed"
logging.setLoggerClass(logging.Logger)
assert logging.getLoggerClass() is logging.Logger, "restored default"
print("setloggerclass_roundtrip OK")
"###);
    assert_output(&out, r###"setloggerclass_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/logging/streamhandler_captures_record.py`.
#[test]
fn test_gen_behavior_std_libs_logging_streamhandler_captures_record() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "streamhandler_captures_record"
# subject = "logging.StreamHandler"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.StreamHandler: a StreamHandler over an io.StringIO captures an emitted INFO record's message into the underlying stream"""
import logging

import io

_stream = io.StringIO()
_handler = logging.StreamHandler(_stream)
_handler.setLevel(logging.DEBUG)
_logger = logging.getLogger("test.behavior.r1")
_logger.setLevel(logging.DEBUG)
_logger.addHandler(_handler)
_logger.info("hello from logger")
_out = _stream.getvalue()
assert "hello from logger" in _out, f"log captured = {_out!r}"
_logger.removeHandler(_handler)
print("streamhandler_captures_record OK")
"###);
    assert_output(&out, r###"streamhandler_captures_record OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/logging/streamhandler_setstream_returns_prior.py`.
#[test]
fn test_gen_behavior_std_libs_logging_streamhandler_setstream_returns_prior() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "streamhandler_setstream_returns_prior"
# subject = "logging.StreamHandler"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.StreamHandler: StreamHandler defaults to sys.stderr; setStream returns the previously attached stream, and returns None when the stream is unchanged"""
import logging

import io
import sys

sh = logging.StreamHandler()
buf = io.StringIO()
old = sh.setStream(buf)
assert old is sys.stderr, "default stream is stderr"
back = sh.setStream(old)
assert back is buf, "setStream returns prior stream"
noop = sh.setStream(old)
assert noop is None, "no-op setStream returns None"
print("streamhandler_setstream_returns_prior OK")
"###);
    assert_output(&out, r###"streamhandler_setstream_returns_prior OK
"###);
}

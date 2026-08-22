use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/logging/base_handler_emit_not_implemented.py`.
#[test]
fn test_gen_errors_std_libs_logging_base_handler_emit_not_implemented() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "errors"
# case = "base_handler_emit_not_implemented"
# subject = "logging.Handler"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.Handler: base_handler_emit_not_implemented (errors)."""
import logging

_raised = False
try:
    logging.Handler().emit(None)
except NotImplementedError:
    _raised = True
assert _raised, "base_handler_emit_not_implemented: expected NotImplementedError"
print("base_handler_emit_not_implemented OK")
"###);
    assert_output(&out, r###"base_handler_emit_not_implemented OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/logging/configurator_bad_cfg_refs_raise.py`.
#[test]
fn test_gen_errors_std_libs_logging_configurator_bad_cfg_refs_raise() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "errors"
# case = "configurator_bad_cfg_refs_raise"
# subject = "logging.config.BaseConfigurator"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.config.BaseConfigurator: bad cfg:// references raise documented errors: an unknown top-level key -> KeyError, a malformed prefix 'cfg://!' -> ValueError, and an out-of-range bracket index -> KeyError"""
import logging.config

data = {"adict": {"d": "e"}}
bc = logging.config.BaseConfigurator(data)
for ref, exc in [("cfg://nosuch", KeyError),
                 ("cfg://!", ValueError),
                 ("cfg://adict[2]", KeyError)]:
    _raised = False
    try:
        bc.convert(ref)
    except exc:
        _raised = True
    assert _raised, f"{ref} should raise {exc.__name__}"
print("configurator_bad_cfg_refs_raise OK")
"###);
    assert_output(&out, r###"configurator_bad_cfg_refs_raise OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/logging/fileconfig_empty_file_runtimeerror.py`.
#[test]
fn test_gen_errors_std_libs_logging_fileconfig_empty_file_runtimeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "errors"
# case = "fileconfig_empty_file_runtimeerror"
# subject = "logging.config.fileConfig"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.config.fileConfig: fileConfig on an empty .ini config file (created inside a TemporaryDirectory) raises RuntimeError"""
import logging.config

import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    path = os.path.join(d, "empty.ini")
    with open(path, "w", encoding="utf-8") as f:
        pass
    _raised = False
    try:
        logging.config.fileConfig(path)
    except RuntimeError:
        _raised = True
    assert _raised, "empty config should raise RuntimeError"
print("fileconfig_empty_file_runtimeerror OK")
"###);
    assert_output(&out, r###"fileconfig_empty_file_runtimeerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/logging/fileconfig_missing_file_filenotfound.py`.
#[test]
fn test_gen_errors_std_libs_logging_fileconfig_missing_file_filenotfound() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "errors"
# case = "fileconfig_missing_file_filenotfound"
# subject = "logging.config.fileConfig"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.config.fileConfig: fileConfig on a path that does not exist (inside a TemporaryDirectory) raises FileNotFoundError"""
import logging.config

import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    missing = os.path.join(d, "does_not_exist.ini")
    _raised = False
    try:
        logging.config.fileConfig(missing)
    except FileNotFoundError:
        _raised = True
    assert _raised, "missing config should raise FileNotFoundError"
print("fileconfig_missing_file_filenotfound OK")
"###);
    assert_output(&out, r###"fileconfig_missing_file_filenotfound OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/logging/setlevel_unknown_name_raises.py`.
#[test]
fn test_gen_errors_std_libs_logging_setlevel_unknown_name_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "errors"
# case = "setlevel_unknown_name_raises"
# subject = "logging.Logger"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.Logger: setlevel_unknown_name_raises (errors)."""
import logging

_raised = False
try:
    logging.getLogger('err.setlevel').setLevel('NO_SUCH_LEVEL')
except ValueError:
    _raised = True
assert _raised, "setlevel_unknown_name_raises: expected ValueError"
print("setlevel_unknown_name_raises OK")
"###);
    assert_output(&out, r###"setlevel_unknown_name_raises OK
"###);
}

# Operational AssertionPass seed for SILENT divergences across
# the `argparse` module identifier surface + `logging` module
# identifier / sentinel surface + `logging.getLogger`
# instance class identity contract + `platform` module
# identifier surface + `string` module identifier surface
# pinned by atomic 202: `argparse` (the documented class /
# exception / formatter / constant identifier surface —
# `Namespace` / `Action` / `ArgumentError` /
# `ArgumentTypeError` / `FileType` / `HelpFormatter` /
# `RawDescriptionHelpFormatter` / `RawTextHelpFormatter` /
# `ArgumentDefaultsHelpFormatter` / `MetavarTypeHelpFormatter`
# / `SUPPRESS` / `OPTIONAL` / `ZERO_OR_MORE` / `ONE_OR_MORE`
# / `REMAINDER` / `PARSER` / `BooleanOptionalAction`),
# `logging` (the documented class / helper / sentinel
# identifier surface — `Logger` / `Handler` / `StreamHandler`
# / `FileHandler` / `Formatter` / `Filter` / `LogRecord` /
# `exception` / `NOTSET` / `WARN` / `FATAL` /
# `captureWarnings` / `getLevelName` / `addLevelName` /
# `getHandlerByName` / `getHandlerNames` / `shutdown` /
# `lastResort` / `raiseExceptions` + the documented
# `NOTSET == 0` integer-sentinel value contract +
# the documented `type(logging.getLogger(...)).__name__
# == "Logger"` instance class identity contract — mamba
# collapses to "dict"), `platform` (the documented
# helper identifier surface — `version` /
# `python_implementation` / `python_version_tuple` /
# `python_branch` / `python_compiler` /
# `python_revision` / `python_build` / `architecture` /
# `uname` / `win32_ver` / `mac_ver` / `java_ver` /
# `libc_ver` / `freedesktop_os_release`), and `string`
# (the documented `printable` constant identifier).
#
# The matching subset (partial argparse hasattr,
# partial logging hasattr + integer-sentinel value
# contract + Logger.name round-trip, partial platform
# hasattr + non-empty string return contract,
# partial string hasattr + constant value contract,
# full types hasattr) is covered by
# `test_argparse_logging_platform_string_types_value_ops`;
# this fixture pins the CPython-only contracts that mamba
# currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • hasattr(argparse, "Namespace") is True — documented
#     class identifier (mamba: False);
#   • hasattr(argparse, "Action") is True — documented
#     class identifier (mamba: False);
#   • hasattr(argparse, "ArgumentError") is True —
#     documented exception identifier (mamba: False);
#   • hasattr(argparse, "ArgumentTypeError") is True —
#     documented exception identifier (mamba: False);
#   • hasattr(argparse, "FileType") is True — documented
#     class identifier (mamba: False);
#   • hasattr(argparse, "HelpFormatter") is True —
#     documented class identifier (mamba: False);
#   • hasattr(argparse, "RawDescriptionHelpFormatter")
#     is True — documented class identifier
#     (mamba: False);
#   • hasattr(argparse, "RawTextHelpFormatter") is True —
#     documented class identifier (mamba: False);
#   • hasattr(argparse, "ArgumentDefaultsHelpFormatter")
#     is True — documented class identifier
#     (mamba: False);
#   • hasattr(argparse, "MetavarTypeHelpFormatter") is
#     True — documented class identifier (mamba: False);
#   • hasattr(argparse, "SUPPRESS") is True — documented
#     constant identifier (mamba: False);
#   • hasattr(argparse, "OPTIONAL") is True — documented
#     constant identifier (mamba: False);
#   • hasattr(argparse, "ZERO_OR_MORE") is True —
#     documented constant identifier (mamba: False);
#   • hasattr(argparse, "ONE_OR_MORE") is True —
#     documented constant identifier (mamba: False);
#   • hasattr(argparse, "REMAINDER") is True —
#     documented constant identifier (mamba: False);
#   • hasattr(argparse, "PARSER") is True — documented
#     constant identifier (mamba: False);
#   • hasattr(argparse, "BooleanOptionalAction") is True
#     — documented class identifier (mamba: False);
#   • hasattr(logging, "Logger") is True — documented
#     class identifier (mamba: False);
#   • hasattr(logging, "Handler") is True — documented
#     class identifier (mamba: False);
#   • hasattr(logging, "StreamHandler") is True —
#     documented class identifier (mamba: False);
#   • hasattr(logging, "FileHandler") is True —
#     documented class identifier (mamba: False);
#   • hasattr(logging, "Formatter") is True — documented
#     class identifier (mamba: False);
#   • hasattr(logging, "Filter") is True — documented
#     class identifier (mamba: False);
#   • hasattr(logging, "LogRecord") is True — documented
#     class identifier (mamba: False);
#   • hasattr(logging, "exception") is True — documented
#     helper identifier (mamba: False);
#   • hasattr(logging, "NOTSET") is True — documented
#     sentinel identifier (mamba: False);
#   • hasattr(logging, "WARN") is True — documented
#     sentinel identifier (mamba: False);
#   • hasattr(logging, "FATAL") is True — documented
#     sentinel identifier (mamba: False);
#   • hasattr(logging, "captureWarnings") is True —
#     documented helper identifier (mamba: False);
#   • hasattr(logging, "getLevelName") is True —
#     documented helper identifier (mamba: False);
#   • hasattr(logging, "addLevelName") is True —
#     documented helper identifier (mamba: False);
#   • hasattr(logging, "getHandlerByName") is True —
#     documented helper identifier (mamba: False);
#   • hasattr(logging, "getHandlerNames") is True —
#     documented helper identifier (mamba: False);
#   • hasattr(logging, "shutdown") is True — documented
#     helper identifier (mamba: False);
#   • hasattr(logging, "lastResort") is True — documented
#     sentinel identifier (mamba: False);
#   • hasattr(logging, "raiseExceptions") is True —
#     documented sentinel identifier (mamba: False);
#   • logging.NOTSET == 0 — documented integer-sentinel
#     value (mamba: divergent / missing);
#   • type(logging.getLogger("x")).__name__ == "Logger"
#     — documented instance class identity
#     (mamba: "dict");
#   • hasattr(platform, "version") is True — documented
#     helper identifier (mamba: False);
#   • hasattr(platform, "python_implementation") is True
#     — documented helper identifier (mamba: False);
#   • hasattr(platform, "python_version_tuple") is True
#     — documented helper identifier (mamba: False);
#   • hasattr(platform, "python_branch") is True —
#     documented helper identifier (mamba: False);
#   • hasattr(platform, "python_compiler") is True —
#     documented helper identifier (mamba: False);
#   • hasattr(platform, "python_revision") is True —
#     documented helper identifier (mamba: False);
#   • hasattr(platform, "python_build") is True —
#     documented helper identifier (mamba: False);
#   • hasattr(platform, "architecture") is True —
#     documented helper identifier (mamba: False);
#   • hasattr(platform, "uname") is True — documented
#     helper identifier (mamba: False);
#   • hasattr(platform, "win32_ver") is True —
#     documented helper identifier (mamba: False);
#   • hasattr(platform, "mac_ver") is True — documented
#     helper identifier (mamba: False);
#   • hasattr(platform, "java_ver") is True — documented
#     helper identifier (mamba: False);
#   • hasattr(platform, "libc_ver") is True —
#     documented helper identifier (mamba: False);
#   • hasattr(platform, "freedesktop_os_release") is True
#     — documented helper identifier (mamba: False);
#   • hasattr(string, "printable") is True —
#     documented constant identifier (mamba: False).
import argparse as _argparse_mod
import logging as _logging_mod
import platform as _platform_mod
import string as _string_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# class identity / module-attribute identifier behavior that
# mamba's bundled type stubs do not surface accurately.
argparse: Any = _argparse_mod
logging: Any = _logging_mod
platform: Any = _platform_mod
string: Any = _string_mod


_ledger: list[int] = []

# 1) argparse — module identifier surface
assert hasattr(argparse, "Namespace") == True; _ledger.append(1)
assert hasattr(argparse, "Action") == True; _ledger.append(1)
assert hasattr(argparse, "ArgumentError") == True; _ledger.append(1)
assert hasattr(argparse, "ArgumentTypeError") == True; _ledger.append(1)
assert hasattr(argparse, "FileType") == True; _ledger.append(1)
assert hasattr(argparse, "HelpFormatter") == True; _ledger.append(1)
assert hasattr(argparse, "RawDescriptionHelpFormatter") == True; _ledger.append(1)
assert hasattr(argparse, "RawTextHelpFormatter") == True; _ledger.append(1)
assert hasattr(argparse, "ArgumentDefaultsHelpFormatter") == True; _ledger.append(1)
assert hasattr(argparse, "MetavarTypeHelpFormatter") == True; _ledger.append(1)
assert hasattr(argparse, "SUPPRESS") == True; _ledger.append(1)
assert hasattr(argparse, "OPTIONAL") == True; _ledger.append(1)
assert hasattr(argparse, "ZERO_OR_MORE") == True; _ledger.append(1)
assert hasattr(argparse, "ONE_OR_MORE") == True; _ledger.append(1)
assert hasattr(argparse, "REMAINDER") == True; _ledger.append(1)
assert hasattr(argparse, "PARSER") == True; _ledger.append(1)
assert hasattr(argparse, "BooleanOptionalAction") == True; _ledger.append(1)

# 2) logging — module identifier surface
assert hasattr(logging, "Logger") == True; _ledger.append(1)
assert hasattr(logging, "Handler") == True; _ledger.append(1)
assert hasattr(logging, "StreamHandler") == True; _ledger.append(1)
assert hasattr(logging, "FileHandler") == True; _ledger.append(1)
assert hasattr(logging, "Formatter") == True; _ledger.append(1)
assert hasattr(logging, "Filter") == True; _ledger.append(1)
assert hasattr(logging, "LogRecord") == True; _ledger.append(1)
assert hasattr(logging, "exception") == True; _ledger.append(1)
assert hasattr(logging, "NOTSET") == True; _ledger.append(1)
assert hasattr(logging, "WARN") == True; _ledger.append(1)
assert hasattr(logging, "FATAL") == True; _ledger.append(1)
assert hasattr(logging, "captureWarnings") == True; _ledger.append(1)
assert hasattr(logging, "getLevelName") == True; _ledger.append(1)
assert hasattr(logging, "addLevelName") == True; _ledger.append(1)
assert hasattr(logging, "getHandlerByName") == True; _ledger.append(1)
assert hasattr(logging, "getHandlerNames") == True; _ledger.append(1)
assert hasattr(logging, "shutdown") == True; _ledger.append(1)
assert hasattr(logging, "lastResort") == True; _ledger.append(1)
assert hasattr(logging, "raiseExceptions") == True; _ledger.append(1)

# 3) logging — sentinel value + Logger instance class identity
assert logging.NOTSET == 0; _ledger.append(1)
_log = logging.getLogger("probe.test.spec")
assert type(_log).__name__ == "Logger"; _ledger.append(1)

# 4) platform — module identifier surface
assert hasattr(platform, "version") == True; _ledger.append(1)
assert hasattr(platform, "python_implementation") == True; _ledger.append(1)
assert hasattr(platform, "python_version_tuple") == True; _ledger.append(1)
assert hasattr(platform, "python_branch") == True; _ledger.append(1)
assert hasattr(platform, "python_compiler") == True; _ledger.append(1)
assert hasattr(platform, "python_revision") == True; _ledger.append(1)
assert hasattr(platform, "python_build") == True; _ledger.append(1)
assert hasattr(platform, "architecture") == True; _ledger.append(1)
assert hasattr(platform, "uname") == True; _ledger.append(1)
assert hasattr(platform, "win32_ver") == True; _ledger.append(1)
assert hasattr(platform, "mac_ver") == True; _ledger.append(1)
assert hasattr(platform, "java_ver") == True; _ledger.append(1)
assert hasattr(platform, "libc_ver") == True; _ledger.append(1)
assert hasattr(platform, "freedesktop_os_release") == True; _ledger.append(1)

# 5) string — module identifier surface
assert hasattr(string, "printable") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_argparse_logging_platform_string_silent {sum(_ledger)} asserts")

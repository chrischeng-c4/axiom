# Operational AssertionPass seed for the value contract of the
# `argparse` / `logging` / `platform` / `string` / `types`
# five-pack pinned to atomic 202: `argparse` (the documented
# partial module-level class identifier hasattr surface —
# `ArgumentParser`), `logging` (the documented partial module-
# level class / function / integer-sentinel identifier
# hasattr surface — `getLogger` / `basicConfig` / `debug` /
# `info` / `warning` / `error` / `critical` / `DEBUG` / `INFO`
# / `WARNING` / `ERROR` / `CRITICAL` + the documented
# DEBUG == 10 / INFO == 20 / WARNING == 30 / ERROR == 40 /
# CRITICAL == 50 integer-value contract + the documented
# `logging.getLogger("name").name == "name"` round-trip),
# `platform` (the documented partial module-level helper
# identifier hasattr surface — `system` / `release` /
# `machine` / `processor` / `node` / `platform` /
# `python_version` + the documented `.system()` /
# `.python_version()` non-empty string return contract),
# `string` (the documented full module-level constant /
# class / helper identifier hasattr surface excluding
# `printable` — `ascii_letters` / `ascii_lowercase` /
# `ascii_uppercase` / `digits` / `hexdigits` / `octdigits`
# / `punctuation` / `whitespace` / `Formatter` /
# `Template` / `capwords` + the documented constant value
# contract — `ascii_letters` ==
# "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ" /
# `digits` == "0123456789" + `capwords("a b c")` ==
# "A B C"), and `types` (the documented full module-level
# class identifier hasattr surface — `FunctionType` /
# `MethodType` / `BuiltinFunctionType` /
# `BuiltinMethodType` / `ModuleType` / `LambdaType` /
# `GeneratorType` / `CoroutineType` / `AsyncGeneratorType`
# / `CodeType` / `CellType` / `MappingProxyType` /
# `SimpleNamespace` / `TracebackType` / `FrameType` /
# `GetSetDescriptorType` / `MemberDescriptorType` /
# `MethodWrapperType` / `ClassMethodDescriptorType` /
# `MethodDescriptorType` / `WrapperDescriptorType` /
# `DynamicClassAttribute` / `GenericAlias` / `UnionType` /
# `EllipsisType` / `NoneType` / `NotImplementedType` /
# `new_class` / `resolve_bases` / `prepare_class`).
#
# Behavioral edges that DIVERGE on mamba
# (hasattr(argparse, "Namespace") / "Action" /
# "ArgumentError" / "ArgumentTypeError" / "FileType" /
# "HelpFormatter" / "RawDescriptionHelpFormatter" /
# "RawTextHelpFormatter" / "ArgumentDefaultsHelpFormatter"
# / "MetavarTypeHelpFormatter" / "SUPPRESS" / "OPTIONAL" /
# "ZERO_OR_MORE" / "ONE_OR_MORE" / "REMAINDER" / "PARSER"
# / "BooleanOptionalAction" all False on mamba,
# hasattr(logging, "Logger") / "Handler" / "StreamHandler"
# / "FileHandler" / "Formatter" / "Filter" / "LogRecord" /
# "exception" / "NOTSET" / "WARN" / "FATAL" /
# "captureWarnings" / "getLevelName" / "addLevelName" /
# "getHandlerByName" / "getHandlerNames" / "shutdown" /
# "lastResort" / "raiseExceptions" all False on mamba,
# logging.NOTSET != 0 on mamba (key missing),
# type(logging.getLogger(...)).__name__ collapses to "dict"
# on mamba instead of "Logger", hasattr(platform,
# "version") / "python_implementation" /
# "python_version_tuple" / "python_branch" /
# "python_compiler" / "python_revision" / "python_build" /
# "architecture" / "uname" / "win32_ver" / "mac_ver" /
# "java_ver" / "libc_ver" / "freedesktop_os_release" all
# False on mamba, hasattr(string, "printable") False on
# mamba) are covered in the matching spec fixture
# `lang_argparse_logging_platform_string_silent`.
import argparse
import logging
import platform
import string
import types


_ledger: list[int] = []

# 1) argparse — partial module hasattr surface
#    (Namespace / Action / ArgumentError / ArgumentTypeError
#    / FileType / HelpFormatter / RawDescriptionHelpFormatter
#    / RawTextHelpFormatter / ArgumentDefaultsHelpFormatter
#    / MetavarTypeHelpFormatter / SUPPRESS / OPTIONAL /
#    ZERO_OR_MORE / ONE_OR_MORE / REMAINDER / PARSER /
#    BooleanOptionalAction all DIVERGE on mamba — moved to
#    spec)
assert hasattr(argparse, "ArgumentParser") == True; _ledger.append(1)

# 2) logging — partial module hasattr surface
#    (Logger / Handler / StreamHandler / FileHandler /
#    Formatter / Filter / LogRecord / exception / NOTSET /
#    WARN / FATAL / captureWarnings / getLevelName /
#    addLevelName / getHandlerByName / getHandlerNames /
#    shutdown / lastResort / raiseExceptions all DIVERGE
#    on mamba — moved to spec)
assert hasattr(logging, "getLogger") == True; _ledger.append(1)
assert hasattr(logging, "basicConfig") == True; _ledger.append(1)
assert hasattr(logging, "debug") == True; _ledger.append(1)
assert hasattr(logging, "info") == True; _ledger.append(1)
assert hasattr(logging, "warning") == True; _ledger.append(1)
assert hasattr(logging, "error") == True; _ledger.append(1)
assert hasattr(logging, "critical") == True; _ledger.append(1)
assert hasattr(logging, "DEBUG") == True; _ledger.append(1)
assert hasattr(logging, "INFO") == True; _ledger.append(1)
assert hasattr(logging, "WARNING") == True; _ledger.append(1)
assert hasattr(logging, "ERROR") == True; _ledger.append(1)
assert hasattr(logging, "CRITICAL") == True; _ledger.append(1)

# 3) logging — integer-sentinel value contract
assert logging.DEBUG == 10; _ledger.append(1)
assert logging.INFO == 20; _ledger.append(1)
assert logging.WARNING == 30; _ledger.append(1)
assert logging.ERROR == 40; _ledger.append(1)
assert logging.CRITICAL == 50; _ledger.append(1)

# 4) logging — Logger.name round-trip
#    (type(...).__name__ collapses to "dict" on mamba — moved
#    to spec)
_log = logging.getLogger("probe.test")
assert _log.name == "probe.test"; _ledger.append(1)

# 5) platform — partial module hasattr surface
#    (version / python_implementation / python_version_tuple
#    / python_branch / python_compiler / python_revision /
#    python_build / architecture / uname / win32_ver /
#    mac_ver / java_ver / libc_ver / freedesktop_os_release
#    all DIVERGE on mamba — moved to spec)
assert hasattr(platform, "system") == True; _ledger.append(1)
assert hasattr(platform, "release") == True; _ledger.append(1)
assert hasattr(platform, "machine") == True; _ledger.append(1)
assert hasattr(platform, "processor") == True; _ledger.append(1)
assert hasattr(platform, "node") == True; _ledger.append(1)
assert hasattr(platform, "platform") == True; _ledger.append(1)
assert hasattr(platform, "python_version") == True; _ledger.append(1)

# 6) platform — non-empty string return contract
assert type(platform.system()).__name__ == "str"; _ledger.append(1)
assert len(platform.system()) > 0; _ledger.append(1)
assert type(platform.python_version()).__name__ == "str"; _ledger.append(1)
assert len(platform.python_version()) > 0; _ledger.append(1)

# 7) string — partial module hasattr surface
#    (printable DIVERGES on mamba — moved to spec)
assert hasattr(string, "ascii_letters") == True; _ledger.append(1)
assert hasattr(string, "ascii_lowercase") == True; _ledger.append(1)
assert hasattr(string, "ascii_uppercase") == True; _ledger.append(1)
assert hasattr(string, "digits") == True; _ledger.append(1)
assert hasattr(string, "hexdigits") == True; _ledger.append(1)
assert hasattr(string, "octdigits") == True; _ledger.append(1)
assert hasattr(string, "punctuation") == True; _ledger.append(1)
assert hasattr(string, "whitespace") == True; _ledger.append(1)
assert hasattr(string, "Formatter") == True; _ledger.append(1)
assert hasattr(string, "Template") == True; _ledger.append(1)
assert hasattr(string, "capwords") == True; _ledger.append(1)

# 8) string — constant value contract
assert string.ascii_letters == "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"; _ledger.append(1)
assert string.ascii_lowercase == "abcdefghijklmnopqrstuvwxyz"; _ledger.append(1)
assert string.ascii_uppercase == "ABCDEFGHIJKLMNOPQRSTUVWXYZ"; _ledger.append(1)
assert string.digits == "0123456789"; _ledger.append(1)
assert string.hexdigits == "0123456789abcdefABCDEF"; _ledger.append(1)
assert string.octdigits == "01234567"; _ledger.append(1)
assert string.capwords("a b c") == "A B C"; _ledger.append(1)

# 9) types — full module class identifier hasattr surface
assert hasattr(types, "FunctionType") == True; _ledger.append(1)
assert hasattr(types, "MethodType") == True; _ledger.append(1)
assert hasattr(types, "BuiltinFunctionType") == True; _ledger.append(1)
assert hasattr(types, "BuiltinMethodType") == True; _ledger.append(1)
assert hasattr(types, "ModuleType") == True; _ledger.append(1)
assert hasattr(types, "LambdaType") == True; _ledger.append(1)
assert hasattr(types, "GeneratorType") == True; _ledger.append(1)
assert hasattr(types, "CoroutineType") == True; _ledger.append(1)
assert hasattr(types, "AsyncGeneratorType") == True; _ledger.append(1)
assert hasattr(types, "CodeType") == True; _ledger.append(1)
assert hasattr(types, "CellType") == True; _ledger.append(1)
assert hasattr(types, "MappingProxyType") == True; _ledger.append(1)
assert hasattr(types, "SimpleNamespace") == True; _ledger.append(1)
assert hasattr(types, "TracebackType") == True; _ledger.append(1)
assert hasattr(types, "FrameType") == True; _ledger.append(1)
assert hasattr(types, "GetSetDescriptorType") == True; _ledger.append(1)
assert hasattr(types, "MemberDescriptorType") == True; _ledger.append(1)
assert hasattr(types, "MethodWrapperType") == True; _ledger.append(1)
assert hasattr(types, "ClassMethodDescriptorType") == True; _ledger.append(1)
assert hasattr(types, "MethodDescriptorType") == True; _ledger.append(1)
assert hasattr(types, "WrapperDescriptorType") == True; _ledger.append(1)
assert hasattr(types, "DynamicClassAttribute") == True; _ledger.append(1)
assert hasattr(types, "GenericAlias") == True; _ledger.append(1)
assert hasattr(types, "UnionType") == True; _ledger.append(1)
assert hasattr(types, "EllipsisType") == True; _ledger.append(1)
assert hasattr(types, "NoneType") == True; _ledger.append(1)
assert hasattr(types, "NotImplementedType") == True; _ledger.append(1)
assert hasattr(types, "new_class") == True; _ledger.append(1)
assert hasattr(types, "resolve_bases") == True; _ledger.append(1)
assert hasattr(types, "prepare_class") == True; _ledger.append(1)

# NB: hasattr(argparse, "Namespace") / "Action" /
# "ArgumentError" / "ArgumentTypeError" / "FileType" /
# "HelpFormatter" / "RawDescriptionHelpFormatter" /
# "RawTextHelpFormatter" / "ArgumentDefaultsHelpFormatter"
# / "MetavarTypeHelpFormatter" / "SUPPRESS" / "OPTIONAL" /
# "ZERO_OR_MORE" / "ONE_OR_MORE" / "REMAINDER" / "PARSER"
# / "BooleanOptionalAction" all False on mamba,
# hasattr(logging, "Logger") / "Handler" / "StreamHandler"
# / "FileHandler" / "Formatter" / "Filter" / "LogRecord" /
# "exception" / "NOTSET" / "WARN" / "FATAL" /
# "captureWarnings" / "getLevelName" / "addLevelName" /
# "getHandlerByName" / "getHandlerNames" / "shutdown" /
# "lastResort" / "raiseExceptions" all False on mamba,
# logging.NOTSET != 0 on mamba (key missing),
# type(logging.getLogger(...)).__name__ collapses to "dict"
# on mamba instead of "Logger", hasattr(platform,
# "version") / "python_implementation" /
# "python_version_tuple" / "python_branch" /
# "python_compiler" / "python_revision" / "python_build" /
# "architecture" / "uname" / "win32_ver" / "mac_ver" /
# "java_ver" / "libc_ver" / "freedesktop_os_release" all
# False on mamba, hasattr(string, "printable") False on
# mamba — all DIVERGE on mamba — moved to the divergence-
# spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_argparse_logging_platform_string_types_value_ops {sum(_ledger)} asserts")

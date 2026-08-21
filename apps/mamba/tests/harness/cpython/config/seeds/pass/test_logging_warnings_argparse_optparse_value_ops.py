# Operational AssertionPass seed for the value contract of the
# `logging` / `warnings` / `argparse` / `optparse` four-pack
# pinned to atomic 191: `logging` (the documented partial
# module-level helper hasattr surface — `getLogger` / `DEBUG`
# / `INFO` / `WARNING` / `ERROR` / `CRITICAL` / `basicConfig`
# / `info` / `debug` / `warning` / `error` / `critical` + the
# documented logging.DEBUG / INFO / WARNING / ERROR / CRITICAL
# integer-level value contract + the documented Logger .name
# attribute contract), `warnings` (the documented full module-
# level helper hasattr surface — `warn` / `warn_explicit` /
# `filterwarnings` / `resetwarnings` / `simplefilter` /
# `catch_warnings` / `showwarning` / `formatwarning` + the
# documented filterwarnings / simplefilter function-type
# contract), `argparse` (the documented partial module-level
# helper hasattr surface — `ArgumentParser`), and `optparse`
# (the documented full module-level helper hasattr surface —
# `OptionParser` / `Option` / `OptionGroup` /
# `OptionValueError` / `OptionError`).
#
# The matching subset between mamba and CPython is the partial
# `logging` module hasattr surface (getLogger / DEBUG / INFO /
# WARNING / ERROR / CRITICAL / basicConfig / info / debug /
# warning / error / critical — the `Logger` / `Handler` /
# `StreamHandler` / `FileHandler` / `Formatter` / `NOTSET` /
# `LogRecord` / `Filter` class / sentinel identifier layer
# DIVERGES) + the integer-level value layer + the .name
# attribute contract (the `type(logging.getLogger(...))
# .__name__ == "Logger"` class-identity layer DIVERGES —
# mamba returns "dict"), the full `warnings` module hasattr
# surface + the function-type layer, the partial `argparse`
# module hasattr surface (`ArgumentParser` — the `Namespace`
# / `Action` / `FileType` / `SUPPRESS` / `OPTIONAL` /
# `ZERO_OR_MORE` / `ONE_OR_MORE` / `REMAINDER` / `PARSER` /
# `ArgumentError` / `ArgumentTypeError` extended class /
# sentinel / exception identifier layer DIVERGES + the
# class-identity / instance-method layer DIVERGES), and the
# full `optparse` module hasattr surface (the `OptionParser`
# class-identity layer DIVERGES — mamba returns "dict").
#
# Surface in this fixture:
#   • logging — partial module hasattr surface (getLogger /
#     DEBUG / INFO / WARNING / ERROR / CRITICAL /
#     basicConfig / info / debug / warning / error /
#     critical);
#   • logging — integer-level value contract (DEBUG=10,
#     INFO=20, WARNING=30, ERROR=40, CRITICAL=50);
#   • logging.getLogger — .name attribute contract;
#   • warnings — full module hasattr surface (warn /
#     warn_explicit / filterwarnings / resetwarnings /
#     simplefilter / catch_warnings / showwarning /
#     formatwarning);
#   • warnings.filterwarnings / simplefilter — function-type
#     contract;
#   • argparse — partial module hasattr surface
#     (ArgumentParser);
#   • optparse — full module hasattr surface (OptionParser /
#     Option / OptionGroup / OptionValueError / OptionError).
#
# Behavioral edges that DIVERGE on mamba
# (hasattr(logging, "Logger") / "Handler" / "StreamHandler"
# / "FileHandler" / "Formatter" / "NOTSET" / "LogRecord" /
# "Filter" all False, type(logging.getLogger("test"))
# .__name__ returns "dict" not "Logger",
# hasattr(argparse, "Namespace") / "Action" / "FileType"
# / "SUPPRESS" / "OPTIONAL" / "ZERO_OR_MORE" /
# "ONE_OR_MORE" / "REMAINDER" / "PARSER" /
# "ArgumentError" / "ArgumentTypeError" all False,
# type(argparse.ArgumentParser()).__name__ returns "dict"
# not "ArgumentParser", argparse.ArgumentParser()
# .add_argument raises AttributeError, type(optparse
# .OptionParser()).__name__ returns "dict" not
# "OptionParser") are covered in the matching spec fixture
# `lang_logging_argparse_optparse_silent`.
import logging
import warnings
import argparse
import optparse


_ledger: list[int] = []

# 1) logging — partial module hasattr surface
#    (Logger / Handler / StreamHandler / FileHandler /
#    Formatter / NOTSET / LogRecord / Filter DIVERGE —
#    moved to spec fixture)
assert hasattr(logging, "getLogger") == True; _ledger.append(1)
assert hasattr(logging, "DEBUG") == True; _ledger.append(1)
assert hasattr(logging, "INFO") == True; _ledger.append(1)
assert hasattr(logging, "WARNING") == True; _ledger.append(1)
assert hasattr(logging, "ERROR") == True; _ledger.append(1)
assert hasattr(logging, "CRITICAL") == True; _ledger.append(1)
assert hasattr(logging, "basicConfig") == True; _ledger.append(1)
assert hasattr(logging, "info") == True; _ledger.append(1)
assert hasattr(logging, "debug") == True; _ledger.append(1)
assert hasattr(logging, "warning") == True; _ledger.append(1)
assert hasattr(logging, "error") == True; _ledger.append(1)
assert hasattr(logging, "critical") == True; _ledger.append(1)

# 2) logging — integer-level value contract
assert logging.DEBUG == 10; _ledger.append(1)
assert logging.INFO == 20; _ledger.append(1)
assert logging.WARNING == 30; _ledger.append(1)
assert logging.ERROR == 40; _ledger.append(1)
assert logging.CRITICAL == 50; _ledger.append(1)

# 3) logging.getLogger — .name attribute contract
_log = logging.getLogger("test")
assert _log.name == "test"; _ledger.append(1)

# 4) warnings — full module hasattr surface
assert hasattr(warnings, "warn") == True; _ledger.append(1)
assert hasattr(warnings, "warn_explicit") == True; _ledger.append(1)
assert hasattr(warnings, "filterwarnings") == True; _ledger.append(1)
assert hasattr(warnings, "resetwarnings") == True; _ledger.append(1)
assert hasattr(warnings, "simplefilter") == True; _ledger.append(1)
assert hasattr(warnings, "catch_warnings") == True; _ledger.append(1)
assert hasattr(warnings, "showwarning") == True; _ledger.append(1)
assert hasattr(warnings, "formatwarning") == True; _ledger.append(1)

# 5) warnings — function-type contract
assert type(warnings.filterwarnings).__name__ == "function"; _ledger.append(1)
assert type(warnings.simplefilter).__name__ == "function"; _ledger.append(1)

# 6) argparse — partial module hasattr surface
#    (Namespace / Action / FileType / SUPPRESS / OPTIONAL /
#    ZERO_OR_MORE / ONE_OR_MORE / REMAINDER / PARSER /
#    ArgumentError / ArgumentTypeError DIVERGE — moved to
#    spec fixture)
assert hasattr(argparse, "ArgumentParser") == True; _ledger.append(1)

# 7) optparse — full module hasattr surface
#    (OptionParser class-identity DIVERGES — moved to spec
#    fixture)
assert hasattr(optparse, "OptionParser") == True; _ledger.append(1)
assert hasattr(optparse, "Option") == True; _ledger.append(1)
assert hasattr(optparse, "OptionGroup") == True; _ledger.append(1)
assert hasattr(optparse, "OptionValueError") == True; _ledger.append(1)
assert hasattr(optparse, "OptionError") == True; _ledger.append(1)

# NB: hasattr(logging, "Logger") / "Handler" / "StreamHandler"
# / "FileHandler" / "Formatter" / "NOTSET" / "LogRecord" /
# "Filter" all False on mamba, type(logging.getLogger("test"))
# .__name__ returns "dict" not "Logger" on mamba,
# hasattr(argparse, "Namespace") / "Action" / "FileType" /
# "SUPPRESS" / "OPTIONAL" / "ZERO_OR_MORE" / "ONE_OR_MORE" /
# "REMAINDER" / "PARSER" / "ArgumentError" /
# "ArgumentTypeError" all False on mamba, type(argparse
# .ArgumentParser()).__name__ returns "dict" on mamba,
# argparse.ArgumentParser().add_argument raises AttributeError
# on mamba, type(optparse.OptionParser()).__name__ returns
# "dict" on mamba — all DIVERGE on mamba — moved to the
# divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_logging_warnings_argparse_optparse_value_ops {sum(_ledger)} asserts")

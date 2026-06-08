# Operational AssertionPass seed for SILENT divergences across the
# logging extended module-helper surface + logging.getLogger
# Logger class-identity contract + argparse extended module-
# helper surface + argparse.ArgumentParser class-identity /
# add_argument instance-method contract + optparse.OptionParser
# class-identity contract pinned by atomic 191: `logging` (the
# documented `Logger` / `Handler` / `StreamHandler` /
# `FileHandler` / `Formatter` / `NOTSET` / `LogRecord` /
# `Filter` extended class / function / sentinel identifier
# surface + the documented `type(logging.getLogger("test"))
# .__name__ == "Logger"` class-identity contract), `argparse`
# (the documented `Namespace` / `Action` / `FileType` /
# `SUPPRESS` / `OPTIONAL` / `ZERO_OR_MORE` / `ONE_OR_MORE` /
# `REMAINDER` / `PARSER` / `ArgumentError` /
# `ArgumentTypeError` extended class / sentinel / exception
# identifier surface + the documented `type(argparse
# .ArgumentParser()).__name__ == "ArgumentParser"` class-
# identity contract + the documented ArgumentParser
# .add_argument instance-method contract), and `optparse`
# (the documented `type(optparse.OptionParser()).__name__ ==
# "OptionParser"` class-identity contract).
#
# The matching subset (partial logging hasattr +
# integer-level values + Logger.name attribute, full warnings
# hasattr + function-type, partial argparse hasattr
# (ArgumentParser), full optparse hasattr (OptionParser /
# Option / OptionGroup / OptionValueError / OptionError)) is
# covered by
# `test_logging_warnings_argparse_optparse_value_ops`; this
# fixture pins the CPython-only contracts that mamba currently
# elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • hasattr(logging, "Logger") is True — documented class
#     identifier (mamba: False);
#   • hasattr(logging, "Handler") is True — documented class
#     identifier (mamba: False);
#   • hasattr(logging, "StreamHandler") is True — documented
#     class identifier (mamba: False);
#   • hasattr(logging, "FileHandler") is True — documented
#     class identifier (mamba: False);
#   • hasattr(logging, "Formatter") is True — documented
#     class identifier (mamba: False);
#   • hasattr(logging, "NOTSET") is True — documented
#     sentinel identifier (mamba: False);
#   • hasattr(logging, "LogRecord") is True — documented
#     class identifier (mamba: False);
#   • hasattr(logging, "Filter") is True — documented class
#     identifier (mamba: False);
#   • type(logging.getLogger("test")).__name__ == "Logger" —
#     documented class identity (mamba: returns "dict" —
#     getLogger short-circuits to a dict placeholder that
#     still surfaces .name);
#   • hasattr(argparse, "Namespace") is True — documented
#     class identifier (mamba: False);
#   • hasattr(argparse, "Action") is True — documented class
#     identifier (mamba: False);
#   • hasattr(argparse, "FileType") is True — documented
#     class identifier (mamba: False);
#   • hasattr(argparse, "SUPPRESS") is True — documented
#     sentinel identifier (mamba: False);
#   • hasattr(argparse, "OPTIONAL") is True — documented
#     sentinel identifier (mamba: False);
#   • hasattr(argparse, "ZERO_OR_MORE") is True — documented
#     sentinel identifier (mamba: False);
#   • hasattr(argparse, "ONE_OR_MORE") is True — documented
#     sentinel identifier (mamba: False);
#   • hasattr(argparse, "REMAINDER") is True — documented
#     sentinel identifier (mamba: False);
#   • hasattr(argparse, "PARSER") is True — documented
#     sentinel identifier (mamba: False);
#   • hasattr(argparse, "ArgumentError") is True —
#     documented exception identifier (mamba: False);
#   • hasattr(argparse, "ArgumentTypeError") is True —
#     documented exception identifier (mamba: False);
#   • type(argparse.ArgumentParser()).__name__ ==
#     "ArgumentParser" — documented class identity (mamba:
#     returns "dict" — the constructor short-circuits to a
#     dict placeholder);
#   • hasattr(argparse.ArgumentParser(), "add_argument") is
#     True — documented instance-method identifier (mamba:
#     False — the ArgumentParser instance lacks the bound
#     method);
#   • type(optparse.OptionParser()).__name__ ==
#     "OptionParser" — documented class identity (mamba:
#     returns "dict" — same dict-placeholder pattern).
import logging as _logging_mod
import argparse as _argparse_mod
import optparse as _optparse_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# class identifiers / instance-method / value-contract behavior
# that mamba's bundled type stubs do not surface accurately.
logging: Any = _logging_mod
argparse: Any = _argparse_mod
optparse: Any = _optparse_mod


_ledger: list[int] = []

# 1) logging — extended class / sentinel identifier surface
assert hasattr(logging, "Logger") == True; _ledger.append(1)
assert hasattr(logging, "Handler") == True; _ledger.append(1)
assert hasattr(logging, "StreamHandler") == True; _ledger.append(1)
assert hasattr(logging, "FileHandler") == True; _ledger.append(1)
assert hasattr(logging, "Formatter") == True; _ledger.append(1)
assert hasattr(logging, "NOTSET") == True; _ledger.append(1)
assert hasattr(logging, "LogRecord") == True; _ledger.append(1)
assert hasattr(logging, "Filter") == True; _ledger.append(1)

# 2) logging.getLogger — Logger class-identity contract
_log = logging.getLogger("test")
assert type(_log).__name__ == "Logger"; _ledger.append(1)

# 3) argparse — extended class / sentinel / exception surface
assert hasattr(argparse, "Namespace") == True; _ledger.append(1)
assert hasattr(argparse, "Action") == True; _ledger.append(1)
assert hasattr(argparse, "FileType") == True; _ledger.append(1)
assert hasattr(argparse, "SUPPRESS") == True; _ledger.append(1)
assert hasattr(argparse, "OPTIONAL") == True; _ledger.append(1)
assert hasattr(argparse, "ZERO_OR_MORE") == True; _ledger.append(1)
assert hasattr(argparse, "ONE_OR_MORE") == True; _ledger.append(1)
assert hasattr(argparse, "REMAINDER") == True; _ledger.append(1)
assert hasattr(argparse, "PARSER") == True; _ledger.append(1)
assert hasattr(argparse, "ArgumentError") == True; _ledger.append(1)
assert hasattr(argparse, "ArgumentTypeError") == True; _ledger.append(1)

# 4) argparse.ArgumentParser — class-identity + instance-method
_p = argparse.ArgumentParser()
assert type(_p).__name__ == "ArgumentParser"; _ledger.append(1)
assert hasattr(_p, "add_argument") == True; _ledger.append(1)

# 5) optparse.OptionParser — class-identity contract
_op = optparse.OptionParser()
assert type(_op).__name__ == "OptionParser"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_logging_argparse_optparse_silent {sum(_ledger)} asserts")

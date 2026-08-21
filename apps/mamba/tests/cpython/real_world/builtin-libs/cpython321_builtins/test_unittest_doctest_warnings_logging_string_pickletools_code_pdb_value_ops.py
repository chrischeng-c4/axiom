# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_unittest_doctest_warnings_logging_string_pickletools_code_pdb_value_ops"
# subject = "cpython321.test_unittest_doctest_warnings_logging_string_pickletools_code_pdb_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_unittest_doctest_warnings_logging_string_pickletools_code_pdb_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_unittest_doctest_warnings_logging_string_pickletools_code_pdb_value_ops: execute CPython 3.12 seed test_unittest_doctest_warnings_logging_string_pickletools_code_pdb_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 222 pass conformance — unittest/doctest/warnings/
# logging/string/pickletools/code/pdb hasattr+value contracts
# that match between CPython 3.12 and mamba.
import unittest
import doctest
import warnings
import logging
import string
import pickletools
import code
import pdb

_ledger: list[int] = []

# 1) unittest — conformant hasattr subset
assert hasattr(unittest, "TestCase") == True; _ledger.append(1)
assert hasattr(unittest, "main") == True; _ledger.append(1)
assert hasattr(unittest, "skip") == True; _ledger.append(1)
assert hasattr(unittest, "skipIf") == True; _ledger.append(1)
assert hasattr(unittest, "skipUnless") == True; _ledger.append(1)
assert hasattr(unittest, "expectedFailure") == True; _ledger.append(1)

# 2) doctest — full hasattr surface
assert hasattr(doctest, "DocTest") == True; _ledger.append(1)
assert hasattr(doctest, "DocTestCase") == True; _ledger.append(1)
assert hasattr(doctest, "DocTestSuite") == True; _ledger.append(1)
assert hasattr(doctest, "DocTestFinder") == True; _ledger.append(1)
assert hasattr(doctest, "DocTestParser") == True; _ledger.append(1)
assert hasattr(doctest, "DocTestRunner") == True; _ledger.append(1)
assert hasattr(doctest, "Example") == True; _ledger.append(1)
assert hasattr(doctest, "OutputChecker") == True; _ledger.append(1)
assert hasattr(doctest, "DebugRunner") == True; _ledger.append(1)
assert hasattr(doctest, "testfile") == True; _ledger.append(1)
assert hasattr(doctest, "testmod") == True; _ledger.append(1)
assert hasattr(doctest, "run_docstring_examples") == True; _ledger.append(1)
assert hasattr(doctest, "register_optionflag") == True; _ledger.append(1)
assert hasattr(doctest, "DONT_ACCEPT_BLANKLINE") == True; _ledger.append(1)
assert hasattr(doctest, "DONT_ACCEPT_TRUE_FOR_1") == True; _ledger.append(1)
assert hasattr(doctest, "ELLIPSIS") == True; _ledger.append(1)
assert hasattr(doctest, "FAIL_FAST") == True; _ledger.append(1)
assert hasattr(doctest, "IGNORE_EXCEPTION_DETAIL") == True; _ledger.append(1)
assert hasattr(doctest, "NORMALIZE_WHITESPACE") == True; _ledger.append(1)
assert hasattr(doctest, "REPORT_CDIFF") == True; _ledger.append(1)
assert hasattr(doctest, "REPORT_NDIFF") == True; _ledger.append(1)
assert hasattr(doctest, "REPORT_ONLY_FIRST_FAILURE") == True; _ledger.append(1)
assert hasattr(doctest, "REPORT_UDIFF") == True; _ledger.append(1)
assert hasattr(doctest, "SKIP") == True; _ledger.append(1)
assert hasattr(doctest, "set_unittest_reportflags") == True; _ledger.append(1)
assert hasattr(doctest, "debug") == True; _ledger.append(1)
assert hasattr(doctest, "debug_src") == True; _ledger.append(1)

# 3) warnings — conformant hasattr subset
assert hasattr(warnings, "warn") == True; _ledger.append(1)
assert hasattr(warnings, "warn_explicit") == True; _ledger.append(1)
assert hasattr(warnings, "showwarning") == True; _ledger.append(1)
assert hasattr(warnings, "formatwarning") == True; _ledger.append(1)
assert hasattr(warnings, "filterwarnings") == True; _ledger.append(1)
assert hasattr(warnings, "simplefilter") == True; _ledger.append(1)
assert hasattr(warnings, "resetwarnings") == True; _ledger.append(1)
assert hasattr(warnings, "catch_warnings") == True; _ledger.append(1)
assert hasattr(warnings, "WarningMessage") == True; _ledger.append(1)
assert hasattr(warnings, "filters") == True; _ledger.append(1)
assert hasattr(warnings, "defaultaction") == True; _ledger.append(1)
assert hasattr(warnings, "onceregistry") == True; _ledger.append(1)

# 4) logging — conformant hasattr subset
assert hasattr(logging, "getLogger") == True; _ledger.append(1)
assert hasattr(logging, "basicConfig") == True; _ledger.append(1)
assert hasattr(logging, "DEBUG") == True; _ledger.append(1)
assert hasattr(logging, "INFO") == True; _ledger.append(1)
assert hasattr(logging, "WARNING") == True; _ledger.append(1)
assert hasattr(logging, "ERROR") == True; _ledger.append(1)
assert hasattr(logging, "CRITICAL") == True; _ledger.append(1)

# 5) string — conformant hasattr + value contract
assert hasattr(string, "ascii_lowercase") == True; _ledger.append(1)
assert hasattr(string, "ascii_uppercase") == True; _ledger.append(1)
assert hasattr(string, "ascii_letters") == True; _ledger.append(1)
assert hasattr(string, "digits") == True; _ledger.append(1)
assert hasattr(string, "hexdigits") == True; _ledger.append(1)
assert hasattr(string, "octdigits") == True; _ledger.append(1)
assert hasattr(string, "punctuation") == True; _ledger.append(1)
assert hasattr(string, "whitespace") == True; _ledger.append(1)
assert hasattr(string, "Formatter") == True; _ledger.append(1)
assert hasattr(string, "Template") == True; _ledger.append(1)
assert hasattr(string, "capwords") == True; _ledger.append(1)
assert string.ascii_lowercase == "abcdefghijklmnopqrstuvwxyz"; _ledger.append(1)
assert string.digits == "0123456789"; _ledger.append(1)
assert string.hexdigits == "0123456789abcdefABCDEF"; _ledger.append(1)
assert string.capwords("hello world") == "Hello World"; _ledger.append(1)

# 6) pickletools — conformant hasattr subset
assert hasattr(pickletools, "dis") == True; _ledger.append(1)
assert hasattr(pickletools, "optimize") == True; _ledger.append(1)
assert hasattr(pickletools, "OpcodeInfo") == True; _ledger.append(1)
assert hasattr(pickletools, "ArgumentDescriptor") == True; _ledger.append(1)
assert hasattr(pickletools, "StackObject") == True; _ledger.append(1)

# 7) code — full hasattr surface
assert hasattr(code, "InteractiveInterpreter") == True; _ledger.append(1)
assert hasattr(code, "InteractiveConsole") == True; _ledger.append(1)
assert hasattr(code, "interact") == True; _ledger.append(1)
assert hasattr(code, "compile_command") == True; _ledger.append(1)

# 8) pdb — conformant hasattr subset
assert hasattr(pdb, "Pdb") == True; _ledger.append(1)
assert hasattr(pdb, "run") == True; _ledger.append(1)
assert hasattr(pdb, "runeval") == True; _ledger.append(1)
assert hasattr(pdb, "runctx") == True; _ledger.append(1)
assert hasattr(pdb, "runcall") == True; _ledger.append(1)
assert hasattr(pdb, "set_trace") == True; _ledger.append(1)
assert hasattr(pdb, "post_mortem") == True; _ledger.append(1)
assert hasattr(pdb, "pm") == True; _ledger.append(1)
assert hasattr(pdb, "Restart") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_unittest_doctest_warnings_logging_string_pickletools_code_pdb_value_ops {sum(_ledger)} asserts")

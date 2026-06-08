# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "print"
# dimension = "behavior"
# case = "test_py2_migration_hint__test_stream_redirection_hint_for_py2_migration"
# subject = "cpython.test_print.TestPy2MigrationHint.test_stream_redirection_hint_for_py2_migration"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_print.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_print.py::TestPy2MigrationHint::test_stream_redirection_hint_for_py2_migration
"""Auto-ported test: TestPy2MigrationHint::test_stream_redirection_hint_for_py2_migration (CPython 3.12 oracle)."""


import unittest
import sys
from io import StringIO
from test import support


NotDefined = object()

dispatch = {(False, False, False): lambda args, sep, end, file: print(*args), (False, False, True): lambda args, sep, end, file: print(*args, file=file), (False, True, False): lambda args, sep, end, file: print(*args, end=end), (False, True, True): lambda args, sep, end, file: print(*args, end=end, file=file), (True, False, False): lambda args, sep, end, file: print(*args, sep=sep), (True, False, True): lambda args, sep, end, file: print(*args, sep=sep, file=file), (True, True, False): lambda args, sep, end, file: print(*args, sep=sep, end=end), (True, True, True): lambda args, sep, end, file: print(*args, sep=sep, end=end, file=file)}

class ClassWith__str__:

    def __init__(self, x):
        self.x = x

    def __str__(self):
        return self.x


# --- test body ---
try:
    (print >> sys.stderr, 'message')
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import types as _types_aR
    context = _types_aR.SimpleNamespace(exception=_aR_e)

assert 'Did you mean "print(<message>, file=<output_stream>)"?' in str(context.exception)
try:
    print >> 42
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import types as _types_aR
    context = _types_aR.SimpleNamespace(exception=_aR_e)

assert 'Did you mean "print(<message>, file=<output_stream>)"?' in str(context.exception)
try:
    max >> sys.stderr
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import types as _types_aR
    context = _types_aR.SimpleNamespace(exception=_aR_e)

assert 'Did you mean ' not in str(context.exception)
try:
    print << sys.stderr
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import types as _types_aR
    context = _types_aR.SimpleNamespace(exception=_aR_e)

assert 'Did you mean' not in str(context.exception)

class OverrideRRShift:

    def __rrshift__(self, lhs):
        return 42

assert print >> OverrideRRShift() == 42
print("TestPy2MigrationHint::test_stream_redirection_hint_for_py2_migration: ok")

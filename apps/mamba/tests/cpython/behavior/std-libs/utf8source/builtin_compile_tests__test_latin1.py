# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "utf8source"
# dimension = "behavior"
# case = "builtin_compile_tests__test_latin1"
# subject = "cpython.test_utf8source.BuiltinCompileTests.test_latin1"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_utf8source.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_utf8source.py::BuiltinCompileTests::test_latin1
"""Auto-ported test: BuiltinCompileTests::test_latin1 (CPython 3.12 oracle)."""


import unittest


# --- test body ---
source_code = '# coding: Latin-1\nu = "Ç"\n'.encode('Latin-1')
try:
    code = compile(source_code, '<dummy>', 'exec')
except SyntaxError:

    raise AssertionError('compile() cannot handle Latin-1 source')
ns = {}
exec(code, ns)

assert 'Ç' == ns['u']
print("BuiltinCompileTests::test_latin1: ok")

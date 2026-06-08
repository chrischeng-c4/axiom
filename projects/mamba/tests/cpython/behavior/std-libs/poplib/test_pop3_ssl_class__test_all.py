# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "poplib"
# dimension = "behavior"
# case = "test_pop3_ssl_class__test_all"
# subject = "cpython.test_poplib.TestPOP3_SSLClass.test__all__"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_poplib.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_poplib.py::TestPOP3_SSLClass::test__all__
"""Auto-ported test: TestPOP3_SSLClass::test__all__ (CPython 3.12 oracle)."""


import poplib


assert "POP3" in poplib.__all__, poplib.__all__
assert "error_proto" in poplib.__all__, poplib.__all__

if hasattr(poplib, "POP3_SSL"):
    assert "POP3_SSL" in poplib.__all__, poplib.__all__

print("TestPOP3_SSLClass::test__all__: ok")

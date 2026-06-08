# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "behavior"
# case = "add_type_persists"
# subject = "mimetypes.add_type"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_mimetypes.py"
# status = "filled"
# ///
"""mimetypes.add_type: add_type registers a type so later guess_type calls see it: add_type('application/x-behavior-test', '.btest') then guess_type('document.btest') returns it"""
import mimetypes

mimetypes.add_type("application/x-behavior-test", ".btest")
t, e = mimetypes.guess_type("document.btest")
assert t == "application/x-behavior-test", f"custom type = {t!r}"
assert e is None, f"custom encoding = {e!r}"
print("add_type_persists OK")

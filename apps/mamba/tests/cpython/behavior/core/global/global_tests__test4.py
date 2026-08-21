# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "global"
# dimension = "behavior"
# case = "global_tests__test4"
# subject = "cpython.test_global.GlobalTests.test4"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_global.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""GlobalTests.test4: module-scope global declaration compiles."""

SOURCE = """\
global x
x = 2
"""

code = compile(SOURCE, "<test string>", "exec")
namespace = {}
exec(code, namespace)
assert namespace["x"] == 2, namespace

print("GlobalTests::test4: ok")

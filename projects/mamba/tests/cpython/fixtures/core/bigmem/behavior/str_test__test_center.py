# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "bigmem"
# dimension = "behavior"
# case = "str_test__test_center"
# subject = "cpython.test_bigmem.StrTest.test_center"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bigmem.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_bigmem.py::StrTest::test_center
"""Auto-ported test: StrTest::test_center (CPython 3.12 oracle)."""


import os


size = int(os.environ.get("MAMBA_CPYTHON_BIGMEM_SIZE", "5147"))
substr = " abc def ghi"

value = substr.center(size)
assert len(value) == size, len(value)

lpadsize = rpadsize = (len(value) - len(substr)) // 2
if len(value) % 2:
    lpadsize += 1

assert value[lpadsize:-rpadsize] == substr, (value[lpadsize:-rpadsize], substr)
assert value.strip() == substr.strip(), value.strip()

print("StrTest::test_center: ok")

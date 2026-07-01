# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "skip__reason_as_str_wrong"
# subject = "unittest.case.skip(reason: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.skip(reason: str); call it with the wrong type.

typeshed contract: reason is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unittest.case import skip
try:
    skip(12345)  # reason: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

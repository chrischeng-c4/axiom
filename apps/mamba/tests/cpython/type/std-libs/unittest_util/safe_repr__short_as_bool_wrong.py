# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_util"
# dimension = "type"
# case = "safe_repr__short_as_bool_wrong"
# subject = "unittest.util.safe_repr(short: bool)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed short"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/util.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed short
# mamba-strict-type: TypeError
"""Type wall: unittest.util.safe_repr(short: bool); call it with the wrong type.

typeshed contract: short is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unittest.util import safe_repr
try:
    safe_repr(None, "not_a_bool")  # short: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

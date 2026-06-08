# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_util"
# dimension = "type"
# case = "three_way_cmp__x_as__SupportsComparison_wrong"
# subject = "unittest.util.three_way_cmp(x: _SupportsComparison)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/util.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.util.three_way_cmp(x: _SupportsComparison); call it with the wrong type.

typeshed contract: x is _SupportsComparison. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.util import three_way_cmp
try:
    three_way_cmp(_W(), None)  # x: _SupportsComparison <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_util"
# dimension = "type"
# case = "sorted_list_difference__expected_as_Sequence_wrong"
# subject = "unittest.util.sorted_list_difference(expected: Sequence)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed expected"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/util.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed expected
# mamba-strict-type: TypeError
"""Type wall: unittest.util.sorted_list_difference(expected: Sequence); call it with the wrong type.

typeshed contract: expected is Sequence. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.util import sorted_list_difference
try:
    sorted_list_difference(_W(), None)  # expected: Sequence <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

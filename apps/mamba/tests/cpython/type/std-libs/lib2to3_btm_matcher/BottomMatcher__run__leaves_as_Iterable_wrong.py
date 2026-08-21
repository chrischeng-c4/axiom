# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_btm_matcher"
# dimension = "type"
# case = "BottomMatcher__run__leaves_as_Iterable_wrong"
# subject = "lib2to3.btm_matcher.BottomMatcher.run(leaves: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/btm_matcher.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.btm_matcher.BottomMatcher.run(leaves: Iterable); call it with the wrong type.

typeshed contract: leaves is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lib2to3.btm_matcher import BottomMatcher
obj = object.__new__(BottomMatcher)
try:
    obj.run(_W())  # leaves: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

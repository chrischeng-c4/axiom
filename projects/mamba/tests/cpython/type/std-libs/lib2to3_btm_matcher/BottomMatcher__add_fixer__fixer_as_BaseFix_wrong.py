# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_btm_matcher"
# dimension = "type"
# case = "BottomMatcher__add_fixer__fixer_as_BaseFix_wrong"
# subject = "lib2to3.btm_matcher.BottomMatcher.add_fixer(fixer: BaseFix)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/btm_matcher.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.btm_matcher.BottomMatcher.add_fixer(fixer: BaseFix); call it with the wrong type.

typeshed contract: fixer is BaseFix. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lib2to3.btm_matcher import BottomMatcher
obj = object.__new__(BottomMatcher)
try:
    obj.add_fixer(_W())  # fixer: BaseFix <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

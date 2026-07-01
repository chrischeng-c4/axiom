# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_fixer_base"
# dimension = "type"
# case = "BaseFix__match__node_as__N_wrong"
# subject = "lib2to3.fixer_base.BaseFix.match(node: _N)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/fixer_base.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.fixer_base.BaseFix.match(node: _N); call it with the wrong type.

typeshed contract: node is _N. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lib2to3.fixer_base import BaseFix
obj = object.__new__(BaseFix)
try:
    obj.match(_W())  # node: _N <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

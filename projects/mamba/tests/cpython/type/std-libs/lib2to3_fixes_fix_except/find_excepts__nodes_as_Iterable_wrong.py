# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_fixes_fix_except"
# dimension = "type"
# case = "find_excepts__nodes_as_Iterable_wrong"
# subject = "lib2to3.fixes.fix_except.find_excepts(nodes: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/fixes/fix_except.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.fixes.fix_except.find_excepts(nodes: Iterable); call it with the wrong type.

typeshed contract: nodes is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lib2to3.fixes.fix_except import find_excepts
try:
    find_excepts(_W())  # nodes: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

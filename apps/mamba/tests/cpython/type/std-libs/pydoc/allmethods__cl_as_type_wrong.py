# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pydoc"
# dimension = "type"
# case = "allmethods__cl_as_type_wrong"
# subject = "pydoc.allmethods(cl: type)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed cl"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pydoc.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed cl
# mamba-strict-type: TypeError
"""Type wall: pydoc.allmethods(cl: type); call it with the wrong type.

typeshed contract: cl is type. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pydoc import allmethods
try:
    allmethods(_W())  # cl: type <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

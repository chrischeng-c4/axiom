# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "genericpath"
# dimension = "type"
# case = "commonprefix__m_as_Sequence_wrong"
# subject = "genericpath.commonprefix(m: Sequence)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed m"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/genericpath.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed m
# mamba-strict-type: TypeError
"""Type wall: genericpath.commonprefix(m: Sequence); call it with the wrong type.

typeshed contract: m is Sequence. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from genericpath import commonprefix
try:
    commonprefix(_W())  # m: Sequence <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

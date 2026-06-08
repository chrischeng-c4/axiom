# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "posixpath"
# dimension = "type"
# case = "splitroot__p_as_PathLike_wrong"
# subject = "posixpath.splitroot(p: PathLike)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed p"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/posixpath.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed p
# mamba-strict-type: TypeError
"""Type wall: posixpath.splitroot(p: PathLike); call it with the wrong type.

typeshed contract: p is PathLike. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from posixpath import splitroot
try:
    splitroot(_W())  # p: PathLike <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

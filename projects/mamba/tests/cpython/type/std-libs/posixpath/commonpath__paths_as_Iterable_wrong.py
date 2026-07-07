# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "posixpath"
# dimension = "type"
# case = "commonpath__paths_as_Iterable_wrong"
# subject = "posixpath.commonpath(paths: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/posixpath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: posixpath.commonpath(paths: Iterable); call it with the wrong type.

typeshed contract: paths is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce).

This case pins the literal-container element gap: the outer list is iterable,
but its bare user-class element is still wrong-typed for the path contract."""

class _W:
    pass


from posixpath import commonpath
try:
    commonpath([_W()])  # paths: Iterable <- literal container with wrong-typed element
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

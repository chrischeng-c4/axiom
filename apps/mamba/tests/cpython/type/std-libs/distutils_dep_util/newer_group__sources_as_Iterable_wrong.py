# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_dep_util"
# dimension = "type"
# case = "newer_group__sources_as_Iterable_wrong"
# subject = "distutils.dep_util.newer_group(sources: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/dep_util.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.dep_util.newer_group(sources: Iterable); call it with the wrong type.

typeshed contract: sources is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.dep_util import newer_group
try:
    newer_group(_W(), None)  # sources: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_sysconfig"
# dimension = "type"
# case = "get_python_inc__plat_specific_as_typed_wrong"
# subject = "distutils.sysconfig.get_python_inc(plat_specific: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/sysconfig.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.sysconfig.get_python_inc(plat_specific: typed); call it with the wrong type.

typeshed contract: plat_specific is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.sysconfig import get_python_inc
try:
    get_python_inc(_W())  # plat_specific: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

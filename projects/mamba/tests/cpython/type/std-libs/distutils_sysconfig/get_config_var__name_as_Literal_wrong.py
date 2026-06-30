# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_sysconfig"
# dimension = "type"
# case = "get_config_var__name_as_Literal_wrong"
# subject = "distutils.sysconfig.get_config_var(name: Literal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/sysconfig.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.sysconfig.get_config_var(name: Literal); call it with the wrong type.

typeshed contract: name is Literal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.sysconfig import get_config_var
try:
    get_config_var(_W())  # name: Literal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

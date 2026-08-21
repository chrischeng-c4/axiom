# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_core"
# dimension = "type"
# case = "gen_usage__script_name_as_StrOrBytesPath_wrong"
# subject = "distutils.core.gen_usage(script_name: StrOrBytesPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/core.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.core.gen_usage(script_name: StrOrBytesPath); call it with the wrong type.

typeshed contract: script_name is StrOrBytesPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.core import gen_usage
try:
    gen_usage(_W())  # script_name: StrOrBytesPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

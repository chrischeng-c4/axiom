# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_core"
# dimension = "type"
# case = "run_setup__script_name_as_str_wrong"
# subject = "distutils.core.run_setup(script_name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/core.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.core.run_setup(script_name: str); call it with the wrong type.

typeshed contract: script_name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from distutils.core import run_setup
try:
    run_setup(12345)  # script_name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

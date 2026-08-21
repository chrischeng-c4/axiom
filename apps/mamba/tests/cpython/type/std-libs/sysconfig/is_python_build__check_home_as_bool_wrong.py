# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sysconfig"
# dimension = "type"
# case = "is_python_build__check_home_as_bool_wrong"
# subject = "sysconfig.is_python_build(check_home: bool)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed check_home"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sysconfig.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed check_home
# mamba-strict-type: TypeError
"""Type wall: sysconfig.is_python_build(check_home: bool); call it with the wrong type.

typeshed contract: check_home is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from sysconfig import is_python_build
try:
    is_python_build("not_a_bool")  # check_home: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

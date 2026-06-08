# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "type"
# case = "platform__aliased_as_bool_wrong"
# subject = "platform.platform(aliased: bool)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed aliased"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/platform.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed aliased
# mamba-strict-type: TypeError
"""Type wall: platform.platform(aliased: bool); call it with the wrong type.

typeshed contract: aliased is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from platform import platform
try:
    platform("not_a_bool")  # aliased: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "type"
# case = "getpreferredencoding__do_setlocale_as_bool_wrong"
# subject = "locale.getpreferredencoding(do_setlocale: bool)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/locale.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: locale.getpreferredencoding(do_setlocale: bool); call it with the wrong type.

typeshed contract: do_setlocale is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from locale import getpreferredencoding
try:
    getpreferredencoding("not_a_bool")  # do_setlocale: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

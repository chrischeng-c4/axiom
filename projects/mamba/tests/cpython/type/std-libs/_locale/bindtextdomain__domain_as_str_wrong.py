# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_locale"
# dimension = "type"
# case = "bindtextdomain__domain_as_str_wrong"
# subject = "_locale.bindtextdomain(domain: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_locale.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _locale.bindtextdomain(domain: str); call it with the wrong type.

typeshed contract: domain is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _locale import bindtextdomain
try:
    bindtextdomain(12345, None)  # domain: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

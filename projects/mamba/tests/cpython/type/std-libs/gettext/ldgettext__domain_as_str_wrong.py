# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gettext"
# dimension = "type"
# case = "ldgettext__domain_as_str_wrong"
# subject = "gettext.ldgettext(domain: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/gettext.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: gettext.ldgettext(domain: str); call it with the wrong type.

typeshed contract: domain is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from gettext import ldgettext
try:
    ldgettext(12345, "")  # domain: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

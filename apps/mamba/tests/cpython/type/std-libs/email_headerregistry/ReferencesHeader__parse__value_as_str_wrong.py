# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_headerregistry"
# dimension = "type"
# case = "ReferencesHeader__parse__value_as_str_wrong"
# subject = "email.headerregistry.ReferencesHeader.parse(value: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/headerregistry.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.headerregistry.ReferencesHeader.parse(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.headerregistry import ReferencesHeader
try:
    ReferencesHeader.parse(12345, None)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

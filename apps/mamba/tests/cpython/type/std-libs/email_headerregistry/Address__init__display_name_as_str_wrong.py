# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_headerregistry"
# dimension = "type"
# case = "Address__init__display_name_as_str_wrong"
# subject = "email.headerregistry.Address.__init__(display_name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/headerregistry.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.headerregistry.Address.__init__(display_name: str); call it with the wrong type.

typeshed contract: display_name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.headerregistry import Address
try:
    Address(12345)  # display_name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_headerregistry"
# dimension = "type"
# case = "HeaderRegistry__map_to_type__name_as_str_wrong"
# subject = "email.headerregistry.HeaderRegistry.map_to_type(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/headerregistry.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.headerregistry.HeaderRegistry.map_to_type(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.headerregistry import HeaderRegistry
obj = object.__new__(HeaderRegistry)
try:
    obj.map_to_type(12345, None)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

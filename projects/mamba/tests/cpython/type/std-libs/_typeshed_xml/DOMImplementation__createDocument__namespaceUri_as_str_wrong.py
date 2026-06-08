# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_typeshed_xml"
# dimension = "type"
# case = "DOMImplementation__createDocument__namespaceUri_as_str_wrong"
# subject = "_typeshed.xml.DOMImplementation.createDocument(namespaceUri: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_typeshed/xml.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _typeshed.xml.DOMImplementation.createDocument(namespaceUri: str); call it with the wrong type.

typeshed contract: namespaceUri is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _typeshed.xml import DOMImplementation
obj = object.__new__(DOMImplementation)
try:
    obj.createDocument(12345, "", None)  # namespaceUri: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

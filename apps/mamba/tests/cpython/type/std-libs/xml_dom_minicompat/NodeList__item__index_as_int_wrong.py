# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minicompat"
# dimension = "type"
# case = "NodeList__item__index_as_int_wrong"
# subject = "xml.dom.minicompat.NodeList.item(index: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minicompat.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minicompat.NodeList.item(index: int); call it with the wrong type.

typeshed contract: index is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minicompat import NodeList
obj = object.__new__(NodeList)
try:
    obj.item("not_an_int")  # index: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

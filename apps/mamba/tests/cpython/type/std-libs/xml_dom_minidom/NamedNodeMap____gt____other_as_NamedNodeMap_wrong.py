# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "NamedNodeMap____gt____other_as_NamedNodeMap_wrong"
# subject = "xml.dom.minidom.NamedNodeMap.__gt__(other: NamedNodeMap)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.NamedNodeMap.__gt__(other: NamedNodeMap); call it with the wrong type.

typeshed contract: other is NamedNodeMap. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import NamedNodeMap
obj = object.__new__(NamedNodeMap)
try:
    obj.__gt__(_W())  # other: NamedNodeMap <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

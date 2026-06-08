# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "DOMImplementation__createDocumentType__qualifiedName_as_typed_wrong"
# subject = "xml.dom.minidom.DOMImplementation.createDocumentType(qualifiedName: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.DOMImplementation.createDocumentType(qualifiedName: typed); call it with the wrong type.

typeshed contract: qualifiedName is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.minidom import DOMImplementation
obj = object.__new__(DOMImplementation)
try:
    obj.createDocumentType(_W(), None, None)  # qualifiedName: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

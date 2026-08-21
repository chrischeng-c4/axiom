# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "DOMImplementation__hasFeature__feature_as_str_wrong"
# subject = "xml.dom.minidom.DOMImplementation.hasFeature(feature: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.DOMImplementation.hasFeature(feature: str); call it with the wrong type.

typeshed contract: feature is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minidom import DOMImplementation
obj = object.__new__(DOMImplementation)
try:
    obj.hasFeature(12345, None)  # feature: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

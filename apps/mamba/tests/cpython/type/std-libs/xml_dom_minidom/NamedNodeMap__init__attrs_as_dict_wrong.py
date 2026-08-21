# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "type"
# case = "NamedNodeMap__init__attrs_as_dict_wrong"
# subject = "xml.dom.minidom.NamedNodeMap.__init__(attrs: dict)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed attrs"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/minidom.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed attrs
# mamba-strict-type: TypeError
"""Type wall: xml.dom.minidom.NamedNodeMap.__init__(attrs: dict); call it with the wrong type.

typeshed contract: attrs is dict. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.minidom import NamedNodeMap
try:
    NamedNodeMap(12345, None, None)  # attrs: dict <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

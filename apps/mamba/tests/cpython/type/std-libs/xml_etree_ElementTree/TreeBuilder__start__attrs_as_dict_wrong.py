# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_ElementTree"
# dimension = "type"
# case = "TreeBuilder__start__attrs_as_dict_wrong"
# subject = "xml.etree.ElementTree.TreeBuilder.start(attrs: dict)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed attrs"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/etree/ElementTree.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed attrs
# mamba-strict-type: TypeError
"""Type wall: xml.etree.ElementTree.TreeBuilder.start(attrs: dict); call it with the wrong type.

typeshed contract: attrs is dict. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.etree.ElementTree import TreeBuilder
obj = object.__new__(TreeBuilder)
try:
    obj.start(None, 12345)  # attrs: dict <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_ElementPath"
# dimension = "type"
# case = "findtext__elem_as_Element_wrong"
# subject = "xml.etree.ElementPath.findtext(elem: Element)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed elem"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/etree/ElementPath.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed elem
# mamba-strict-type: TypeError
"""Type wall: xml.etree.ElementPath.findtext(elem: Element); call it with the wrong type.

typeshed contract: elem is Element. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.etree.ElementPath import findtext
try:
    findtext(_W(), "")  # elem: Element <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

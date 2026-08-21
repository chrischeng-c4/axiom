# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_ElementTree"
# dimension = "type"
# case = "Element__extend__elements_as_Iterable_wrong"
# subject = "xml.etree.ElementTree.Element.extend(elements: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/etree/ElementTree.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.etree.ElementTree.Element.extend(elements: Iterable); call it with the wrong type.

typeshed contract: elements is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.etree.ElementTree import Element
obj = object.__new__(Element)
try:
    obj.extend(_W())  # elements: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

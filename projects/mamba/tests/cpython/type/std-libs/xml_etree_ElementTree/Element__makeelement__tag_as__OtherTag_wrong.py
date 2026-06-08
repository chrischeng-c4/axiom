# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_ElementTree"
# dimension = "type"
# case = "Element__makeelement__tag_as__OtherTag_wrong"
# subject = "xml.etree.ElementTree.Element.makeelement(tag: _OtherTag)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed tag"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/etree/ElementTree.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed tag
# mamba-strict-type: TypeError
"""Type wall: xml.etree.ElementTree.Element.makeelement(tag: _OtherTag); call it with the wrong type.

typeshed contract: tag is _OtherTag. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.etree.ElementTree import Element
obj = object.__new__(Element)
try:
    obj.makeelement(_W(), None)  # tag: _OtherTag <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

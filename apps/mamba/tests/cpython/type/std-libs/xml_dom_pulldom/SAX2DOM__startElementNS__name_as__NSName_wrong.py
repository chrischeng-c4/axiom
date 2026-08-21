# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_pulldom"
# dimension = "type"
# case = "SAX2DOM__startElementNS__name_as__NSName_wrong"
# subject = "xml.dom.pulldom.SAX2DOM.startElementNS(name: _NSName)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/pulldom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.pulldom.SAX2DOM.startElementNS(name: _NSName); call it with the wrong type.

typeshed contract: name is _NSName. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.pulldom import SAX2DOM
obj = object.__new__(SAX2DOM)
try:
    obj.startElementNS(_W(), None, None)  # name: _NSName <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

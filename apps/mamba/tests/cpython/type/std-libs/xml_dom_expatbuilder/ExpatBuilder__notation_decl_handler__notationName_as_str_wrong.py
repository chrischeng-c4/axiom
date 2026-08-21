# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_expatbuilder"
# dimension = "type"
# case = "ExpatBuilder__notation_decl_handler__notationName_as_str_wrong"
# subject = "xml.dom.expatbuilder.ExpatBuilder.notation_decl_handler(notationName: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/expatbuilder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.expatbuilder.ExpatBuilder.notation_decl_handler(notationName: str); call it with the wrong type.

typeshed contract: notationName is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.expatbuilder import ExpatBuilder
obj = object.__new__(ExpatBuilder)
try:
    obj.notation_decl_handler(12345, None, "", None)  # notationName: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

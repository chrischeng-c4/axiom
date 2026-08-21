# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_ElementPath"
# dimension = "type"
# case = "get_parent_map__context_as__SelectorContext_wrong"
# subject = "xml.etree.ElementPath.get_parent_map(context: _SelectorContext)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/etree/ElementPath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.etree.ElementPath.get_parent_map(context: _SelectorContext); call it with the wrong type.

typeshed contract: context is _SelectorContext. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.etree.ElementPath import get_parent_map
try:
    get_parent_map(_W())  # context: _SelectorContext <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

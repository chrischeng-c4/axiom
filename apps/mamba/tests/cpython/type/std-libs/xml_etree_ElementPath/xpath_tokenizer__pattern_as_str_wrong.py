# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_ElementPath"
# dimension = "type"
# case = "xpath_tokenizer__pattern_as_str_wrong"
# subject = "xml.etree.ElementPath.xpath_tokenizer(pattern: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/etree/ElementPath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.etree.ElementPath.xpath_tokenizer(pattern: str); call it with the wrong type.

typeshed contract: pattern is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.etree.ElementPath import xpath_tokenizer
try:
    xpath_tokenizer(12345)  # pattern: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

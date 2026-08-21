# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "type"
# case = "grapheme_cluster_break__chr_as_str_wrong"
# subject = "unicodedata.grapheme_cluster_break(chr: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unicodedata.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unicodedata.grapheme_cluster_break(chr: str); call it with the wrong type.

typeshed contract: chr is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unicodedata import grapheme_cluster_break
try:
    grapheme_cluster_break(12345)  # chr: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

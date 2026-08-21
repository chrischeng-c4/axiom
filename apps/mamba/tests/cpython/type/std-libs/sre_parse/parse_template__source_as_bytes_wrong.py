# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sre_parse"
# dimension = "type"
# case = "parse_template__source_as_bytes_wrong"
# subject = "sre_parse.parse_template(source: bytes)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed source"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sre_parse.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed source
# mamba-strict-type: TypeError
"""Type wall: sre_parse.parse_template(source: bytes); call it with the wrong type.

typeshed contract: source is bytes. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from sre_parse import parse_template
try:
    parse_template(12345, None)  # source: bytes <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

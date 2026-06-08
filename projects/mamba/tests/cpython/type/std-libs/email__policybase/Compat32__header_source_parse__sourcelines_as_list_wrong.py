# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__policybase"
# dimension = "type"
# case = "Compat32__header_source_parse__sourcelines_as_list_wrong"
# subject = "email._policybase.Compat32.header_source_parse(sourcelines: list)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed sourcelines"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_policybase.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed sourcelines
# mamba-strict-type: TypeError
"""Type wall: email._policybase.Compat32.header_source_parse(sourcelines: list); call it with the wrong type.

typeshed contract: sourcelines is list. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._policybase import Compat32
obj = object.__new__(Compat32)
try:
    obj.header_source_parse(12345)  # sourcelines: list <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

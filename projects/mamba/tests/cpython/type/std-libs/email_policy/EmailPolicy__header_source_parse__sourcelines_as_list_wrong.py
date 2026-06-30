# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_policy"
# dimension = "type"
# case = "EmailPolicy__header_source_parse__sourcelines_as_list_wrong"
# subject = "email.policy.EmailPolicy.header_source_parse(sourcelines: list)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/policy.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.policy.EmailPolicy.header_source_parse(sourcelines: list); call it with the wrong type.

typeshed contract: sourcelines is list. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.policy import EmailPolicy
obj = object.__new__(EmailPolicy)
try:
    obj.header_source_parse(12345)  # sourcelines: list <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

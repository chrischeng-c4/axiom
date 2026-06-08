# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tabnanny"
# dimension = "type"
# case = "process_tokens__tokens_as_Iterable_wrong"
# subject = "tabnanny.process_tokens(tokens: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tabnanny.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tabnanny.process_tokens(tokens: Iterable); call it with the wrong type.

typeshed contract: tokens is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tabnanny import process_tokens
try:
    process_tokens(_W())  # tokens: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

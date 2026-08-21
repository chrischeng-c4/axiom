# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "quopri"
# dimension = "type"
# case = "decode__input_as__Input_wrong"
# subject = "quopri.decode(input: _Input)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/quopri.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: quopri.decode(input: _Input); call it with the wrong type.

typeshed contract: input is _Input. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from quopri import decode
try:
    decode(_W(), None)  # input: _Input <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

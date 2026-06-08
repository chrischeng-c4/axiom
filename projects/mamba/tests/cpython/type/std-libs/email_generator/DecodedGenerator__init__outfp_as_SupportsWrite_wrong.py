# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_generator"
# dimension = "type"
# case = "DecodedGenerator__init__outfp_as_SupportsWrite_wrong"
# subject = "email.generator.DecodedGenerator.__init__(outfp: SupportsWrite)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed outfp"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/generator.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed outfp
# mamba-strict-type: TypeError
"""Type wall: email.generator.DecodedGenerator.__init__(outfp: SupportsWrite); call it with the wrong type.

typeshed contract: outfp is SupportsWrite. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.generator import DecodedGenerator
try:
    DecodedGenerator(_W())  # outfp: SupportsWrite <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

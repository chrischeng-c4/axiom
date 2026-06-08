# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "type"
# case = "interact__banner_as_typed_wrong"
# subject = "code.interact(banner: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/code.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: code.interact(banner: typed); call it with the wrong type.

typeshed contract: banner is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from code import interact
try:
    interact(_W())  # banner: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "type"
# case = "clear_frames__tb_as_typed_wrong"
# subject = "traceback.clear_frames(tb: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/traceback.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: traceback.clear_frames(tb: typed); call it with the wrong type.

typeshed contract: tb is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from traceback import clear_frames
try:
    clear_frames(_W())  # tb: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

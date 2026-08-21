# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "type"
# case = "Pattern__sub__repl_as_typed_wrong"
# subject = "re.Pattern.sub(repl: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed repl"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/re.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed repl
# mamba-strict-type: TypeError
"""Type wall: re.Pattern.sub(repl: typed); call it with the wrong type.

typeshed contract: repl is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from re import Pattern
obj = object.__new__(Pattern)
try:
    obj.sub(_W(), "")  # repl: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

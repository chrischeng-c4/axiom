# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shlex"
# dimension = "type"
# case = "shlex__error_leader__infile_as_typed_wrong"
# subject = "shlex.shlex.error_leader(infile: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/shlex.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: shlex.shlex.error_leader(infile: typed); call it with the wrong type.

typeshed contract: infile is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from shlex import shlex
obj = object.__new__(shlex)
try:
    obj.error_leader(_W())  # infile: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

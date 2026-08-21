# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_spawn"
# dimension = "type"
# case = "spawn__cmd_as_Iterable_wrong"
# subject = "distutils.spawn.spawn(cmd: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/spawn.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.spawn.spawn(cmd: Iterable); call it with the wrong type.

typeshed contract: cmd is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.spawn import spawn
try:
    spawn(_W())  # cmd: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

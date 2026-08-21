# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shlex"
# dimension = "type"
# case = "join__split_command_as_Iterable_wrong"
# subject = "shlex.join(split_command: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/shlex.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: shlex.join(split_command: Iterable); call it with the wrong type.

typeshed contract: split_command is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from shlex import join
try:
    join(_W())  # split_command: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

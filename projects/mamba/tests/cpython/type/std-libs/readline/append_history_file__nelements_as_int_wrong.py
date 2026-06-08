# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "readline"
# dimension = "type"
# case = "append_history_file__nelements_as_int_wrong"
# subject = "readline.append_history_file(nelements: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/readline.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: readline.append_history_file(nelements: int); call it with the wrong type.

typeshed contract: nelements is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from readline import append_history_file
try:
    append_history_file("not_an_int")  # nelements: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

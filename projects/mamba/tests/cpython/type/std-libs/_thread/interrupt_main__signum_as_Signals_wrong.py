# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_thread"
# dimension = "type"
# case = "interrupt_main__signum_as_Signals_wrong"
# subject = "_thread.interrupt_main(signum: Signals)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_thread.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _thread.interrupt_main(signum: Signals); call it with the wrong type.

typeshed contract: signum is Signals. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _thread import interrupt_main
try:
    interrupt_main(_W())  # signum: Signals <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "type"
# case = "sigwait__sigset_as_Iterable_wrong"
# subject = "signal.sigwait(sigset: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/signal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: signal.sigwait(sigset: Iterable); call it with the wrong type.

typeshed contract: sigset is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from signal import sigwait
try:
    sigwait(_W())  # sigset: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

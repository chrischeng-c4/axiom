# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "type"
# case = "LoggerAdapter__process__kwargs_as_MutableMapping_wrong"
# subject = "logging.LoggerAdapter.process(kwargs: MutableMapping)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed kwargs"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed kwargs
# mamba-strict-type: TypeError
"""Type wall: logging.LoggerAdapter.process(kwargs: MutableMapping); call it with the wrong type.

typeshed contract: kwargs is MutableMapping. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging import LoggerAdapter
obj = object.__new__(LoggerAdapter)
try:
    obj.process(None, _W())  # kwargs: MutableMapping <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

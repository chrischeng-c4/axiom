# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "type"
# case = "Logger__setLevel__level_as__Level_wrong"
# subject = "logging.Logger.setLevel(level: _Level)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.Logger.setLevel(level: _Level); call it with the wrong type.

typeshed contract: level is _Level. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging import Logger
obj = object.__new__(Logger)
try:
    obj.setLevel(_W())  # level: _Level <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "select"
# dimension = "type"
# case = "epoll____new____sizehint_as_int_wrong"
# subject = "select.epoll.__new__(sizehint: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/select.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: select.epoll.__new__(sizehint: int); call it with the wrong type.

typeshed contract: sizehint is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from select import epoll
obj = object.__new__(epoll)
try:
    obj.__new__("not_an_int")  # sizehint: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

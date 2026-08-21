# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncore"
# dimension = "type"
# case = "dispatcher__create_socket__family_as_int_wrong"
# subject = "asyncore.dispatcher.create_socket(family: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncore.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncore.dispatcher.create_socket(family: int); call it with the wrong type.

typeshed contract: family is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from asyncore import dispatcher
obj = object.__new__(dispatcher)
try:
    obj.create_socket("not_an_int")  # family: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

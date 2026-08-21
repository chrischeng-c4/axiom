# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncore"
# dimension = "type"
# case = "dispatcher__log_info__type_as_str_wrong"
# subject = "asyncore.dispatcher.log_info(type: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncore.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncore.dispatcher.log_info(type: str); call it with the wrong type.

typeshed contract: type is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from asyncore import dispatcher
obj = object.__new__(dispatcher)
try:
    obj.log_info(None, 12345)  # type: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

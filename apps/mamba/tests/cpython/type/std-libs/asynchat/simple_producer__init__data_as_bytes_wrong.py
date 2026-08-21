# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asynchat"
# dimension = "type"
# case = "simple_producer__init__data_as_bytes_wrong"
# subject = "asynchat.simple_producer.__init__(data: bytes)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed data"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asynchat.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed data
# mamba-strict-type: TypeError
"""Type wall: asynchat.simple_producer.__init__(data: bytes); call it with the wrong type.

typeshed contract: data is bytes. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from asynchat import simple_producer
try:
    simple_producer(12345)  # data: bytes <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

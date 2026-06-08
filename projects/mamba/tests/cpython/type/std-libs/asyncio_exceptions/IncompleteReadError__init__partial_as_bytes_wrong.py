# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_exceptions"
# dimension = "type"
# case = "IncompleteReadError__init__partial_as_bytes_wrong"
# subject = "asyncio.exceptions.IncompleteReadError.__init__(partial: bytes)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed partial"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/exceptions.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed partial
# mamba-strict-type: TypeError
"""Type wall: asyncio.exceptions.IncompleteReadError.__init__(partial: bytes); call it with the wrong type.

typeshed contract: partial is bytes. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from asyncio.exceptions import IncompleteReadError
try:
    IncompleteReadError(12345, None)  # partial: bytes <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

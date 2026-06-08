# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_connection"
# dimension = "type"
# case = "deliver_challenge__connection_as_Connection_wrong"
# subject = "multiprocessing.connection.deliver_challenge(connection: Connection)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed connection"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/connection.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed connection
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.connection.deliver_challenge(connection: Connection); call it with the wrong type.

typeshed contract: connection is Connection. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.connection import deliver_challenge
try:
    deliver_challenge(_W(), b"")  # connection: Connection <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

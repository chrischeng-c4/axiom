# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_connection"
# dimension = "type"
# case = "Client__address_as__Address_wrong"
# subject = "multiprocessing.connection.Client(address: _Address)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed address"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/connection.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed address
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.connection.Client(address: _Address); call it with the wrong type.

typeshed contract: address is _Address. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.connection import Client
try:
    Client(_W())  # address: _Address <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

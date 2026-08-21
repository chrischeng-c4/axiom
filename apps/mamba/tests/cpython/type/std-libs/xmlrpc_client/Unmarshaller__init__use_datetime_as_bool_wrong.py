# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "Unmarshaller__init__use_datetime_as_bool_wrong"
# subject = "xmlrpc.client.Unmarshaller.__init__(use_datetime: bool)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed use_datetime"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed use_datetime
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.Unmarshaller.__init__(use_datetime: bool); call it with the wrong type.

typeshed contract: use_datetime is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xmlrpc.client import Unmarshaller
try:
    Unmarshaller("not_a_bool")  # use_datetime: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)

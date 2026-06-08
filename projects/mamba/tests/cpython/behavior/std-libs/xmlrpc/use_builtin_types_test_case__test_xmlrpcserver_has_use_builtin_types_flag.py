# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc"
# dimension = "behavior"
# case = "use_builtin_types_test_case__test_xmlrpcserver_has_use_builtin_types_flag"
# subject = "cpython.test_xmlrpc.UseBuiltinTypesTestCase.test_xmlrpcserver_has_use_builtin_types_flag"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_xmlrpc.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import base64
import datetime
import decimal
import sys
import time
import xmlrpc.client as xmlrpclib
import xmlrpc.server
import http.client
import http, http.server
import socket
import threading
import re
import io
import contextlib
server = xmlrpc.server.SimpleXMLRPCServer(('localhost', 0), use_builtin_types=True)
server.server_close()
assert server.use_builtin_types

print("UseBuiltinTypesTestCase::test_xmlrpcserver_has_use_builtin_types_flag: ok")

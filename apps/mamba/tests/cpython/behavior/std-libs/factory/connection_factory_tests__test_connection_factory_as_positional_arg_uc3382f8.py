# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "factory"
# dimension = "behavior"
# case = "connection_factory_tests__test_connection_factory_as_positional_arg_uc3382f8"
# subject = "cpython.test_factory.ConnectionFactoryTests.test_connection_factory_as_positional_arg"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_factory.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import sqlite3 as sqlite
from collections.abc import Sequence

class Factory(sqlite.Connection):

    def __init__(self, *args, **kwargs):
        super(Factory, self).__init__(*args, **kwargs)
con = sqlite.connect(':memory:', 5.0, 0, None, True, Factory)
assert con.isolation_level is None
assert isinstance(con, Factory)

print("ConnectionFactoryTests::test_connection_factory_as_positional_arg: ok")

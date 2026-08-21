# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "factory"
# dimension = "behavior"
# case = "connection_factory_tests__test_connection_factory_relayed_call_uce689d1"
# subject = "cpython.test_factory.ConnectionFactoryTests.test_connection_factory_relayed_call"
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
        kwargs['isolation_level'] = None
        super(Factory, self).__init__(*args, **kwargs)
con = sqlite.connect(':memory:', factory=Factory)
assert con.isolation_level is None
assert isinstance(con, Factory)

print("ConnectionFactoryTests::test_connection_factory_relayed_call: ok")

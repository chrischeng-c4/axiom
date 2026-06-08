# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "surface"
# case = "connect_returns_connection_instance"
# subject = "sqlite3.connect"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sqlite3.connect: connect(':memory:') returns an instance of sqlite3.Connection, whose .cursor() returns an instance of sqlite3.Cursor"""
import sqlite3

conn = sqlite3.connect(":memory:")
assert isinstance(conn, sqlite3.Connection), f"connect type = {type(conn)!r}"
cur = conn.cursor()
assert isinstance(cur, sqlite3.Cursor), f"cursor type = {type(cur)!r}"
conn.close()

print("connect_returns_connection_instance OK")

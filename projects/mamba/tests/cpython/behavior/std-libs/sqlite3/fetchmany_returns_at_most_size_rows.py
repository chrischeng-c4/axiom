# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "behavior"
# case = "fetchmany_returns_at_most_size_rows"
# subject = "sqlite3.Cursor"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sqlite3.Cursor: fetchmany(size) returns at most `size` rows from the result set, in order"""
import sqlite3

conn = sqlite3.connect(":memory:")
conn.execute("CREATE TABLE many (n INTEGER)")
conn.executemany("INSERT INTO many VALUES (?)", [(i,) for i in range(10)])
conn.commit()
cur = conn.cursor()
cur.execute("SELECT n FROM many ORDER BY n")
chunk = cur.fetchmany(3)
assert len(chunk) == 3, f"fetchmany(3) = {len(chunk)!r}"
assert chunk[0][0] == 0, f"first = {chunk[0][0]!r}"
assert chunk[2][0] == 2, f"third = {chunk[2][0]!r}"
conn.close()

print("fetchmany_returns_at_most_size_rows OK")

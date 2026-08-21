# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "errors"
# case = "wrong_param_count_raises_programmingerror"
# subject = "sqlite3.Cursor"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sqlite3.Cursor: binding the wrong number of '?' parameters raises sqlite3.ProgrammingError"""
import sqlite3

conn = sqlite3.connect(":memory:")
cur = conn.cursor()
cur.execute("CREATE TABLE t (a INTEGER, b TEXT)")
_raised = False
try:
    # Two '?' placeholders but only one bound value.
    cur.execute("INSERT INTO t VALUES (?, ?)", (1,))
except sqlite3.ProgrammingError:
    _raised = True
assert _raised, "wrong parameter count raises ProgrammingError"
conn.close()

print("wrong_param_count_raises_programmingerror OK")

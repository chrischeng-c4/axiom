# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "errors"
# case = "malformed_isoformat_raises"
# subject = "datetime.datetime.fromisoformat"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.datetime.fromisoformat: datetime.fromisoformat rejects each of a set of malformed ISO strings with ValueError rather than returning silent None"""
import datetime

for bad in ("", "009-03-04", "200a-12-04", "2009-01-32", "2009-02-29",
            "2020-W25-0", "2020-W25-8"):
    _raised = False
    try:
        datetime.datetime.fromisoformat(bad)
    except ValueError:
        _raised = True
    assert _raised, f"bad iso {bad!r}: expected ValueError"
print("malformed_isoformat_raises OK")

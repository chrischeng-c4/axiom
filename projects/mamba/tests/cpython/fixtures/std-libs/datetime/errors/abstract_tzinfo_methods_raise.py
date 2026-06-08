# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "errors"
# case = "abstract_tzinfo_methods_raise"
# subject = "datetime.tzinfo"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.tzinfo: a bare tzinfo() is abstract: its tzname/utcoffset/dst query methods each raise NotImplementedError"""
import datetime

useless = datetime.tzinfo()
sample = datetime.datetime(2010, 1, 1)
for method in ("tzname", "utcoffset", "dst"):
    _raised = False
    try:
        getattr(useless, method)(sample)
    except NotImplementedError:
        _raised = True
    assert _raised, f"tzinfo.{method}: expected NotImplementedError"
print("abstract_tzinfo_methods_raise OK")

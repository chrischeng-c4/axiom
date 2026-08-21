# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "listcomps"
# dimension = "behavior"
# case = "list_comprehension_test__test_exception_locations"
# subject = "cpython.test_listcomps.ListComprehensionTest.test_exception_locations"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_listcomps.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_listcomps.py::ListComprehensionTest::test_exception_locations
"""Auto-ported test: ListComprehensionTest::test_exception_locations."""


import traceback
from test.support import BrokenIter


class _Case:
    def run(self):
        def init_raises():
            try:
                [x for x in BrokenIter(init_raises=True)]
            except Exception as e:
                return e

        def next_raises():
            try:
                [x for x in BrokenIter(next_raises=True)]
            except Exception as e:
                return e

        def iter_raises():
            try:
                [x for x in BrokenIter(iter_raises=True)]
            except Exception as e:
                return e

        for func, expected in [
            (init_raises, "BrokenIter(init_raises=True)"),
            (next_raises, "BrokenIter(next_raises=True)"),
            (iter_raises, "BrokenIter(iter_raises=True)"),
        ]:
            exc = func()
            frame_summary = traceback.extract_tb(exc.__traceback__)[0]
            indent = 16
            code = func.__code__
            assert frame_summary.lineno == code.co_firstlineno + 2
            assert frame_summary.end_lineno == code.co_firstlineno + 2
            assert frame_summary.line[
                frame_summary.colno - indent: frame_summary.end_colno - indent
            ] == expected


_Case().run()
print("ListComprehensionTest::test_exception_locations: ok")

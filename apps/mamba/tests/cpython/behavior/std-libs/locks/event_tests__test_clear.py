# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locks"
# dimension = "behavior"
# case = "event_tests__test_clear"
# subject = "cpython.test_locks.EventTests.test_clear"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_locks.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import re
import asyncio
import collections
ev = asyncio.Event()
assert not ev.is_set()
ev.set()
assert ev.is_set()
ev.clear()
assert not ev.is_set()

print("EventTests::test_clear: ok")

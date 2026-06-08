# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "thread_info_shape"
# subject = "sys.thread_info"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.thread_info: thread_info is a 3-field struct with a documented name (nt/pthread/pthread-stubs/solaris/None) and lock (semaphore/mutex+cond/None)"""
import sys

ti = sys.thread_info
assert len(ti) == 3, f"thread_info len = {len(ti)!r}"
assert ti.name in ("nt", "pthread", "pthread-stubs", "solaris", None), \
    f"thread_info.name = {ti.name!r}"
assert ti.lock in ("semaphore", "mutex+cond", None), \
    f"thread_info.lock = {ti.lock!r}"
print("thread_info_shape OK")

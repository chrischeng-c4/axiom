# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "behavior"
# case = "disk_usage_total_invariant"
# subject = "shutil.disk_usage"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
"""shutil.disk_usage: disk_usage('/') returns a (total, used, free) named tuple with total > 0 and total == used + free"""
import shutil

du = shutil.disk_usage("/")
assert hasattr(du, "total"), "has total"
assert hasattr(du, "used"), "has used"
assert hasattr(du, "free"), "has free"
assert du.total > 0, f"total > 0: {du.total!r}"
assert du.total == du.used + du.free, "total = used + free"

print("disk_usage_total_invariant OK")

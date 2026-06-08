# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "defaultdict_factory_called_on_missing_only"
# subject = "collections.defaultdict"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.defaultdict: the default_factory is invoked exactly once when a missing key is first read (observable via a logging factory) and not on subsequent reads"""
from collections import defaultdict

log = []
def factory():
    log.append("called")
    return 0

d = defaultdict(factory)
assert d["new_key"] == 0, "factory result used for a missing key"
assert log == ["called"], "factory invoked exactly once"
_ = d["new_key"]
assert log == ["called"], "factory not invoked again for a present key"

print("defaultdict_factory_called_on_missing_only OK")

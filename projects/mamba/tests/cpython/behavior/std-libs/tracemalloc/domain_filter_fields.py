# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "behavior"
# case = "domain_filter_fields"
# subject = "tracemalloc.DomainFilter"
# kind = "semantic"
# xfail = "mamba does not implement the tracemalloc.DomainFilter class (#1261 long-tail stub batch)"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.DomainFilter: DomainFilter(True, 5) exposes inclusive True and domain 5"""
import tracemalloc

# DomainFilter carries inclusive + domain.
d = tracemalloc.DomainFilter(True, 5)
assert d.inclusive is True, "domain filter inclusive"
assert d.domain == 5, "domain value"

print("domain_filter_fields OK")

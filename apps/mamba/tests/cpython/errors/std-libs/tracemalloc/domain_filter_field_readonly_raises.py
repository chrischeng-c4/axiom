# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "errors"
# case = "domain_filter_field_readonly_raises"
# subject = "tracemalloc.DomainFilter"
# kind = "mechanical"
# xfail = "mamba does not implement the tracemalloc.DomainFilter class (#1261 long-tail stub batch)"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.DomainFilter: domain_filter_field_readonly_raises (errors)."""
import tracemalloc
_d = tracemalloc.DomainFilter(True, 5)

_raised = False
try:
    setattr(_d, 'domain', 9)
except AttributeError:
    _raised = True
assert _raised, "domain_filter_field_readonly_raises: expected AttributeError"
print("domain_filter_field_readonly_raises OK")

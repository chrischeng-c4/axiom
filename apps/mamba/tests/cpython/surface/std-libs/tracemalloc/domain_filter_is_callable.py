# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "surface"
# case = "domain_filter_is_callable"
# subject = "tracemalloc.DomainFilter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tracemalloc.DomainFilter: domain_filter_is_callable (surface)."""
import tracemalloc

assert callable(tracemalloc.DomainFilter)
print("domain_filter_is_callable OK")

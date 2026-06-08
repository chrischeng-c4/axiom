# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "mapping_pattern_order_independent"
# subject = "match.mapping_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.mapping_pattern: mapping patterns are order-independent in both subject and pattern"""

# Mapping patterns are order-independent in both subject and pattern.
match {"latency": 1, "bandwidth": 0}:
    case {"bandwidth": b2, "latency": l2}:
        pass
assert b2 == 0 and l2 == 1
print("mapping_pattern_order_independent OK")

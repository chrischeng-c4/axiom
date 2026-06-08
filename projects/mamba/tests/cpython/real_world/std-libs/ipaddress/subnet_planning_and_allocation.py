# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "real_world"
# case = "subnet_planning_and_allocation"
# subject = "ipaddress"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
"""ipaddress: carve a /16 supernet into /24 subnets, classify a batch of addresses as in/out of an allocated block, summarize a contiguous range, and collapse the result back to a minimal cover"""
import ipaddress

# Carve a /16 supernet into /24 subnets.
supernet = ipaddress.IPv4Network("10.20.0.0/16")
subnets = list(supernet.subnets(new_prefix=24))
assert len(subnets) == 256, len(subnets)
assert str(subnets[0]) == "10.20.0.0/24", str(subnets[0])
assert str(subnets[-1]) == "10.20.255.0/24", str(subnets[-1])

# Allocate one /24 and classify a batch of addresses as in/out of it.
allocated = subnets[5]  # 10.20.5.0/24
inside = [ipaddress.ip_address("10.20.5.%d" % d) for d in (1, 42, 254)]
outside = [ipaddress.ip_address(a) for a in ("10.20.6.1", "10.21.0.1", "8.8.8.8")]
assert all(a in allocated for a in inside), "all inside"
assert all(a not in allocated for a in outside), "all outside"

# Summarize a contiguous host range into CIDR blocks.
blocks = list(ipaddress.summarize_address_range(
    ipaddress.IPv4Address("10.20.5.0"),
    ipaddress.IPv4Address("10.20.5.255"),
))
assert [str(b) for b in blocks] == ["10.20.5.0/24"], [str(b) for b in blocks]

# Collapse two adjacent allocations back to a minimal cover.
cover = list(ipaddress.collapse_addresses([subnets[0], subnets[1]]))
assert [str(c) for c in cover] == ["10.20.0.0/23"], [str(c) for c in cover]
print("subnet_planning_and_allocation OK")

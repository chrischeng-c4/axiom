"""Bulk ipaddress.ip_address("192.168.x.x") (Task #69, Wave-6 ship #2).

Predicted regime per scout: compute (pure-string parse → integer
handle alloc, no per-call MbObject for the constructed address).
Wall target >=2.0x — CPython instantiates a full Python class
hierarchy (IPv4Address inherits _BaseAddress + _IPAddressBase +
_TotalOrderingMixin); mamba parses 4 octets into a u32 and emits an
i64 handle. The .version attribute access in the loop body
exercises the class.rs handle-attr branch.

Workload: 10000 ip_address calls × 10 iters. Patterns mix 192.168.x.x
(private) + 8.8.x.x (global) to exercise both is_private branches
implicitly (via .version assertion on the result type).

Per scout sequencing: ipaddress is from-scratch surface (#1474);
this fixture pairs with ipaddress_mod.rs registering 2 dispatchers
+ 3 class shells (IPv4Address / IPv6Address / IPv4Network) at the
same revision. class.rs is wired with 6 attribute accessors for
handle-int receivers (.packed, .compressed, .exploded, .version,
.is_private, .is_global).

Hoist convention (#2097): bind `ipaddress.ip_address` locally.
Mamba import quirk avoidance: separate `import sys` / `import time` /
`import ipaddress` lines (xml.etree Task #56 finding).

# tier: compute
"""

import ipaddress

_ip_address = ipaddress.ip_address

ADDRS = []
for i in range(10000):
    if i % 2 == 0:
        ADDRS.append(f"192.168.{(i >> 8) & 0xFF}.{i & 0xFF}")
    else:
        ADDRS.append(f"8.8.{(i >> 8) & 0xFF}.{i & 0xFF}")
ITERS = 10

acc = 0
for _ in range(ITERS):
    for addr in ADDRS:
        h = _ip_address(addr)
        if h.version == 4:
            acc += 1
print("ip_address_bulk:", acc)

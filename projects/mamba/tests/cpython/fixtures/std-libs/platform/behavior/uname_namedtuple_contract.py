# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "behavior"
# case = "uname_namedtuple_contract"
# subject = "platform.uname"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_platform.py"
# status = "filled"
# ///
"""platform.uname: uname() returns a 6-field named tuple: positional/negative/attribute access agree, _fields/_asdict/_replace behave, and the value round-trips through tuple()/slice/copy"""
import platform

import copy

res = platform.uname()

# Six positional fields, each reachable by name and by index (incl. negatives).
assert len(res) == 6, "uname has 6 fields"
assert res[0] == res.system and res[-6] == res.system, "system at 0 / -6"
assert res[1] == res.node and res[-5] == res.node, "node at 1 / -5"
assert res[2] == res.release and res[-4] == res.release, "release at 2 / -4"
assert res[3] == res.version and res[-3] == res.version, "version at 3 / -3"
assert res[4] == res.machine and res[-2] == res.machine, "machine at 4 / -2"
assert res[5] == res.processor and res[-1] == res.processor, "processor at 5/-1"

# Field names and casting to a plain tuple.
assert res._fields == (
    "system", "node", "release", "version", "machine", "processor"
), "field names"
expected = (res.system, res.node, res.release,
            res.version, res.machine, res.processor)
assert tuple(res) == expected, "tuple() yields the 6 values in order"
assert res[:] == expected, "full slice equals tuple"
assert res[:5] == expected[:5], "partial slice"

# _asdict preserves order and keys.
d = res._asdict()
assert len(d) == 6 and "processor" in d, "asdict has 6 keys incl processor"
assert list(d.values()) == list(expected), "asdict values match"

# _replace overrides named fields and leaves the rest untouched.
new = res._replace(system="S", node="N", release="R",
                   version="V", machine="M")
assert (new.system, new.node, new.release, new.version, new.machine) == \
    ("S", "N", "R", "V", "M"), "replaced fields"
assert new.processor == res.processor, "unreplaced field preserved"

# copy / deepcopy compare equal to the original.
assert copy.copy(res) == res, "shallow copy equal"
assert copy.deepcopy(res) == res, "deep copy equal"

print("uname_namedtuple_contract OK")

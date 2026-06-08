# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "opcode"
# dimension = "behavior"
# case = "specialization_stats_tests__test_specialization_stats"
# subject = "cpython.test__opcode.SpecializationStatsTests.test_specialization_stats"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test__opcode.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test__opcode.py::SpecializationStatsTests::test_specialization_stats
"""Auto-ported test: SpecializationStatsTests::test_specialization_stats."""


import opcode
from test.support.import_helper import import_module


_opcode = import_module("_opcode")

stat_names = ["success", "failure", "hit", "deferred", "miss", "deopt"]
specialized_opcodes = [
    op.lower()
    for op in opcode._specializations
    if opcode._inline_cache_entries[opcode.opmap[op]]
]

assert "load_attr" in specialized_opcodes
assert "binary_subscr" in specialized_opcodes

stats = _opcode.get_specialization_stats()
if stats is not None:
    assert isinstance(stats, dict)
    assert sorted(stats.keys()) == sorted(specialized_opcodes)
    assert sorted(stats["load_attr"].keys()) == sorted(stat_names + ["failure_kinds"])
    for stat_name in stat_names:
        assert isinstance(stats["load_attr"][stat_name], int)
    assert isinstance(stats["load_attr"]["failure_kinds"], tuple)
    for value in stats["load_attr"]["failure_kinds"]:
        assert isinstance(value, int)

print("SpecializationStatsTests::test_specialization_stats: ok")

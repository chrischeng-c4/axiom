# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gc"
# dimension = "behavior"
# case = "self_referential_cycle_soak"
# subject = "gc.collect"
# kind = "semantic"
# xfail = "#1123/#1360: current mamba gc.collect returns 0 for self-referential Python cycles"
# source = "mamba:#1360"
# status = "filled"
# ///
# mamba-xfail: #1123/#1360: current mamba gc.collect returns 0 for self-referential Python cycles

import gc


class SelfReferentialInstance:
    def __init__(self):
        self.me = self


def make_list_cycle():
    cycle = []
    cycle.append(cycle)
    return cycle


def make_dict_cycle():
    cycle = {}
    cycle["self"] = cycle
    return cycle


def make_instance_cycle():
    return SelfReferentialInstance()


gc_was_enabled = gc.isenabled()
if gc_was_enabled:
    gc.disable()

try:
    gc.collect()

    rounds = 8
    minimum_per_round = 3
    total_collected = 0

    for _ in range(rounds):
        list_cycle = make_list_cycle()
        dict_cycle = make_dict_cycle()
        instance_cycle = make_instance_cycle()

        del list_cycle
        del dict_cycle
        del instance_cycle

        collected = gc.collect()
        assert collected >= minimum_per_round, (
            f"expected at least {minimum_per_round} collected objects, got {collected}"
        )
        total_collected += collected

    assert total_collected >= rounds * minimum_per_round, (
        f"expected aggregate collection >= {rounds * minimum_per_round}, got {total_collected}"
    )
finally:
    if gc_was_enabled:
        gc.enable()

print("self_referential_cycle_soak: PASS")

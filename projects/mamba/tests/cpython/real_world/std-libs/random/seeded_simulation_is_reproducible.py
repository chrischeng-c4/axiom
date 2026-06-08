# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "real_world"
# case = "seeded_simulation_is_reproducible"
# subject = "random"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random: a Monte-Carlo style simulation driven by a seeded Random (dice rolls, weighted event picks, a shuffled deck draw, and a without-replacement sample) is fully reproducible: re-running the same seeded pipeline reproduces an identical aggregate summary"""
import random

def simulate(seed):
    """One deterministic Monte-Carlo style run over a seeded generator."""
    rng = random.Random(seed)

    # 1000 dice rolls, tallied per face.
    faces = [0, 0, 0, 0, 0, 0]
    for _ in range(1000):
        faces[rng.randint(1, 6) - 1] += 1

    # Weighted event sampling: "rare" is heavily down-weighted.
    events = ["common", "uncommon", "rare"]
    picks = rng.choices(events, weights=[80, 19, 1], k=500)
    rare_count = picks.count("rare")

    # Shuffle a 52-card deck and read off the top 5.
    deck = list(range(52))
    rng.shuffle(deck)
    top5 = deck[:5]

    # Draw a without-replacement sample from a population.
    drawn = rng.sample(range(100), 10)

    return {
        "faces": faces,
        "rare_count": rare_count,
        "top5": top5,
        "drawn": drawn,
    }

# The same seed reproduces the same aggregate summary exactly.
first = simulate(2026)
second = simulate(2026)
assert first == second, f"seeded simulation not reproducible: {first!r} != {second!r}"

# Sanity: structural invariants hold for the run.
assert sum(first["faces"]) == 1000, f"dice tally = {first['faces']!r}"
assert 0 <= first["rare_count"] <= 500, f"rare_count = {first['rare_count']!r}"
assert len(first["top5"]) == 5 and len(set(first["top5"])) == 5, "deck draw distinct"
assert len(first["drawn"]) == 10 and len(set(first["drawn"])) == 10, "sample distinct"

# A different seed yields a different summary.
assert simulate(99) != first, "different seed should differ"

print("seeded_simulation_is_reproducible OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""Small deterministic allocation churn probe for soak/RSS checks."""
import os

outer = int(os.environ.get("MAMBA_SOAK_OUTER", "120"))
rows = int(os.environ.get("MAMBA_SOAK_ROWS", "32"))
width = int(os.environ.get("MAMBA_SOAK_WIDTH", "96"))

checksum = 0
peak_slots = 0

for turn in range(outer):
    table = [[turn + col for col in range(width)] for _ in range(rows)]
    mapping = {f"{turn}:{idx}": row[idx % width] for idx, row in enumerate(table)}
    checksum += sum(mapping.values()) + len(table) + len(mapping)
    peak_slots = max(peak_slots, rows * width)

assert checksum > 0
print("soak_outer:", outer)
print("soak_peak_slots:", peak_slots)
print("soak_checksum:", checksum)
print("heap_churn_soak: OK")

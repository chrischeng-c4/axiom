# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "security"
# case = "decompression_bomb_bounded"
# subject = "bz2.BZ2Decompressor"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.BZ2Decompressor: a high-ratio decompression bomb is contained by max_length-bounded BZ2Decompressor.decompress under a total budget, with EOFError after the stream ends and trailing attacker bytes routed to unused_data"""
import bz2

# Hostile blob: 4 MiB of a single repeated byte -> tiny compressed payload but
# a very high inflation ratio. Stays well under the 64 MiB uncompressed cap.
PAYLOAD = b"\x00" * (4 * 1024 * 1024)
BOMB = bz2.compress(PAYLOAD, 9)
RATIO = len(PAYLOAD) / len(BOMB)
assert RATIO > 100, f"ratio not high enough to be a bomb: {RATIO}"

# Defense 1: a per-call max_length cap is honored exactly. A single call never
# emits more than the cap even though the stream wants to inflate to 4 MiB.
CHUNK = 64 * 1024
d = bz2.BZ2Decompressor()
first = d.decompress(BOMB, max_length=CHUNK)
assert len(first) <= CHUNK, f"cap breached: {len(first)} > {CHUNK}"
assert d.needs_input is False, "needs_input must stay False while output pending"
assert d.eof is False, "must not be eof after one capped chunk"

# Defense 2: drain under a TOTAL budget. A guard refuses to exceed BUDGET, so a
# real bomb (one whose true size exceeds the budget) would be rejected here
# instead of exhausting memory. For this blob the true size fits the budget.
BUDGET = 8 * 1024 * 1024
total = len(first)
chunks = 1
while not d.eof:
    out = d.decompress(b"", max_length=CHUNK)
    assert len(out) <= CHUNK, f"cap breached mid-drain: {len(out)}"
    total += len(out)
    chunks += 1
    if total > BUDGET:
        raise AssertionError("decompression exceeded budget -- bomb!")
    if chunks > 10000:
        raise AssertionError("runaway loop -- decompressor not progressing")
assert total == len(PAYLOAD), f"reassembled {total} != {len(PAYLOAD)}"

# Defense 3: a finished stream is finished. Further decompress() raises EOFError
# rather than silently returning data or re-reading the bomb.
assert d.eof is True, "stream should be eof after full drain"
for arg in (b"", b"trailing junk"):
    try:
        d.decompress(arg)
        raise AssertionError("expected EOFError after stream end")
    except EOFError:
        pass

# Defense 4: bytes appended after a complete stream are NOT decompressed; they
# are surfaced as unused_data so the caller can decide, not silently inflated.
trailer = b"attacker-appended-bytes"
d2 = bz2.BZ2Decompressor()
head = b""
buf = BOMB + trailer
head += d2.decompress(buf, max_length=CHUNK)
while not d2.eof:
    head += d2.decompress(b"", max_length=CHUNK)
assert head == PAYLOAD, "payload before trailer must match"
assert d2.unused_data == trailer, f"unused_data = {d2.unused_data!r}"

# Defense 5: max_length must be a sane bound. A zero cap yields no output but
# does not finish the stream, so a consumer that wrongly passes 0 cannot be
# tricked into thinking the bomb is done.
d3 = bz2.BZ2Decompressor()
zero = d3.decompress(BOMB, max_length=0)
assert zero == b"", f"max_length=0 should yield b'' got {len(zero)} bytes"
assert d3.eof is False, "zero-cap must not mark eof"

print("decompression_bomb_bounded OK")

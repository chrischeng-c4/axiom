# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "security"
# case = "decompression_bomb_bounded"
# subject = "gzip.GzipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""gzip.GzipFile: a tiny gzip blob expanding >1000x (48 MB payload) is contained by three defenses: bounded read(n) chunks aborting at a 1 MB guard cap, a single size-guarded read(cap) limiting output, and a pre-inflation expansion-ratio budget that declines the bomb up front while benign small input still round-trips"""
import gzip
import io

# A decompression bomb: a tiny compressed blob that expands enormously.
# Highly repetitive data compresses to a fraction of a percent of its
# uncompressed size. We keep the uncompressed payload at 48 MB (<= 64 MB)
# to stay safe in CI while still exercising a >1000x expansion ratio.
UNCOMPRESSED_SIZE = 48 * 1024 * 1024  # 48 MB
PAYLOAD = b"\x00" * UNCOMPRESSED_SIZE
BOMB = gzip.compress(PAYLOAD, compresslevel=9)

# The blob is genuinely tiny relative to what it expands to.
ratio = UNCOMPRESSED_SIZE / len(BOMB)
assert len(BOMB) < 64 * 1024, f"bomb should be tiny, got {len(BOMB)} bytes"
assert ratio > 1000, f"expansion ratio too small: {ratio:.1f}x"

# DEFENSE 1: bounded reads via read(n) let a consumer cap total output and
# stop early. We never hold more than CHUNK bytes from a single read, and we
# abort once a guard cap is hit -- long before the full 48 MB materialises.
CHUNK = 64 * 1024
GUARD_CAP = 1 * 1024 * 1024  # refuse to emit more than 1 MB
total = 0
aborted = False
with gzip.GzipFile(fileobj=io.BytesIO(BOMB), mode="rb") as gf:
    while True:
        block = gf.read(CHUNK)
        if not block:
            break
        assert len(block) <= CHUNK, "read(n) honours its bound"
        total += len(block)
        if total > GUARD_CAP:
            aborted = True
            break
assert aborted, "guard must trip before the whole bomb is read"
assert total <= GUARD_CAP + CHUNK, f"capped near guard, got {total}"

# DEFENSE 2: a size-guarded single read -- pass a max length to read() so even
# a single call cannot return more than the guard.
with gzip.GzipFile(fileobj=io.BytesIO(BOMB), mode="rb") as gf:
    guarded = gf.read(GUARD_CAP)
assert len(guarded) == GUARD_CAP, f"read(cap) limits output, got {len(guarded)}"

# DEFENSE 3: a caller can refuse based on the (known) expansion ratio BEFORE
# inflating. We only call decompress when the ratio is within budget; for the
# bomb the budget check fails, so we never allocate the 48 MB at all.
MAX_ALLOWED_RATIO = 100
declined = False
if ratio > MAX_ALLOWED_RATIO:
    declined = True  # reject hostile blob up front
else:
    gzip.decompress(BOMB)  # unreachable for this bomb
assert declined, "ratio budget must reject the bomb up front"

# Sanity: the same bounded machinery still round-trips an honest small file
# (no false positives on benign input).
small = b"the quick brown fox\n" * 8
ok = gzip.decompress(gzip.compress(small)) == small
assert ok, "benign small payload still decompresses fully"

print("decompression_bomb_bounded OK")

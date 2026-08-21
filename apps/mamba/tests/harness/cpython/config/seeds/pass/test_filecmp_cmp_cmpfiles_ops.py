# Operational AssertionPass seed for the `filecmp` module — the
# stdlib file/directory comparison helpers used by build tools
# (Bazel-style cache lookups), diff scaffolds, dev-loop incremental
# regen comparators, test snapshot pipelines (`pytest-snapshot`,
# `syrupy`), and rsync-style sync utilities. Surface focuses on the
# two-file comparison API — `filecmp.cmp(a, b)` and the multi-file
# `filecmp.cmpfiles(dir1, dir2, names)` — plus the module-level
# attribute surface (`DEFAULT_IGNORES`, `clear_cache`, `dircmp`).
# Streaming-mode `dircmp` instance construction works on mamba but
# its instance attributes (`left_only`, `right_only`, `common`,
# `diff_files`) are not yet a stable contract — those are left to a
# later fixture once the runtime stabilises. No fixture coverage yet
# for filecmp.
#
# Surface:
#   • filecmp.cmp(p1, p2) → bool
#       — True iff files are byte-equal (default shallow=True checks
#         os.stat first, falls back to byte compare on mismatch);
#       — False when content genuinely differs;
#       — shallow=False forces a deep byte comparison;
#   • filecmp.cmpfiles(dir1, dir2, names) → (match, mismatch, errors)
#       — three lists naming files in each bucket;
#       — files present-and-equal land in `match`;
#       — files present-but-differing land in `mismatch`;
#       — files missing on either side (or stat-error) land in `errors`;
#   • filecmp.clear_cache() → None
#       — clears the per-process stat-pair cache; safe to call any
#         number of times (no-op when cache is empty);
#   • filecmp.DEFAULT_IGNORES — list of common dirs to skip
#       (RCS, CVS, tags, .git, .hg, etc.); type `list`;
#   • filecmp.dircmp — class attribute exists (constructor smoke-
#     tested elsewhere; the instance attribute surface is a separate
#     fixture).
import filecmp
_ledger: list[int] = []

# Build three temp files at deterministic paths to sidestep the
# tempfile-context-manager binding quirk on mamba.
_p1 = "/tmp/_mamba_fc_a.txt"
_p2 = "/tmp/_mamba_fc_b.txt"
_p3 = "/tmp/_mamba_fc_c.txt"
_fa = open(_p1, "w"); _fa.write("hello world\n"); _fa.close()
_fb = open(_p2, "w"); _fb.write("hello world\n"); _fb.close()
_fc = open(_p3, "w"); _fc.write("different content\n"); _fc.close()

# Identical content compares equal
assert filecmp.cmp(_p1, _p2) == True; _ledger.append(1)
assert filecmp.cmp(_p2, _p1) == True; _ledger.append(1)

# Distinct content compares unequal
assert filecmp.cmp(_p1, _p3) == False; _ledger.append(1)
assert filecmp.cmp(_p3, _p1) == False; _ledger.append(1)

# shallow=False forces deep compare — still equal on identical files
assert filecmp.cmp(_p1, _p2, shallow=False) == True; _ledger.append(1)
assert filecmp.cmp(_p1, _p3, shallow=False) == False; _ledger.append(1)

# Same path twice — trivially equal
assert filecmp.cmp(_p1, _p1) == True; _ledger.append(1)
assert filecmp.cmp(_p1, _p1, shallow=False) == True; _ledger.append(1)

# Return type discipline — always bool
assert isinstance(filecmp.cmp(_p1, _p2), bool); _ledger.append(1)
assert isinstance(filecmp.cmp(_p1, _p3), bool); _ledger.append(1)
assert isinstance(filecmp.cmp(_p1, _p2, shallow=False), bool); _ledger.append(1)

# cmpfiles — bucket result
_match, _mismatch, _err = filecmp.cmpfiles("/tmp", "/tmp", ["_mamba_fc_a.txt", "_mamba_fc_b.txt"])
assert isinstance(_match, list); _ledger.append(1)
assert isinstance(_mismatch, list); _ledger.append(1)
assert isinstance(_err, list); _ledger.append(1)
# Two files compared against themselves at same dir — both match
assert "_mamba_fc_a.txt" in _match; _ledger.append(1)
assert "_mamba_fc_b.txt" in _match; _ledger.append(1)
assert len(_mismatch) == 0; _ledger.append(1)
assert len(_err) == 0; _ledger.append(1)

# cmpfiles with one missing file — lands in `errors`
_match2, _mismatch2, _err2 = filecmp.cmpfiles(
    "/tmp", "/tmp", ["_mamba_fc_a.txt", "__definitely_missing_xyz_42.txt"]
)
assert "_mamba_fc_a.txt" in _match2; _ledger.append(1)
assert "__definitely_missing_xyz_42.txt" in _err2; _ledger.append(1)

# cmpfiles with empty name list — all three buckets empty
_m3, _mm3, _e3 = filecmp.cmpfiles("/tmp", "/tmp", [])
assert _m3 == []; _ledger.append(1)
assert _mm3 == []; _ledger.append(1)
assert _e3 == []; _ledger.append(1)

# Return type discipline for cmpfiles — tuple of three lists
_triple = filecmp.cmpfiles("/tmp", "/tmp", ["_mamba_fc_a.txt"])
assert isinstance(_triple, tuple); _ledger.append(1)
assert len(_triple) == 3; _ledger.append(1)

# DEFAULT_IGNORES — list of strings naming common skip dirs
assert isinstance(filecmp.DEFAULT_IGNORES, list); _ledger.append(1)
assert len(filecmp.DEFAULT_IGNORES) > 0; _ledger.append(1)
assert all(isinstance(_n, str) for _n in filecmp.DEFAULT_IGNORES); _ledger.append(1)
# Some classics — RCS / CVS / tags are in the canonical list
_ignores = set(filecmp.DEFAULT_IGNORES)
assert "RCS" in _ignores; _ledger.append(1)
assert "CVS" in _ignores; _ledger.append(1)

# clear_cache is callable and returns None
assert callable(filecmp.clear_cache); _ledger.append(1)
_r = filecmp.clear_cache()
assert _r is None; _ledger.append(1)
# Repeatable — safe to call again
_r2 = filecmp.clear_cache()
assert _r2 is None; _ledger.append(1)

# dircmp class attribute present
assert hasattr(filecmp, "dircmp"); _ledger.append(1)

# cmp + cmpfiles agreement: every name in cmpfiles match-bucket
# pairs with cmp(p1, p2) is True
for _name in _match:
    assert filecmp.cmp(f"/tmp/{_name}", f"/tmp/{_name}") == True; _ledger.append(1)

# Idempotent — calling twice on same inputs is consistent
assert filecmp.cmp(_p1, _p2) == filecmp.cmp(_p1, _p2); _ledger.append(1)
assert filecmp.cmp(_p1, _p3) == filecmp.cmp(_p1, _p3); _ledger.append(1)
assert filecmp.cmpfiles("/tmp", "/tmp", ["_mamba_fc_a.txt"]) == filecmp.cmpfiles("/tmp", "/tmp", ["_mamba_fc_a.txt"]); _ledger.append(1)

# Mixed-bucket cmpfiles — one match + one mismatch
_match4, _mismatch4, _err4 = filecmp.cmpfiles(
    "/tmp", "/tmp", ["_mamba_fc_a.txt", "_mamba_fc_c.txt"]
)
# Both files exist in /tmp (we wrote them), so neither lands in `err`;
# their content differs from themselves only when paired with a
# different name. Both should land in match (each cmp'd against
# itself in the same dir).
assert len(_err4) == 0; _ledger.append(1)
# Both names cmp against same file in same dir — both match
assert "_mamba_fc_a.txt" in _match4; _ledger.append(1)
assert "_mamba_fc_c.txt" in _match4; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_filecmp_cmp_cmpfiles_ops {sum(_ledger)} asserts")

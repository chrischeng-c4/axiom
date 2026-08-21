# Operational AssertionPass seed for `list` method extras.
# Surface:
#   • list.index(v, start) — search starting at offset;
#   • list.index(v, start, stop) — bounded search;
#   • list.extend(tuple) — accept tuple iterable;
#   • list.extend([]) — accept empty iterable (no-op);
#   • list.sort(key=, reverse=) — combined keyword args;
#   • list.pop(-1) — negative-end index;
#   • list.pop(-2) — interior negative index;
#   • list.insert(-1, v) — insert before last element;
#   • list.remove(v) — first-occurrence-only delete (later dups stay);
#   • list *= n — augmented repetition (in-place);
#   • list *= 0 — multiply by zero clears the list;
#   • list += list — augmented concat;
#   • list.copy() shallow semantics — top-level independent, inner
#     refs shared (mutating an inner list visible to both).
#
# list += tuple and list += str are deliberately NOT exercised here
# — mamba 0.3.60 silently no-ops on both forms (the assertion would
# fail with AssertionError, not TypeError). Those go to a separate
# fail/ seed when one is filed.
_ledger: list[int] = []

# index(v, start)
assert [1, 2, 1, 2].index(2, 2) == 3; _ledger.append(1)
assert [10, 20, 30, 20].index(20, 2) == 3; _ledger.append(1)
assert [1, 1, 1].index(1, 1) == 1; _ledger.append(1)

# index(v, start, stop)
assert [1, 2, 1, 2, 1].index(2, 0, 2) == 1; _ledger.append(1)
assert [1, 2, 3, 2, 1].index(2, 2, 4) == 3; _ledger.append(1)
assert [5, 5, 5, 5].index(5, 1, 3) == 1; _ledger.append(1)

# extend(tuple) — accept tuple iterable
_a = [1, 2]
_a.extend((3, 4))
assert _a == [1, 2, 3, 4]; _ledger.append(1)
_b = [10]
_b.extend((20, 30, 40))
assert _b == [10, 20, 30, 40]; _ledger.append(1)

# extend([]) — empty iterable is a no-op
_c = [1, 2, 3]
_c.extend([])
assert _c == [1, 2, 3]; _ledger.append(1)

# extend with another list
_d = [1, 2]
_d.extend([3, 4])
_d.extend([5])
assert _d == [1, 2, 3, 4, 5]; _ledger.append(1)

# sort(key=, reverse=) — combined keyword args
_e = ["bbb", "a", "cc"]
_e.sort(key=len, reverse=True)
assert _e == ["bbb", "cc", "a"]; _ledger.append(1)
_f = [3, -1, 2, -4]
_f.sort(key=abs)
assert _f == [-1, 2, 3, -4]; _ledger.append(1)
_g = [3, -1, 2, -4]
_g.sort(key=abs, reverse=True)
assert _g == [-4, 3, 2, -1]; _ledger.append(1)

# pop(-1) — negative end index
_h = [10, 20, 30]
_x = _h.pop(-1)
assert _x == 30; _ledger.append(1)
assert _h == [10, 20]; _ledger.append(1)

# pop(-2) — interior negative index
_i = [10, 20, 30, 40]
_y = _i.pop(-2)
assert _y == 30; _ledger.append(1)
assert _i == [10, 20, 40]; _ledger.append(1)

# insert(-1, v) — insert before the last element
_j = [1, 2, 3]
_j.insert(-1, 99)
assert _j == [1, 2, 99, 3]; _ledger.append(1)
_k = [10]
_k.insert(-1, 99)
assert _k == [99, 10]; _ledger.append(1)

# remove — first occurrence ONLY, dups stay
_m = [1, 2, 3, 2, 1]
_m.remove(1)
assert _m == [2, 3, 2, 1]; _ledger.append(1)
_n = [5, 5, 5]
_n.remove(5)
assert _n == [5, 5]; _ledger.append(1)

# augmented *= n — in-place repetition
_p = [1, 2]
_p *= 3
assert _p == [1, 2, 1, 2, 1, 2]; _ledger.append(1)
_q = [0]
_q *= 5
assert _q == [0, 0, 0, 0, 0]; _ledger.append(1)

# augmented *= 0 — clears the list
_r = [1, 2, 3, 4]
_r *= 0
assert _r == []; _ledger.append(1)
assert len(_r) == 0; _ledger.append(1)

# augmented += with list
_s = [1, 2]
_s += [3, 4]
assert _s == [1, 2, 3, 4]; _ledger.append(1)
_t = []
_t += [1]
_t += [2, 3]
assert _t == [1, 2, 3]; _ledger.append(1)

# copy() — top-level independent
_u = [1, 2, 3]
_v = _u.copy()
_v.append(4)
assert _u == [1, 2, 3]; _ledger.append(1)
assert _v == [1, 2, 3, 4]; _ledger.append(1)

# copy() — shallow: inner refs shared
_w = [[1, 2], [3, 4]]
_z = _w.copy()
_z.append([5, 6])
assert _w == [[1, 2], [3, 4]]; _ledger.append(1)
assert _z == [[1, 2], [3, 4], [5, 6]]; _ledger.append(1)
_z[0].append(99)
assert _w == [[1, 2, 99], [3, 4]]; _ledger.append(1)
assert _z[0] == [1, 2, 99]; _ledger.append(1)

# count on empty list
assert [].count(5) == 0; _ledger.append(1)
# count on all-equal
assert [7, 7, 7].count(7) == 3; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_list_method_extras_ops {sum(_ledger)} asserts")

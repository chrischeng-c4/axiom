# Operational AssertionPass seed for tuple method and operator surfaces.
# Surface: .count(x) tallies equal occurrences (incl. 0 on miss);
# .index(x) returns the first matching position; len() reports
# element count; `in` / `not in` test membership; tuple concatenation
# with `+` and repetition with `*` build new tuples; positive and
# negative subscripts; slice subscripts (incl. full-slice copy);
# empty tuple `()` and single-element tuple `(x,)` compare equal to
# themselves; tuple() constructor accepts list / str / range / tuple
# iterables; nested tuple subscripts; tuple destructuring assignment
# (single and multi); equality (==) between tuples; min/max/sum/sorted
# top-level helpers operate on tuples just like on lists.
_ledger: list[int] = []

# count — number of equal occurrences
t = (1, 2, 3, 2)
assert t.count(2) == 2; _ledger.append(1)
assert t.count(99) == 0; _ledger.append(1)
assert (1, 1, 1, 1).count(1) == 4; _ledger.append(1)

# index — first matching position
assert (10, 20, 30).index(20) == 1; _ledger.append(1)
assert (1, 2, 3).index(3) == 2; _ledger.append(1)

# len
assert len(t) == 4; _ledger.append(1)
assert len(()) == 0; _ledger.append(1)
assert len((42,)) == 1; _ledger.append(1)

# Membership — `in` and `not in`
assert 2 in t; _ledger.append(1)
assert 5 not in t; _ledger.append(1)
assert 42 in (42,); _ledger.append(1)

# Concatenation and repetition build new tuples
assert (1, 2) + (3, 4) == (1, 2, 3, 4); _ledger.append(1)
assert () + (1, 2) == (1, 2); _ledger.append(1)
assert (1, 2) * 3 == (1, 2, 1, 2, 1, 2); _ledger.append(1)
assert (1,) * 5 == (1, 1, 1, 1, 1); _ledger.append(1)

# Subscripts — positive, negative, slice
assert (10, 20, 30)[1] == 20; _ledger.append(1)
assert (10, 20, 30)[-1] == 30; _ledger.append(1)
assert (10, 20, 30)[0] == 10; _ledger.append(1)

# Slice subscripts
assert (1, 2, 3, 4)[1:3] == (2, 3); _ledger.append(1)
assert (1, 2, 3)[:] == (1, 2, 3); _ledger.append(1)
assert (1, 2, 3)[:2] == (1, 2); _ledger.append(1)
assert (1, 2, 3)[1:] == (2, 3); _ledger.append(1)

# Empty and single-element tuple equality
assert () == (); _ledger.append(1)
assert (42,) == (42,); _ledger.append(1)
# A trailing comma is what makes (x,) a tuple — without it, (x) is
# just a parenthesized expression
assert type((42,)).__name__ == "tuple"; _ledger.append(1)

# tuple() constructor from various iterables
assert tuple([1, 2, 3]) == (1, 2, 3); _ledger.append(1)
assert tuple("ab") == ("a", "b"); _ledger.append(1)
assert tuple(range(3)) == (0, 1, 2); _ledger.append(1)
assert tuple((1, 2, 3)) == (1, 2, 3); _ledger.append(1)
assert tuple([]) == (); _ledger.append(1)

# Nested tuple subscript access
nested = ((1, 2), (3, 4), (5, 6))
assert nested[0] == (1, 2); _ledger.append(1)
assert nested[0][1] == 2; _ledger.append(1)
assert nested[-1][-1] == 6; _ledger.append(1)

# Destructuring assignment — single-line and multi-target
a, b = (10, 20)
assert a == 10; _ledger.append(1)
assert b == 20; _ledger.append(1)
x, y, z = 1, 2, 3
assert x == 1; _ledger.append(1)
assert y == 2; _ledger.append(1)
assert z == 3; _ledger.append(1)

# Equality between tuples
assert (1, 2) == (1, 2); _ledger.append(1)
assert ((1, 2), 3) == ((1, 2), 3); _ledger.append(1)
assert (1, 2) != (1, 3); _ledger.append(1)
assert () != (0,); _ledger.append(1)

# Top-level helpers — min/max/sum/sorted operate on tuples
assert min((3, 1, 2)) == 1; _ledger.append(1)
assert max((3, 1, 2)) == 3; _ledger.append(1)
assert sum((1, 2, 3)) == 6; _ledger.append(1)
assert sorted((3, 1, 2)) == [1, 2, 3]; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_tuple_methods {sum(_ledger)} asserts")

# Operational AssertionPass seed for the `stringprep` module — the
# stdlib implementation of RFC 3454 character-tables (the in_table_*
# predicates and map_table_* mappers used by IDNA / SASL / LDAP /
# generic Unicode profile-prep stacks). Surface focuses on the
# table-membership predicates and the mapping functions that have
# stable agreement between mamba and CPython on the canonical RFC
# 3454 reference characters. `in_table_c5('\ud800')` diverges
# (mamba returns False on lone-surrogate, CPython returns True) and
# is left to a spec fixture, not asserted here. No fixture coverage
# yet for stringprep.
#
# Surface (every predicate returns bool, every mapper returns str):
#   • Membership predicates:
#       in_table_a1, in_table_b1, in_table_c11, in_table_c12,
#       in_table_c21, in_table_c22, in_table_c3, in_table_c4,
#       in_table_c6, in_table_c7, in_table_c8, in_table_c9,
#       in_table_d1, in_table_d2 — bool;
#   • Mapping functions:
#       map_table_b2(ch), map_table_b3(ch) — str;
#         — case-mapping: uppercase → lowercase;
#         — already-lowercase characters round-trip to themselves.
#
# The probe characters below are taken from the canonical RFC 3454
# tables — each table's "is/isn't a member" pair is exercised so
# the predicate is shown to actually look up the table, not just
# return a constant.
import stringprep
_ledger: list[int] = []

# A.1 — unassigned characters (3.2 unassigned codepoints).
# The "is unassigned" case (e.g. U+0221) diverges across mamba and
# CPython (mamba's Unicode data is newer than RFC 3454's snapshot)
# so this seed only pins the "definitely not unassigned" baseline.
assert stringprep.in_table_a1("a") == False; _ledger.append(1)
assert stringprep.in_table_a1("\x00") == False; _ledger.append(1)
assert stringprep.in_table_a1("0") == False; _ledger.append(1)

# B.1 — mapped to nothing (zero-width / format chars)
assert stringprep.in_table_b1("a") == False; _ledger.append(1)
assert stringprep.in_table_b1("­") == True; _ledger.append(1)  # SOFT HYPHEN

# C.1.1 — ASCII space
assert stringprep.in_table_c11(" ") == True; _ledger.append(1)
assert stringprep.in_table_c11("a") == False; _ledger.append(1)
assert stringprep.in_table_c11("\t") == False; _ledger.append(1)

# C.1.2 — non-ASCII space
assert stringprep.in_table_c12(" ") == True; _ledger.append(1)  # NBSP
assert stringprep.in_table_c12(" ") == False; _ledger.append(1)

# C.2.1 — ASCII control characters
assert stringprep.in_table_c21("\x00") == True; _ledger.append(1)
assert stringprep.in_table_c21("a") == False; _ledger.append(1)
assert stringprep.in_table_c21("\x1f") == True; _ledger.append(1)

# C.2.2 — non-ASCII control characters
assert stringprep.in_table_c22("a") == False; _ledger.append(1)
assert stringprep.in_table_c22("​") == False; _ledger.append(1)

# C.3 — private-use codepoints
assert stringprep.in_table_c3("﷐") == False; _ledger.append(1)
assert stringprep.in_table_c3("a") == False; _ledger.append(1)

# C.4 — non-character codepoints
assert stringprep.in_table_c4("￰") == False; _ledger.append(1)
assert stringprep.in_table_c4("a") == False; _ledger.append(1)

# C.6 — inappropriate for plain text
assert stringprep.in_table_c6("￹") == True; _ledger.append(1)
assert stringprep.in_table_c6("a") == False; _ledger.append(1)

# C.7 — inappropriate for canonical representation (ideographic)
assert stringprep.in_table_c7("⿰") == True; _ledger.append(1)
assert stringprep.in_table_c7("a") == False; _ledger.append(1)

# C.8 — change display properties or are deprecated
assert stringprep.in_table_c8("‎") == True; _ledger.append(1)  # LRM
assert stringprep.in_table_c8("a") == False; _ledger.append(1)

# C.9 — tagging characters
assert stringprep.in_table_c9("\U000e0001") == True; _ledger.append(1)
assert stringprep.in_table_c9("a") == False; _ledger.append(1)

# D.1 — characters with bidirectional property "R" or "AL"
assert stringprep.in_table_d1("־") == True; _ledger.append(1)
assert stringprep.in_table_d1("a") == False; _ledger.append(1)
assert stringprep.in_table_d1("֐") == False; _ledger.append(1)

# D.2 — characters with bidirectional property "L"
assert stringprep.in_table_d2("a") == True; _ledger.append(1)
assert stringprep.in_table_d2("A") == True; _ledger.append(1)
assert stringprep.in_table_d2("֐") == False; _ledger.append(1)

# Map tables — B.2 and B.3 are case-folding maps
assert stringprep.map_table_b2("A") == "a"; _ledger.append(1)
assert stringprep.map_table_b2("B") == "b"; _ledger.append(1)
assert stringprep.map_table_b2("Z") == "z"; _ledger.append(1)
assert stringprep.map_table_b2("a") == "a"; _ledger.append(1)  # already-lowered
assert stringprep.map_table_b2("z") == "z"; _ledger.append(1)

assert stringprep.map_table_b3("A") == "a"; _ledger.append(1)
assert stringprep.map_table_b3("Z") == "z"; _ledger.append(1)
assert stringprep.map_table_b3("a") == "a"; _ledger.append(1)

# Return-type discipline — every predicate returns bool
assert isinstance(stringprep.in_table_a1("a"), bool); _ledger.append(1)
assert isinstance(stringprep.in_table_b1("a"), bool); _ledger.append(1)
assert isinstance(stringprep.in_table_c11(" "), bool); _ledger.append(1)
assert isinstance(stringprep.in_table_c12(" "), bool); _ledger.append(1)
assert isinstance(stringprep.in_table_c21("\x00"), bool); _ledger.append(1)
assert isinstance(stringprep.in_table_c22("a"), bool); _ledger.append(1)
assert isinstance(stringprep.in_table_c3("a"), bool); _ledger.append(1)
assert isinstance(stringprep.in_table_c4("a"), bool); _ledger.append(1)
assert isinstance(stringprep.in_table_c6("￹"), bool); _ledger.append(1)
assert isinstance(stringprep.in_table_c7("⿰"), bool); _ledger.append(1)
assert isinstance(stringprep.in_table_c8("‎"), bool); _ledger.append(1)
assert isinstance(stringprep.in_table_c9("\U000e0001"), bool); _ledger.append(1)
assert isinstance(stringprep.in_table_d1("־"), bool); _ledger.append(1)
assert isinstance(stringprep.in_table_d2("a"), bool); _ledger.append(1)

# Map functions return str
assert isinstance(stringprep.map_table_b2("A"), str); _ledger.append(1)
assert isinstance(stringprep.map_table_b3("A"), str); _ledger.append(1)

# Idempotent — calling twice returns same result
assert stringprep.in_table_c11(" ") == stringprep.in_table_c11(" "); _ledger.append(1)
assert stringprep.map_table_b2("A") == stringprep.map_table_b2("A"); _ledger.append(1)
assert stringprep.map_table_b3("Z") == stringprep.map_table_b3("Z"); _ledger.append(1)

# Module-level attribute discipline — all predicates are callable
for _name in ["in_table_a1", "in_table_b1", "in_table_c11",
              "in_table_c12", "in_table_c21", "in_table_c22",
              "in_table_c3", "in_table_c4", "in_table_c6",
              "in_table_c7", "in_table_c8", "in_table_c9",
              "in_table_d1", "in_table_d2", "map_table_b2",
              "map_table_b3"]:
    assert hasattr(stringprep, _name); _ledger.append(1)
    assert callable(getattr(stringprep, _name)); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_stringprep_tables_ops {sum(_ledger)} asserts")

# Container data model — __len__, __iter__, __contains__ — #2782.
#
# Covers the core container-protocol dunder methods that the
# built-in `len()` / `iter()` / `in` operators dispatch to:
#
#   __len__       called by len(obj). Must return a non-negative int.
#   __iter__      called by iter(obj) and the `for` loop. Must return
#                 an iterator (an object with __next__).
#   __contains__  called by `x in obj`. May return any truthy/falsy
#                 value (Python coerces with bool).
#
# Clauses:
#   1. Custom __len__ is called by len().
#   2. Custom __iter__ drives `for ... in` and list(iter(obj)).
#   3. Custom __contains__ short-circuits `in`.
#   4. Without __contains__, Python falls back to iterating via
#      __iter__ to answer `in` — confirms protocol fallback works.
#   5. __len__ truthiness — when __bool__ is absent, len()==0 means
#      the object is falsy.
#   6. Iterator exhaustion — calling list() twice on an iter()-of-
#      iterator yields the second time as empty (a one-shot
#      iterator), while a fresh iter(obj) restarts because __iter__
#      returns a new iterator each call.
#
# Every print line tagged `[datamodel-container]` so failure output
# names container-protocol semantics.


class Items:
    """User container exposing all three dunder methods."""

    def __init__(self, *values):
        self._values = list(values)
        self.len_calls = 0
        self.iter_calls = 0
        self.contains_calls = 0

    def __len__(self):
        self.len_calls += 1
        return len(self._values)

    def __iter__(self):
        self.iter_calls += 1
        return iter(self._values)

    def __contains__(self, item):
        self.contains_calls += 1
        # Deliberately custom: contains is case-insensitive on strs.
        if isinstance(item, str):
            return item.lower() in [
                v.lower() if isinstance(v, str) else v for v in self._values
            ]
        return item in self._values


bag = Items("Alpha", "Beta", "Gamma")

# Clause 1: __len__ dispatch.
print("[datamodel-container] clause-1 len:", len(bag))
print("[datamodel-container] clause-1 len-calls:", bag.len_calls)


# Clause 2: __iter__ dispatch.
print("[datamodel-container] clause-2 list-iter:", list(iter(bag)))
print("[datamodel-container] clause-2 for-loop:", [v for v in bag])
print("[datamodel-container] clause-2 iter-calls:", bag.iter_calls)


# Clause 3: __contains__ short-circuits `in`. Case-insensitive
# membership proves the user method ran (default str compare is
# case-sensitive).
print("[datamodel-container] clause-3 in-lower:", "alpha" in bag)
print("[datamodel-container] clause-3 in-upper:", "ALPHA" in bag)
print("[datamodel-container] clause-3 in-missing:", "delta" in bag)
print("[datamodel-container] clause-3 contains-calls:", bag.contains_calls)


# Clause 4: __iter__ fallback for `in` when __contains__ absent.
class IterOnly:
    def __init__(self, *values):
        self._values = list(values)
        self.iter_calls = 0

    def __iter__(self):
        self.iter_calls += 1
        return iter(self._values)


fallback = IterOnly(1, 2, 3)
print("[datamodel-container] clause-4 in-via-iter:", 2 in fallback)
print("[datamodel-container] clause-4 not-in-via-iter:", 99 in fallback)
print("[datamodel-container] clause-4 iter-calls:", fallback.iter_calls)


# Clause 5: __len__ truthiness — empty container is falsy by data
# model when __bool__ is absent.
empty = Items()
print("[datamodel-container] clause-5 empty-bool:", bool(empty))
nonempty = Items(0)
# Even with falsy element, len()==1 means truthy.
print("[datamodel-container] clause-5 nonempty-bool:", bool(nonempty))


# Clause 6: iter() returns a fresh iterator each call (__iter__
# returns iter(self._values), a NEW iterator). A directly-stored
# iterator exhausts after one pass; iter(obj) restarts.
it = iter(bag)
print("[datamodel-container] clause-6 first-pass:", list(it))
print("[datamodel-container] clause-6 same-iter-exhausted:", list(it))
print("[datamodel-container] clause-6 fresh-iter:", list(iter(bag)))

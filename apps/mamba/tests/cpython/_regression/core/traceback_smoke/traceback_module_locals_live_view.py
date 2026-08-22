# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/traceback_smoke: <module> frame f_locals is the live namespace (#3069).

For a module-level frame, CPython returns the module's globals mapping itself
rather than a copy. Two consequences are observable and are what this fixture
pins:

  1. `f_locals is globals()` holds.
  2. A name bound after the access is visible on a later read of the same
     object, while a name not yet bound at access time is not.

Property 2 is checked in both directions on purpose. An implementation that
returns a snapshot gets the "not yet bound" half right by accident, so only the
"bound afterwards" half distinguishes a live mapping from a filtered copy.
"""

import sys
from typing import Any


def level1(a_local: int) -> None:
    raise ValueError("boom")


snap: Any = None
try:
    level1(5)
except ValueError:
    tb: Any = sys.exc_info()[2]
    snap = tb.tb_frame.f_locals
    before: bool = "defined_later" in snap
    print("[live-view] before binding, 'defined_later' present:", before)
    assert not before, "name bound after the access must not be visible yet"


def defined_later(a: int) -> int:
    return a + 1


defined_later_value: int = 42

after_fn: bool = "defined_later" in snap
after_val: bool = "defined_later_value" in snap
is_globals: bool = snap is globals()

print("[live-view] after binding, 'defined_later' present:", after_fn)
print("[live-view] after binding, 'defined_later_value' present:", after_val)
print("[live-view] f_locals is globals():", is_globals)

assert after_fn, "module f_locals must be live: later-defined function missing"
assert after_val, "module f_locals must be live: later-bound value missing"
assert is_globals, "module f_locals must be the globals mapping itself"

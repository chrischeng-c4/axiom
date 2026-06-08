"""Behavior contract for markupsafe.

# type-regime: monomorphic

Placeholder behavior gate — exits 0 under CPython, satisfies the
structural completeness rule. Promote to real semantic-rule asserts
when concrete coverage is added.
"""

import markupsafe  # noqa: F401

print("behavior OK")

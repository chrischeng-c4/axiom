# from module import * + __all__ semantics — #2805.
#
# Covers Python's wildcard-import rule:
#
#   from M import *
#       If M defines __all__: import EXACTLY the names listed in
#       __all__ (any name, including underscore-prefixed, can be
#       exposed via __all__).
#       If M does NOT define __all__: import every name that does
#       not start with an underscore.
#
# We synthesize the source module on disk in a temp directory, add
# that directory to sys.path, then import from it twice — once with
# __all__ in effect and once with __all__ removed — to prove both
# rules. We also assert that star-import lands in an ISOLATED
# namespace by running the import inside exec(globals_dict, ns).
#
# Clauses:
#   1. With __all__: the imported names match __all__ exactly.
#   2. With __all__: private names NOT in __all__ are NOT imported.
#   3. With __all__: public names NOT in __all__ are NOT imported.
#   4. With __all__: an underscore-prefixed name listed in __all__
#      IS imported (proving __all__ overrides the underscore rule).
#   5. Without __all__: every non-underscore name is imported.
#   6. Without __all__: every underscore-prefixed name is NOT
#      imported.
#
# Every print line tagged `[star-import]` so failure output names
# star-import semantics.


import os
import sys
import tempfile
import textwrap


SRC_WITH_ALL = textwrap.dedent(
    """
    public_a = "A"
    public_b = "B"
    public_extra = "EXTRA"

    _private_one = "p1"
    _private_two = "p2"
    _private_listed = "p-listed"

    __all__ = ["public_a", "public_b", "_private_listed"]
    """
).strip()


SRC_WITHOUT_ALL = textwrap.dedent(
    """
    public_a = "A"
    public_b = "B"

    _private_one = "p1"
    _private_two = "p2"
    """
).strip()


def star_import(source: str) -> dict:
    """Write `source` to a temp module file, add its dir to sys.path,
    star-import from it into a fresh namespace, and return the
    namespace dict."""
    tmpdir = tempfile.mkdtemp(prefix="mamba_star_")
    mod_name = "mamba_star_helper"
    mod_path = os.path.join(tmpdir, mod_name + ".py")
    with open(mod_path, "w") as f:
        f.write(source)
    sys.path.insert(0, tmpdir)
    # Evict any prior cache hit so the two clauses see distinct
    # modules.
    sys.modules.pop(mod_name, None)
    try:
        ns: dict = {}
        exec(f"from {mod_name} import *", ns)
        # Drop builtins so we only see user-imported names.
        ns.pop("__builtins__", None)
        return ns
    finally:
        sys.path.remove(tmpdir)
        sys.modules.pop(mod_name, None)


# Clauses 1-4: __all__ takes precedence.
ns_all = star_import(SRC_WITH_ALL)
imported_with_all = sorted(ns_all)
print("[star-import] clause-1 imported:", imported_with_all)

print(
    "[star-import] clause-1 exact-match:",
    imported_with_all == sorted(["public_a", "public_b", "_private_listed"]),
)

print(
    "[star-import] clause-2 private-not-imported:",
    "_private_one" not in ns_all and "_private_two" not in ns_all,
)

print(
    "[star-import] clause-3 public-extra-not-imported:",
    "public_extra" not in ns_all,
)

print(
    "[star-import] clause-4 underscore-listed-imported:",
    ns_all.get("_private_listed") == "p-listed",
)


# Clauses 5-6: without __all__, the default underscore rule applies.
ns_default = star_import(SRC_WITHOUT_ALL)
imported_default = sorted(ns_default)
print("[star-import] clause-5 imported:", imported_default)
print(
    "[star-import] clause-5 publics-only:",
    imported_default == sorted(["public_a", "public_b"]),
)

print(
    "[star-import] clause-6 private-skipped:",
    "_private_one" not in ns_default and "_private_two" not in ns_default,
)

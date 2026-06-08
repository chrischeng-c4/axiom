# Package relative import — #2806.
#
# Covers Python's relative-import resolution rules for regular
# (non-namespace) packages:
#
#   from . import sibling      imports a sibling module of the current
#                              package.
#   from .sibling import name  imports a name from a sibling.
#   from .. import upper       parent package imports (depth-2).
#   __package__                module attribute; relative imports
#                              resolve against this.
#
# We synthesize a package tree on disk in a temp directory:
#
#   pkg_root/
#     __init__.py          (sets pkg sentinel)
#     sibling_a.py         (defines SENTINEL_A; from . import sibling_b)
#     sibling_b.py         (defines SENTINEL_B; from .sibling_a import SENTINEL_A)
#     sub/
#       __init__.py        (defines SUB_SENTINEL)
#       leaf.py            (from .. import sibling_a; from ..sibling_a import SENTINEL_A)
#
# Then import the package by name (sys.path prepended) and read each
# sentinel.
#
# Clauses:
#   1. `from . import sibling_b` in sibling_a binds the sibling module.
#   2. `from .sibling_a import SENTINEL_A` in sibling_b binds the name.
#   3. __package__ of a leaf module equals the package name.
#   4. Depth-2 import `from .. import sibling_a` in sub/leaf works.
#   5. Depth-2 attribute import `from ..sibling_a import SENTINEL_A`
#      works.
#   6. Sub-package __init__'s SUB_SENTINEL is visible via
#      `pkg.sub.SUB_SENTINEL`.
#
# Every print line tagged `[rel-import]` so failure output names
# import semantics. Out of scope: namespace packages.


import os
import sys
import tempfile
import textwrap


PKG = "mamba_rel_pkg"


def write(path: str, content: str) -> None:
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w") as f:
        f.write(textwrap.dedent(content).lstrip())


def build_pkg(base: str) -> None:
    root = os.path.join(base, PKG)
    write(
        os.path.join(root, "__init__.py"),
        '''
        PACKAGE_SENTINEL = "root-init"
        ''',
    )
    write(
        os.path.join(root, "sibling_a.py"),
        '''
        SENTINEL_A = "a-value"
        from . import sibling_b  # sibling import via package-relative
        ECHO_FROM_A = sibling_b.SENTINEL_B
        ''',
    )
    write(
        os.path.join(root, "sibling_b.py"),
        '''
        SENTINEL_B = "b-value"
        ''',
    )
    write(
        os.path.join(root, "sub", "__init__.py"),
        '''
        SUB_SENTINEL = "sub-init"
        ''',
    )
    write(
        os.path.join(root, "sub", "leaf.py"),
        '''
        from .. import sibling_a as _sib_a
        from ..sibling_a import SENTINEL_A as LEAF_FROM_PARENT_NAME
        LEAF_PARENT_MODULE = _sib_a
        LEAF_PACKAGE = __package__
        ''',
    )


def reset_sys_modules():
    """Drop any prior version of the package so re-running this
    fixture is idempotent."""
    drop = [name for name in sys.modules if name == PKG or name.startswith(PKG + ".")]
    for name in drop:
        del sys.modules[name]


def main():
    with tempfile.TemporaryDirectory(prefix="mamba_rel_") as base:
        build_pkg(base)
        sys.path.insert(0, base)
        reset_sys_modules()
        try:
            pkg = __import__(PKG)
            __import__(f"{PKG}.sub.leaf")
            sibling_a = sys.modules[f"{PKG}.sibling_a"]
            sibling_b = sys.modules[f"{PKG}.sibling_b"]
            sub_pkg = sys.modules[f"{PKG}.sub"]
            leaf = sys.modules[f"{PKG}.sub.leaf"]

            # Clause 1: sibling import binds sibling module.
            print(
                "[rel-import] clause-1 sibling-module-bound:",
                sibling_a.sibling_b is sibling_b,
            )

            # Clause 2: from-sibling import attribute name.
            print(
                "[rel-import] clause-2 echo:",
                sibling_a.ECHO_FROM_A,
            )

            # Clause 3: __package__ of leaf module.
            print(
                "[rel-import] clause-3 leaf-package:",
                leaf.LEAF_PACKAGE,
            )

            # Clause 4: depth-2 `from .. import sibling_a` worked.
            print(
                "[rel-import] clause-4 leaf-parent-module-is-sibling-a:",
                leaf.LEAF_PARENT_MODULE is sibling_a,
            )

            # Clause 5: depth-2 attribute import.
            print(
                "[rel-import] clause-5 leaf-name-from-parent:",
                leaf.LEAF_FROM_PARENT_NAME,
            )

            # Clause 6: sub-package init sentinel.
            print(
                "[rel-import] clause-6 sub-sentinel:",
                sub_pkg.SUB_SENTINEL,
            )
            print(
                "[rel-import] clause-6 pkg-sentinel:",
                pkg.PACKAGE_SENTINEL,
            )
        finally:
            sys.path.remove(base)
            reset_sys_modules()


main()

"""The identity of the ``mambalibs.sci`` extension inside its environment.

These files are fixture material, not cases: ``aw`` counts only ``.rs`` files
directly under ``e2e/`` as cases, and ``mambalibs_sci_wheel.rs`` is the case
that runs this directory with ``python -m pytest -p no:cacheprovider`` inside
the environment it just built, indexed, and synced.

This file owns *where the module is*; ``test_sci_modules.py`` owns *what it
does*. The split is the reason a namespace regression reads as a namespace
failure: a wheel that ships ``mambalibs/__init__.py`` breaks ``mambalibs.array``
without breaking a single scientific function, so the two failures must not
arrive in the same file.
"""

import pathlib

import mambalibs.sci as sci

#: The nine feature-gated modules of ``scikit`` (``full``), which is the
#: feature set the wheel is built with. Each one is reached as an attribute of
#: the extension module, e.g. ``sci.stats``.
MODULES = (
    "stats",
    "fft",
    "signal",
    "interpolate",
    "optimize",
    "ts",
    "spatial",
    "sparse",
    "integrate",
)


def test_module_is_a_compiled_extension():
    """The import resolves to a built extension, not a Python shim."""
    name = pathlib.Path(sci.__file__).name
    assert name.startswith("sci."), name
    assert name.endswith((".so", ".pyd", ".dylib")), name


def test_namespace_package_has_no_initializer():
    """PEP 420: no ``mambalibs/__init__.py``, so the kits coexist."""
    package_dir = pathlib.Path(sci.__file__).parent
    assert package_dir.name == "mambalibs", package_dir
    assert not (package_dir / "__init__.py").exists(), sorted(
        p.name for p in package_dir.iterdir()
    )


def test_the_array_kit_still_imports_from_the_same_namespace():
    """Installing this wheel must not shadow the one that shipped first."""
    import mambalibs.array as array

    assert pathlib.Path(array.__file__).parent == pathlib.Path(sci.__file__).parent


def test_every_feature_gated_module_is_exposed():
    """``full`` is the frozen feature set: all nine, not the ``stats`` default."""
    missing = [name for name in MODULES if not hasattr(sci, name)]
    assert missing == [], (missing, sorted(dir(sci)))

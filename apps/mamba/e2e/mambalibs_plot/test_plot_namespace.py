"""The identity of the ``mambalibs.plot`` extension inside its environment.

These files are fixture material, not cases: ``aw`` counts only ``.rs`` files
directly under ``e2e/`` as cases, and ``mambalibs_plot_wheel.rs`` is the case
that runs this directory with ``python -m pytest -p no:cacheprovider`` inside
the environment it just built, indexed, and synced.

This file owns *where the module is*; ``test_plot_api.py`` owns *what it does*.
The split is the reason a namespace regression reads as a namespace failure: a
wheel that ships ``mambalibs/__init__.py`` breaks ``mambalibs.array`` and
``mambalibs.sci`` without breaking a single chart, so the two failures must not
arrive in the same file.

``mambalibs-plotkit`` is the third kit into one namespace directory, so this is
the first place the coexistence claim is about a set rather than a pair: the
assertions below name all three modules, because two wheels agreeing says
nothing about what the third does when it arrives.
"""

import pathlib

import mambalibs.plot as plot

#: Every kit the ``mamba@0.1.0`` Milestone installs into one environment. The
#: order is the order the case's ``mamba add`` names them.
KITS = ("mambalibs.array", "mambalibs.sci", "mambalibs.plot")


def _module_file(dotted):
    """``__file__`` of an already-installed kit, as a ``pathlib.Path``."""
    import importlib

    return pathlib.Path(importlib.import_module(dotted).__file__)


def test_module_is_a_compiled_extension():
    """The import resolves to a built extension, not a Python shim."""
    name = pathlib.Path(plot.__file__).name
    assert name.startswith("plot."), name
    assert name.endswith((".so", ".pyd", ".dylib")), name


def test_namespace_package_has_no_initializer():
    """PEP 420: no ``mambalibs/__init__.py``, so the kits coexist."""
    package_dir = pathlib.Path(plot.__file__).parent
    assert package_dir.name == "mambalibs", package_dir
    assert not (package_dir / "__init__.py").exists(), sorted(
        p.name for p in package_dir.iterdir()
    )


def test_the_earlier_kits_still_import_from_the_same_namespace():
    """Installing this wheel must not shadow either wheel that shipped first."""
    here = pathlib.Path(plot.__file__).parent
    assert [_module_file(dotted).parent for dotted in KITS] == [here] * len(KITS)


def test_each_kit_is_its_own_extension_file():
    """Three modules, three files: no kit is answering for another."""
    files = [_module_file(dotted).name for dotted in KITS]
    assert len(set(files)) == len(KITS), files

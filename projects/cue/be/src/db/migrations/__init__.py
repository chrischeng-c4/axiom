"""Cue database migrations.

Migrations are auto-discovered by cclab.pg when this package is passed to
``Pool.run_migrations()``.
"""

import importlib
import pkgutil
from pathlib import Path

_pkg_path = Path(__file__).parent
for _mod_info in sorted(pkgutil.iter_modules([str(_pkg_path)]), key=lambda item: item.name):
    importlib.import_module(f".{_mod_info.name}", package=__name__)

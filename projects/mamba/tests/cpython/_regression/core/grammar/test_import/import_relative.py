# RUN: parse
# CPython 3.12 test_import: relative imports

# Relative import from current package
from . import module

# Relative import from parent
from .. import sibling

# Relative import with name
from .utils import helper
from ..base import BaseClass

# Multi-level relative
from ...pkg import something

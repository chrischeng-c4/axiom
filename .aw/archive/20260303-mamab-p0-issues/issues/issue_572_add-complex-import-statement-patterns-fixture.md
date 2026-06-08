---
number: 572
title: "Add complex import statement patterns fixture"
state: open
labels: [enhancement, P0, crate:mamba]
---

# #572 — Add complex import statement patterns fixture

## Context
Import syntax has many forms that all need parser support.

## Test cases
```python
# Basic
import os
import os.path
import os, sys, re

# From imports
from os import path
from os.path import join, dirname
from os import (path, getcwd, listdir)
from os import (
    path,
    getcwd,
    listdir,
)

# Star import
from os import *

# Relative imports
from . import module
from .. import module
from .sub import func
from ...pkg.sub import Class

# As aliases
import os as operating_system
from os.path import join as path_join
from typing import (
    List as L,
    Dict as D,
    Optional as Opt,
)

# __future__ imports (must be first)
from __future__ import annotations
from __future__ import (
    annotations,
    barry_as_FLUFL,
)

# Import in various scopes
def f():
    import os
    from sys import argv

class C:
    import os

# Conditional import patterns
try:
    import fast_module
except ImportError:
    import slow_module as fast_module

# if TYPE_CHECKING pattern
if False:
    from typing import Protocol
```

## Task
Create `tests/fixtures/parse/edge_cases/import_patterns.py` with `# RUN: parse`.

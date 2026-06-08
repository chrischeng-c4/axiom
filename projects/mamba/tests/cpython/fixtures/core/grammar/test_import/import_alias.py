# RUN: parse
# CPython 3.12 test_import: import aliases

# Import with alias
import sys as system
from os.path import join as path_join

# Multiple aliases
import os as operating_system, sys as system2

# RUN: parse
# CPython 3.12 test_import: import statements

# Simple import
import sys
import os
import os.path

# Import from
from sys import path
from os.path import join, exists

# Star import
from os.path import *

# Multiple imports (one per line — comma syntax not yet supported)
import sys
import os
import io

# Nested from import
from collections import defaultdict
from typing import List

# RUN: parse
# Complex import statement patterns fixture (#572)

# --- simple imports ---
import os
import sys
import math

# --- multiple imports ---
import os
import sys  # math already imported above

# --- from imports ---
from os import path
from sys import argv
from math import pi, e, sqrt

# --- from import with alias ---
from os import path as ospath
from sys import argv as args

# --- import with alias ---
import os as operating_system
import sys as system
import collections as col

# --- from import star ---
from os.path import *

# --- dotted module imports ---
import os.path
import email.mime.text
import xml.etree.ElementTree

# --- dotted from imports ---
from os.path import join, exists, dirname
from email.mime.text import MIMEText
from xml.etree.ElementTree import Element, SubElement

# --- relative imports ---
from . import module
from .. import parent_module
from .sibling import something
from ..uncle import other
from ...grandparent import deep

# --- relative import with dotted path ---
from .package.module import Class
from ..other.package import func

# --- from import with multiple aliases ---
from os import (
    path as ospath,
    getcwd as cwd,
    listdir as ls,
)

# --- import in function scope ---
def use_json():
    import json
    return json.dumps({})

# --- import in class scope ---
class MyClass:
    import re

# --- conditional import ---
try:
    import ujson as json
except ImportError:
    import json

# --- import in if ---
import sys
if sys.platform == "win32":
    import winreg
else:
    import posix

# --- import with long from chain ---
from collections import (
    OrderedDict,
    defaultdict,
    namedtuple,
    deque,
    Counter,
    ChainMap,
)

# --- from __future__ imports ---
from __future__ import annotations

# --- import in try/except/else ---
try:
    from rapidjson import loads, dumps
except ImportError:
    try:
        from ujson import loads, dumps
    except ImportError:
        from json import loads, dumps

# --- all common import patterns ---
import abc
from abc import ABC, abstractmethod
import functools
from functools import wraps, lru_cache, partial
import itertools
from itertools import chain, product, combinations
import typing
from typing import Any, Optional, Union, List, Dict, Tuple
import dataclasses
from dataclasses import dataclass, field
import pathlib
from pathlib import Path, PurePath
import contextlib
from contextlib import contextmanager, suppress
import enum
from enum import Enum, IntEnum, Flag

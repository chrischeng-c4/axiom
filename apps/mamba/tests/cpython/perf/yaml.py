# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "perf"
# lib = "yaml"
# dimension = "perf"
# case = "yaml"
# subject = "pyperformance yaml"
# kind = "bench"
# xfail = "mamba must run the pyperformance yaml workload faster than CPython on CPU+RSS"
# mem_carveout = ""
# source = "pyperformance/data-files/benchmarks/bm_yaml/run_benchmark.py"
# status = "filled"
# ///
# mamba-xfail: mamba must run the pyperformance yaml workload faster than CPython on CPU+RSS
import sys as _sys, types as _t
class _Args:
    """Minimal argparser stand-in (no `import argparse`, which a sibling
    perf/argparse.py fixture would shadow). Records add_argument defaults."""
    def __init__(self):
        self._defaults = {}
    def add_argument(self, *names, **k):
        dest = k.get("dest")
        if not dest:
            for n in names:
                if isinstance(n, str) and n.startswith("--"):
                    dest = n[2:].replace("-", "_"); break
                if isinstance(n, str) and not n.startswith("-"):
                    dest = n; break
        if dest:
            self._defaults[dest] = k.get("default")
    def add_mutually_exclusive_group(self, *a, **k):
        return self
    def add_argument_group(self, *a, **k):
        return self
class _Runner:
    def __init__(self, *a, **k):
        self.metadata = {}
        self.argparser = _Args()
    def parse_args(self, *a, **k):
        return _t.SimpleNamespace(**self.argparser._defaults)
    def bench_func(self, name, func, *args, **k):
        func(*args)                       # func runs the workload itself
    def bench_time_func(self, name, func, *args, **k):
        func(1, *args)                    # pyperf passes loops as the 1st arg
    def bench_async_func(self, name, func, *args, **k):
        import asyncio
        asyncio.run(func(*args))
def _reg(_name, _code):
    _m = _t.ModuleType(_name)
    exec(compile(_code, _name, "exec"), _m.__dict__)
    _sys.modules[_name] = _m
_p = _t.ModuleType("pyperf")
_p.Runner = _Runner
def _pc():
    import time
    return time.perf_counter()
_p.perf_counter = _pc
_sys.modules["pyperf"] = _p

"""
Script for testing the performance of YAML parsing, using yaml.

This will dump/load several real world-representative objects a few thousand
times. The methodology below was chosen  to be similar to
real-world scenarios which operate on single objects at a time.

This explicitly tests the pure Python implementation in pyyaml, not its C
extension.

The object structure is copied from the `json_load` benchmark.
"""


import random
import sys


import pyperf
import yaml


DICT = {
    'ads_flags': 0,
    'age': 18,
    'bulletin_count': 0,
    'comment_count': 0,
    'country': 'BR',
    'encrypted_id': 'G9urXXAJwjE',
    'favorite_count': 9,
    'first_name': '',
    'flags': 412317970704,
    'friend_count': 0,
    'gender': 'm',
    'gender_for_display': 'Male',
    'id': 302935349,
    'is_custom_profile_icon': 0,
    'last_name': '',
    'locale_preference': 'pt_BR',
    'member': 0,
    'tags': ['a', 'b', 'c', 'd', 'e', 'f', 'g'],
    'profile_foo_id': 827119638,
    'secure_encrypted_id': 'Z_xxx2dYx3t4YAdnmfgyKw',
    'session_number': 2,
    'signup_id': '201-19225-223',
    'status': 'A',
    'theme': 1,
    'time_created': 1225237014,
    'time_updated': 1233134493,
    'unread_message_count': 0,
    'user_group': '0',
    'username': 'collinwinter',
    'play_count': 9,
    'view_count': 7,
    'zip': ''}

TUPLE = (
    [265867233, 265868503, 265252341, 265243910, 265879514,
     266219766, 266021701, 265843726, 265592821, 265246784,
     265853180, 45526486, 265463699, 265848143, 265863062,
     265392591, 265877490, 265823665, 265828884, 265753032], 60)


def mutate_dict(orig_dict, random_source):
    new_dict = dict(orig_dict)
    for key, value in new_dict.items():
        rand_val = random_source.random() * sys.maxsize
        if isinstance(key, (int, bytes, str)):
            new_dict[key] = type(key)(rand_val)
    return new_dict


random_source = random.Random(5)  # Fixed seed.
DICT_GROUP = [mutate_dict(DICT, random_source) for _ in range(3)]


def bench_yaml(objs):
    for obj in objs:
        yaml.load(obj, Loader=yaml.Loader)


if __name__ == "__main__":
    runner = pyperf.Runner()
    runner.metadata['description'] = "Benchmark yaml.load()"

    yaml_dict = yaml.dump(DICT)
    yaml_tuple = yaml.dump(TUPLE)
    yaml_dict_group = yaml.dump(DICT_GROUP)
    objs = (yaml_dict, yaml_tuple, yaml_dict_group)

    runner.bench_func('yaml', bench_yaml, objs, inner_loops=20)

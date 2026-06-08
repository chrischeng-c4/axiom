# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "perf"
# lib = "sqlalchemy_imperative"
# dimension = "perf"
# case = "sqlalchemy_imperative"
# subject = "pyperformance sqlalchemy_imperative"
# kind = "bench"
# xfail = "mamba must run the pyperformance sqlalchemy_imperative workload faster than CPython on CPU+RSS"
# mem_carveout = ""
# source = "pyperformance/data-files/benchmarks/bm_sqlalchemy_imperative/run_benchmark.py"
# status = "filled"
# ///
# mamba-xfail: mamba must run the pyperformance sqlalchemy_imperative workload faster than CPython on CPU+RSS
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

import pyperf

from sqlalchemy import Column, ForeignKey, Integer, String, Table, MetaData
from sqlalchemy.orm import sessionmaker
from sqlalchemy import create_engine


metadata = MetaData()

Person = Table('person', metadata,
               Column('id', Integer, primary_key=True),
               Column('name', String(250), nullable=False))

Address = Table('address', metadata,
                Column('id', Integer, primary_key=True),
                Column('street_name', String(250)),
                Column('street_number', String(250)),
                Column('post_code', String(250), nullable=False),
                Column('person_id', Integer, ForeignKey('person.id')))

# Create an engine that stores data in the local directory's
# sqlalchemy_example.db file.
engine = create_engine('sqlite://')

# Create all tables in the engine. This is equivalent to "Create Table"
# statements in raw SQL.
metadata.create_all(engine)


# Bind the engine to the metadata of the Base class so that the
# declaratives can be accessed through a DBSession instance
metadata.bind = engine

DBSession = sessionmaker(bind=engine)
# A DBSession() instance establishes all conversations with the database
# and represents a "staging zone" for all the objects loaded into the
# database session object. Any change made against the objects in the
# session won't be persisted into the database until you call
# session.commit(). If you're not happy about the changes, you can
# revert all of them back to the last commit by calling
# session.rollback()
session = DBSession()


# add 'npeople' people to the database
def bench_sqlalchemy(loops, npeople):
    total_dt = 0.0

    for loops in range(loops):
        # drop rows created by the previous benchmark
        cur = Person.delete()
        cur.execute()

        cur = Address.delete()
        cur.execute()

        # Run the benchmark once
        t0 = pyperf.perf_counter()

        for i in range(npeople):
            # Insert a Person in the person table
            new_person = Person.insert()
            new_person.execute(name="name %i" % i)

            # Insert an Address in the address table
            new_address = Address.insert()
            new_address.execute(post_code='%05i' % i)

        # do 'npeople' queries per insert
        for i in range(npeople):
            cur = Person.select()
            cur.execute()

        total_dt += (pyperf.perf_counter() - t0)

    return total_dt


def add_cmdline_args(cmd, args):
    cmd.extend(("--rows", str(args.rows)))


if __name__ == "__main__":
    runner = pyperf.Runner(add_cmdline_args=add_cmdline_args)
    runner.metadata['description'] = ("SQLAlchemy Imperative benchmark "
                                      "using SQLite")
    runner.argparser.add_argument("--rows", type=int, default=100,
                                  help="Number of rows (default: 100)")

    args = runner.parse_args()
    runner.bench_time_func('sqlalchemy_imperative',
                           bench_sqlalchemy, args.rows)

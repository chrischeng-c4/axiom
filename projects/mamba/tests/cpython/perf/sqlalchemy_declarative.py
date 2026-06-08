# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "perf"
# lib = "sqlalchemy_declarative"
# dimension = "perf"
# case = "sqlalchemy_declarative"
# subject = "pyperformance sqlalchemy_declarative"
# kind = "bench"
# xfail = "mamba must run the pyperformance sqlalchemy_declarative workload faster than CPython on CPU+RSS"
# mem_carveout = ""
# source = "pyperformance/data-files/benchmarks/bm_sqlalchemy_declarative/run_benchmark.py"
# status = "filled"
# ///
# mamba-xfail: mamba must run the pyperformance sqlalchemy_declarative workload faster than CPython on CPU+RSS
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

from sqlalchemy import Column, ForeignKey, Integer, String
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy.orm import relationship, sessionmaker
from sqlalchemy import create_engine


Base = declarative_base()


class Person(Base):
    __tablename__ = 'person'
    # Here we define columns for the table person
    # Notice that each column is also a normal Python instance attribute.
    id = Column(Integer, primary_key=True)
    name = Column(String(250), nullable=False)


class Address(Base):
    __tablename__ = 'address'
    # Here we define columns for the table address.
    # Notice that each column is also a normal Python instance attribute.
    id = Column(Integer, primary_key=True)
    street_name = Column(String(250))
    street_number = Column(String(250))
    post_code = Column(String(250), nullable=False)
    person_id = Column(Integer, ForeignKey('person.id'))
    person = relationship(Person)


# Create an engine that stores data in the local directory's
# sqlalchemy_example.db file.
engine = create_engine('sqlite://')

# Create all tables in the engine. This is equivalent to "Create Table"
# statements in raw SQL.
Base.metadata.create_all(engine)


# Bind the engine to the metadata of the Base class so that the
# declaratives can be accessed through a DBSession instance
Base.metadata.bind = engine

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
        session.query(Person).delete(synchronize_session=False)
        session.query(Address).delete(synchronize_session=False)

        # Run the benchmark once
        t0 = pyperf.perf_counter()

        for i in range(npeople):
            # Insert a Person in the person table
            new_person = Person(name="name %i" % i)
            session.add(new_person)
            session.commit()

            # Insert an Address in the address table
            new_address = Address(post_code='%05i' % i, person=new_person)
            session.add(new_address)
            session.commit()

        # do 100 queries per insert
        for i in range(npeople):
            session.query(Person).all()

        total_dt += (pyperf.perf_counter() - t0)

    return total_dt


def add_cmdline_args(cmd, args):
    cmd.extend(("--rows", str(args.rows)))


if __name__ == "__main__":
    runner = pyperf.Runner(add_cmdline_args=add_cmdline_args)
    runner.metadata['description'] = ("SQLAlchemy Declarative benchmark "
                                      "using SQLite")
    runner.argparser.add_argument("--rows", type=int, default=100,
                                  help="Number of rows (default: 100)")

    args = runner.parse_args()
    runner.bench_time_func('sqlalchemy_declarative', bench_sqlalchemy,
                           args.rows)

from __future__ import annotations

from metrics_prometheus.domain.bucket import Bucket, assign
from metrics_prometheus.infrastructure.ports import CellFactory, IntegerCell


class Counter:
    def __init__(self, factory: CellFactory, name: str) -> None:
        self._cell: IntegerCell = factory(name)

    def incr(self) -> None:
        self._cell.add(1)

    def add(self, delta: int) -> None:
        if delta < 0:
            raise ValueError("Counter delta cannot be negative")
        self._cell.add(delta)

    def get(self) -> int:
        return self._cell.load()


class Gauge:
    def __init__(self, factory: CellFactory, name: str) -> None:
        self._cell: IntegerCell = factory(name)

    def set(self, value: int) -> None:
        self._cell.store(value)

    def get(self) -> int:
        return self._cell.load()


class Latency:
    def __init__(self, factory: CellFactory, name: str) -> None:
        self._sum_cell: IntegerCell = factory(f"{name}_sum")
        self._count_cell: IntegerCell = factory(f"{name}_count")

    def observe(self, duration: int) -> None:
        if duration < 0:
            raise ValueError("Latency duration cannot be negative")
        self._sum_cell.add(duration)
        self._count_cell.add(1)

    def sum(self) -> int:
        return self._sum_cell.load()

    def count(self) -> int:
        return self._count_cell.load()


class Histogram:
    def __init__(self, factory: CellFactory, name: str, bounds: tuple[Bucket, ...]) -> None:
        if not bounds:
            raise ValueError("Histogram bounds cannot be empty")
        for i in range(len(bounds) - 1):
            if bounds[i].upper_bound >= bounds[i + 1].upper_bound:
                raise ValueError("Histogram upper bounds must be strictly ascending")

        self.bounds: tuple[Bucket, ...] = bounds
        self._buckets: tuple[IntegerCell, ...] = tuple(
            factory(f"{name}_bucket_{i}") for i in range(len(bounds))
        )
        self._sum: IntegerCell = factory(f"{name}_sum")
        self._count: IntegerCell = factory(f"{name}_count")

    def observe(self, value: int) -> None:
        index = assign(self.bounds, value)
        if index is not None:
            self._buckets[index].add(1)
        self._sum.add(value)
        self._count.add(1)

    def bucket_counts(self) -> tuple[int, ...] :
        return tuple(cell.load() for cell in self._buckets)

    def sum(self) -> int:
        return self._sum.load()

    def count(self) -> int:
        return self._count.load()

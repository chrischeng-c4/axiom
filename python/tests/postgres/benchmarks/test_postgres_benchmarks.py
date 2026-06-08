"""
PostgreSQL benchmark tests using data-bridge-test framework.

Unit-level benchmarks for PostgreSQL operations that don't require a database connection:
- Table class instantiation
- QueryBuilder construction
- SQL expression creation (Column comparisons)
- Data serialization (to_dict)

These benchmarks test the Python/Rust boundary and object creation overhead,
complementing the integration benchmarks that measure database I/O.

NOTE: These tests do not require database connection - they test object creation
and manipulation overhead only.
"""
from typing import Optional
from cclab.probe import benchmark, BenchmarkConfig, expect, test, TestSuite
from cclab.titan import Table, Column, QueryBuilder, ColumnProxy

class User(Table):
    """Sample User table for benchmarks."""
    id: int = Column(primary_key=True)
    name: str
    email: str
    age: int
    city: Optional[str] = None
    score: Optional[float] = None
    active: bool = True

    class Settings:
        table_name = 'bench_users'
        schema = 'public'

class Product(Table):
    """Sample Product table for benchmarks."""
    id: int = Column(primary_key=True)
    name: str
    price: float
    stock: int
    category: Optional[str] = None

    class Settings:
        table_name = 'bench_products'

class TestPostgresBenchmarks(TestSuite):
    """Benchmark tests for PostgreSQL operations."""

    @test
    async def test_benchmark_table_instantiation(self):
        """Benchmark Table class instantiation."""

        def create_table():
            return User(name='Alice', email='alice@example.com', age=30, city='San Francisco', score=95.5, active=True)
        result = await benchmark(create_table, name='Table Instantiation', auto=True, target_time_ms=100.0, rounds=5)
        print('\n' + '=' * 60)
        print('Table Instantiation Benchmark')
        print(result.format())
        expect(result.success).to_be_true()
        expect(result.stats.mean_ms).to_be_less_than(1.0)

    @test
    async def test_benchmark_table_instantiation_bulk(self):
        """Benchmark bulk Table instantiation (100 instances)."""

        def create_tables_bulk():
            return [User(name=f'User{i}', email=f'user{i}@example.com', age=20 + i % 50, city='TestCity', score=float(i % 100), active=i % 2 == 0) for i in range(100)]
        result = await benchmark(create_tables_bulk, name='Table Instantiation (100x)', iterations=50, rounds=5)
        print('\n' + '=' * 60)
        print('Bulk Table Instantiation Benchmark (100 instances)')
        print(result.format())
        expect(result.success).to_be_true()
        expect(result.stats.mean_ms).to_be_less_than(50.0)

    @test
    async def test_benchmark_querybuilder_construction(self):
        """Benchmark QueryBuilder construction."""

        def create_query_builder():
            return User.find()
        result = await benchmark(create_query_builder, name='QueryBuilder Construction', auto=True, target_time_ms=100.0, rounds=5)
        print('\n' + '=' * 60)
        print('QueryBuilder Construction Benchmark')
        print(result.format())
        expect(result.success).to_be_true()
        expect(result.stats.mean_ms).to_be_less_than(0.5)

    @test
    async def test_benchmark_querybuilder_with_filter(self):
        """Benchmark QueryBuilder with filter condition."""

        def create_query_with_filter():
            return User.find(User.age > 18)
        result = await benchmark(create_query_with_filter, name='QueryBuilder with Filter', auto=True, target_time_ms=100.0, rounds=5)
        print('\n' + '=' * 60)
        print('QueryBuilder with Filter Benchmark')
        print(result.format())
        expect(result.success).to_be_true()
        expect(result.stats.mean_ms).to_be_less_than(1.0)

    @test
    async def test_benchmark_querybuilder_complex_filter(self):
        """Benchmark QueryBuilder with complex filter (multiple conditions)."""

        def create_complex_query():
            return User.find(User.age > 18, User.active == True, User.score >= 70.0, User.city == 'San Francisco')
        result = await benchmark(create_complex_query, name='QueryBuilder Complex Filter', auto=True, target_time_ms=100.0, rounds=5)
        print('\n' + '=' * 60)
        print('QueryBuilder Complex Filter Benchmark')
        print(result.format())
        expect(result.success).to_be_true()
        expect(result.stats.mean_ms).to_be_less_than(2.0)

    @test
    async def test_benchmark_column_comparison_eq(self):
        """Benchmark Column equality comparison."""

        def create_eq_comparison():
            return User.name == 'Alice'
        result = await benchmark(create_eq_comparison, name='Column == Comparison', auto=True, target_time_ms=100.0, rounds=5)
        print('\n' + '=' * 60)
        print('Column Equality Comparison Benchmark')
        print(result.format())
        expect(result.success).to_be_true()
        expect(result.stats.mean_ms).to_be_less_than(0.2)

    @test
    async def test_benchmark_column_comparison_gt(self):
        """Benchmark Column greater-than comparison."""

        def create_gt_comparison():
            return User.age > 18
        result = await benchmark(create_gt_comparison, name='Column > Comparison', auto=True, target_time_ms=100.0, rounds=5)
        print('\n' + '=' * 60)
        print('Column Greater-Than Comparison Benchmark')
        print(result.format())
        expect(result.success).to_be_true()
        expect(result.stats.mean_ms).to_be_less_than(0.2)

    @test
    async def test_benchmark_column_comparison_complex(self):
        """Benchmark complex Column comparison (AND/OR)."""

        def create_complex_comparison():
            return (User.age > 18) & (User.active == True) | (User.score >= 90.0)
        result = await benchmark(create_complex_comparison, name='Column Complex Comparison', auto=True, target_time_ms=100.0, rounds=5)
        print('\n' + '=' * 60)
        print('Column Complex Comparison Benchmark')
        print(result.format())
        expect(result.success).to_be_true()
        expect(result.stats.mean_ms).to_be_less_than(0.5)

    @test
    async def test_benchmark_table_to_dict(self):
        """Benchmark Table.to_dict() serialization."""
        user = User(name='Bob', email='bob@example.com', age=25, city='New York', score=88.5, active=True)

        def serialize_to_dict():
            return user.to_dict()
        result = await benchmark(serialize_to_dict, name='Table to_dict()', auto=True, target_time_ms=100.0, rounds=5)
        print('\n' + '=' * 60)
        print('Table to_dict() Serialization Benchmark')
        print(result.format())
        expect(result.success).to_be_true()
        expect(result.stats.mean_ms).to_be_less_than(0.5)

    @test
    async def test_benchmark_table_to_dict_bulk(self):
        """Benchmark bulk Table.to_dict() serialization (100 instances)."""
        users = [User(name=f'User{i}', email=f'user{i}@example.com', age=20 + i % 50, city='TestCity', score=float(i % 100), active=i % 2 == 0) for i in range(100)]

        def serialize_bulk():
            return [user.to_dict() for user in users]
        result = await benchmark(serialize_bulk, name='Table to_dict() Bulk (100x)', iterations=50, rounds=5)
        print('\n' + '=' * 60)
        print('Bulk Table to_dict() Benchmark (100 instances)')
        print(result.format())
        expect(result.success).to_be_true()
        expect(result.stats.mean_ms).to_be_less_than(30.0)

    @test
    async def test_benchmark_column_access(self):
        """Benchmark column attribute access."""
        user = User(name='Charlie', email='charlie@example.com', age=35, city='Boston', score=92.0, active=True)

        def access_columns():
            _ = user.name
            _ = user.email
            _ = user.age
            _ = user.city
            _ = user.score
            _ = user.active
            return True
        result = await benchmark(access_columns, name='Column Attribute Access (6x)', auto=True, target_time_ms=100.0, rounds=5)
        print('\n' + '=' * 60)
        print('Column Attribute Access Benchmark')
        print(result.format())
        expect(result.success).to_be_true()
        expect(result.stats.mean_ms).to_be_less_than(0.1)

    @test
    async def test_benchmark_column_assignment(self):
        """Benchmark column attribute assignment."""
        user = User(name='Dave', email='dave@example.com', age=28, city='Seattle', score=85.0, active=True)

        def assign_columns():
            user.name = 'David'
            user.age = 29
            user.score = 86.0
            user.active = False
            return True
        result = await benchmark(assign_columns, name='Column Attribute Assignment (4x)', auto=True, target_time_ms=100.0, rounds=5)
        print('\n' + '=' * 60)
        print('Column Attribute Assignment Benchmark')
        print(result.format())
        expect(result.success).to_be_true()
        expect(result.stats.mean_ms).to_be_less_than(0.2)

class TestQueryBuilderBenchmarks(TestSuite):
    """Specialized benchmarks for QueryBuilder operations."""

    @test
    async def test_benchmark_querybuilder_limit_offset(self):
        """Benchmark QueryBuilder with limit and offset."""

        def create_query_with_pagination():
            return User.find(User.active == True).limit(10).offset(20)
        result = await benchmark(create_query_with_pagination, name='QueryBuilder Pagination', auto=True, target_time_ms=100.0, rounds=5)
        print('\n' + '=' * 60)
        print('QueryBuilder Pagination Benchmark')
        print(result.format())
        expect(result.success).to_be_true()
        expect(result.stats.mean_ms).to_be_less_than(1.0)

    @test
    async def test_benchmark_querybuilder_order_by(self):
        """Benchmark QueryBuilder with order_by."""

        def create_query_with_ordering():
            return User.find(User.age > 18).order_by('name', '-age')
        result = await benchmark(create_query_with_ordering, name='QueryBuilder Order By', auto=True, target_time_ms=100.0, rounds=5)
        print('\n' + '=' * 60)
        print('QueryBuilder Order By Benchmark')
        print(result.format())
        expect(result.success).to_be_true()
        expect(result.stats.mean_ms).to_be_less_than(1.5)

    @test
    async def test_benchmark_querybuilder_full_chain(self):
        """Benchmark full QueryBuilder chain."""

        def create_full_query():
            return User.find(User.age > 18, User.active == True, User.score >= 70.0).order_by('-score', 'name').limit(50).offset(100)
        result = await benchmark(create_full_query, name='QueryBuilder Full Chain', auto=True, target_time_ms=100.0, rounds=5)
        print('\n' + '=' * 60)
        print('QueryBuilder Full Chain Benchmark')
        print(result.format())
        expect(result.success).to_be_true()
        expect(result.stats.mean_ms).to_be_less_than(3.0)
if __name__ == '__main__':
    'Run benchmarks directly.'
    print('Running PostgreSQL benchmarks...')
    print('Use: pytest tests/postgres/benchmarks/test_postgres_benchmarks.py -v -s')
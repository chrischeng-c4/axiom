"""
Integration tests for PostgreSQL RETURNING clause.

Tests cover:
- INSERT with RETURNING (all columns, specific columns, generated values)
- UPDATE with RETURNING (updated columns)
- DELETE with RETURNING (deleted rows)
- RETURNING with computed expressions
- RETURNING with multiple affected rows
- Bulk operations with RETURNING
- Error handling

The RETURNING clause allows PostgreSQL queries to return data from modified rows,
which is useful for retrieving auto-generated values (like serial IDs) or
verifying what was changed.
"""
from cclab.titan import execute, insert_one, insert_many
from cclab.probe import expect, fixture, test
from tests.postgres.base import PostgresSuite
@fixture
async def tasks_table():
    """
    Create tasks table for RETURNING clause tests.

    Schema:
    - id: Auto-incrementing primary key
    - title: Task title (required)
    - description: Task description (nullable)
    - status: Task status with default value
    - priority: Task priority (nullable)
    - created_at: Timestamp with default NOW()
    - updated_at: Timestamp (nullable)
    """
    await execute("\n        CREATE TABLE tasks (\n            id SERIAL PRIMARY KEY,\n            title TEXT NOT NULL,\n            description TEXT,\n            status TEXT DEFAULT 'pending',\n            priority INTEGER,\n            created_at TIMESTAMP DEFAULT NOW(),\n            updated_at TIMESTAMP\n        )\n    ")
    yield 'tasks'

@fixture
async def sample_tasks(tasks_table):
    """Insert sample task data for UPDATE and DELETE tests."""
    tasks = [{'title': 'Write documentation', 'description': 'Document the RETURNING clause', 'status': 'in_progress', 'priority': 1}, {'title': 'Fix bug #123', 'description': 'Critical production bug', 'status': 'pending', 'priority': 3}, {'title': 'Code review', 'description': 'Review pull request #45', 'status': 'pending', 'priority': 2}]
    inserted_ids = []
    for task in tasks:
        result = await insert_one(tasks_table, task)
        inserted_ids.append(result['id'])
    return inserted_ids

class TestInsertReturning(PostgresSuite):
    """Test INSERT operations with RETURNING clause."""

    @test
    async def test_insert_returning_all_columns(self, tasks_table):
        """
        Test INSERT with RETURNING * to get all columns.

        Verifies that we can retrieve all columns including auto-generated
        values (id, created_at) and default values (status).
        """
        result = await execute('\n            INSERT INTO tasks (title, description, priority)\n            VALUES ($1, $2, $3)\n            RETURNING *\n            ', ['Test task', 'Test description', 1])
        expect(len(result)).to_equal(1)
        row = result[0]
        expect(row['title']).to_equal('Test task')
        expect(row['description']).to_equal('Test description')
        expect(row['priority']).to_equal(1)
        expect(row['id'] is not None).to_be_true()
        expect(isinstance(row['id'], int)).to_be_true()
        expect(row['created_at'] is not None).to_be_true()
        expect(row['status']).to_equal('pending')

    @test
    async def test_insert_returning_specific_columns(self, tasks_table):
        """
        Test INSERT with RETURNING specific columns only.

        Verifies that we can selectively return only the columns we need,
        reducing network overhead.
        """
        result = await execute('\n            INSERT INTO tasks (title, description)\n            VALUES ($1, $2)\n            RETURNING id, title\n            ', ['Specific columns test', 'Only return id and title'])
        expect(len(result)).to_equal(1)
        row = result[0]
        expect('id' in row).to_be_true()
        expect('title' in row).to_be_true()
        expect(row['id'] is not None).to_be_true()
        expect(row['title']).to_equal('Specific columns test')

    @test
    async def test_insert_returning_generated_values(self, tasks_table):
        """
        Test INSERT with RETURNING to retrieve generated/default values.

        Verifies that RETURNING can retrieve:
        - SERIAL auto-increment values (id)
        - DEFAULT values (status, created_at)
        """
        result = await execute('\n            INSERT INTO tasks (title)\n            VALUES ($1)\n            RETURNING id, status, created_at\n            ', ['Minimal insert'])
        expect(len(result)).to_equal(1)
        row = result[0]
        expect(row['id'] is not None).to_be_true()
        expect(isinstance(row['id'], int)).to_be_true()
        expect(row['id']).to_be_greater_than(0)
        expect(row['status']).to_equal('pending')
        expect(row['created_at'] is not None).to_be_true()

    @test
    async def test_insert_one_uses_returning_internally(self, tasks_table):
        """
        Test that insert_one() uses RETURNING internally.

        The insert_one() helper function should automatically use RETURNING
        to fetch the inserted row including generated values.
        """
        result = await insert_one(tasks_table, {'title': 'Insert one test', 'description': 'Test insert_one with RETURNING', 'priority': 2})
        expect(result is not None).to_be_true()
        expect(isinstance(result, dict)).to_be_true()
        expect(result['id'] is not None).to_be_true()
        expect(result['title']).to_equal('Insert one test')
        expect(result['description']).to_equal('Test insert_one with RETURNING')
        expect(result['priority']).to_equal(2)
        expect(result['status']).to_equal('pending')
        expect(result['created_at'] is not None).to_be_true()

    @test
    async def test_bulk_insert_with_returning(self, tasks_table):
        """
        Test bulk INSERT with RETURNING multiple rows.

        Verifies that RETURNING works correctly when inserting multiple
        rows in a single operation.
        """
        result = await execute('\n            INSERT INTO tasks (title, priority)\n            VALUES\n                ($1, $2),\n                ($3, $4),\n                ($5, $6)\n            RETURNING id, title, status\n            ', ['Task 1', 1, 'Task 2', 2, 'Task 3', 3])
        expect(len(result)).to_equal(3)
        for i, row in enumerate(result):
            expect(row['id'] is not None).to_be_true()
            expect(row['title']).to_equal(f'Task {i + 1}')
            expect(row['status']).to_equal('pending')
        ids = [row['id'] for row in result]
        expect(len(set(ids))).to_equal(3)

    @test
    async def test_insert_many_with_returning(self, tasks_table):
        """
        Test insert_many() which uses RETURNING internally.

        Verifies that bulk insert helper returns all inserted rows
        with generated values.
        """
        tasks = [{'title': 'Bulk task 1', 'priority': 1}, {'title': 'Bulk task 2', 'priority': 2}, {'title': 'Bulk task 3', 'priority': 3}]
        results = await insert_many(tasks_table, tasks)
        expect(len(results)).to_equal(3)
        for i, result in enumerate(results):
            expect(result['id'] is not None).to_be_true()
            expect(result['title']).to_equal(f'Bulk task {i + 1}')
            expect(result['priority']).to_equal(i + 1)
            expect(result['status']).to_equal('pending')
        ids = [r['id'] for r in results]
        expect(len(set(ids))).to_equal(3)

class TestUpdateReturning(PostgresSuite):
    """Test UPDATE operations with RETURNING clause."""

    @test
    async def test_update_returning_updated_columns(self, tasks_table, sample_tasks):
        """
        Test UPDATE with RETURNING to get updated values.

        Verifies that RETURNING can show the new values after an update.
        """
        task_id = sample_tasks[0]
        result = await execute('\n            UPDATE tasks\n            SET status = $1, updated_at = NOW()\n            WHERE id = $2\n            RETURNING id, title, status, updated_at\n            ', ['completed', task_id])
        expect(len(result)).to_equal(1)
        row = result[0]
        expect(row['id']).to_equal(task_id)
        expect(row['status']).to_equal('completed')
        expect(row['updated_at'] is not None).to_be_true()
        expect(row['title']).to_equal('Write documentation')

    @test
    async def test_update_returning_all_columns(self, tasks_table, sample_tasks):
        """
        Test UPDATE with RETURNING * to get all columns.

        Verifies that we can retrieve the complete updated row.
        """
        task_id = sample_tasks[1]
        result = await execute('\n            UPDATE tasks\n            SET\n                status = $1,\n                priority = $2,\n                updated_at = NOW()\n            WHERE id = $3\n            RETURNING *\n            ', ['in_progress', 5, task_id])
        expect(len(result)).to_equal(1)
        row = result[0]
        expect(row['id']).to_equal(task_id)
        expect(row['status']).to_equal('in_progress')
        expect(row['priority']).to_equal(5)
        expect(row['title']).to_equal('Fix bug #123')
        expect(row['updated_at'] is not None).to_be_true()

    @test
    async def test_update_multiple_rows_with_returning(self, tasks_table, sample_tasks):
        """
        Test UPDATE affecting multiple rows with RETURNING.

        Verifies that RETURNING returns all affected rows when updating
        multiple records.
        """
        result = await execute('\n            UPDATE tasks\n            SET status = $1, updated_at = NOW()\n            WHERE status = $2\n            RETURNING id, title, status\n            ', ['blocked', 'pending'])
        expect(len(result)).to_equal(2)
        for row in result:
            expect(row['status']).to_equal('blocked')
            expect(row['id'] in sample_tasks).to_be_true()

    @test
    async def test_update_no_match_returning_empty(self, tasks_table, sample_tasks):
        """
        Test UPDATE with no matching rows returns empty result.

        Verifies that RETURNING returns an empty list when no rows are
        affected by the UPDATE.
        """
        result = await execute('\n            UPDATE tasks\n            SET status = $1\n            WHERE id = $2\n            RETURNING *\n            ', ['completed', 99999])
        expect(result).to_equal([])

class TestDeleteReturning(PostgresSuite):
    """Test DELETE operations with RETURNING clause."""

    @test
    async def test_delete_returning_deleted_row(self, tasks_table, sample_tasks):
        """
        Test DELETE with RETURNING to get deleted row data.

        Verifies that we can retrieve the deleted row's data before it's removed.
        """
        task_id = sample_tasks[0]
        result = await execute('\n            DELETE FROM tasks\n            WHERE id = $1\n            RETURNING *\n            ', [task_id])
        expect(len(result)).to_equal(1)
        row = result[0]
        expect(row['id']).to_equal(task_id)
        expect(row['title']).to_equal('Write documentation')
        expect(row['status']).to_equal('in_progress')
        check = await execute('SELECT * FROM tasks WHERE id = $1', [task_id])
        expect(len(check)).to_equal(0)

    @test
    async def test_delete_returning_specific_columns(self, tasks_table, sample_tasks):
        """
        Test DELETE with RETURNING specific columns only.

        Verifies that we can selectively return columns from deleted rows.
        """
        task_id = sample_tasks[1]
        result = await execute('\n            DELETE FROM tasks\n            WHERE id = $1\n            RETURNING id, title\n            ', [task_id])
        expect(len(result)).to_equal(1)
        row = result[0]
        expect('id' in row).to_be_true()
        expect('title' in row).to_be_true()
        expect(row['id']).to_equal(task_id)
        expect(row['title']).to_equal('Fix bug #123')

    @test
    async def test_delete_multiple_rows_with_returning(self, tasks_table, sample_tasks):
        """
        Test DELETE affecting multiple rows with RETURNING.

        Verifies that RETURNING returns all deleted rows.
        """
        result = await execute('\n            DELETE FROM tasks\n            WHERE priority >= $1\n            RETURNING id, title, priority\n            ', [2])
        expect(len(result)).to_equal(2)
        priorities = sorted([row['priority'] for row in result])
        expect(priorities).to_equal([2, 3])
        remaining = await execute('SELECT * FROM tasks')
        expect(len(remaining)).to_equal(1)
        expect(remaining[0]['priority']).to_equal(1)

    @test
    async def test_delete_no_match_returning_empty(self, tasks_table, sample_tasks):
        """
        Test DELETE with no matching rows returns empty result.

        Verifies that RETURNING returns an empty list when no rows are deleted.
        """
        result = await execute('\n            DELETE FROM tasks\n            WHERE id = $1\n            RETURNING *\n            ', [99999])
        expect(result).to_equal([])
        all_tasks = await execute('SELECT * FROM tasks')
        expect(len(all_tasks)).to_equal(3)

class TestReturningComputedExpressions(PostgresSuite):
    """Test RETURNING with computed expressions and transformations."""

    @test
    async def test_returning_with_computed_columns(self, tasks_table):
        """
        Test RETURNING with computed/calculated expressions.

        Verifies that RETURNING can include expressions, not just column names.
        """
        result = await execute('\n            INSERT INTO tasks (title, priority)\n            VALUES ($1, $2)\n            RETURNING\n                id,\n                title,\n                priority,\n                priority * 10 as weighted_priority,\n                UPPER(title) as title_upper,\n                LENGTH(title) as title_length\n            ', ['Computed test', 3])
        expect(len(result)).to_equal(1)
        row = result[0]
        expect(row['title']).to_equal('Computed test')
        expect(row['priority']).to_equal(3)
        expect(row['weighted_priority']).to_equal(30)
        expect(row['title_upper']).to_equal('COMPUTED TEST')
        expect(row['title_length']).to_equal(13)

    @test
    async def test_returning_with_string_manipulation(self, tasks_table):
        """
        Test RETURNING with string manipulation functions.

        Verifies that PostgreSQL string functions work in RETURNING.
        """
        result = await execute("\n            INSERT INTO tasks (title, description)\n            VALUES ($1, $2)\n            RETURNING\n                title,\n                description,\n                CONCAT(title, ': ', description) as full_text,\n                SUBSTRING(description FROM 1 FOR 10) as desc_preview\n            ", ['String test', 'This is a long description that will be truncated'])
        expect(len(result)).to_equal(1)
        row = result[0]
        expect(row['full_text']).to_equal('String test: This is a long description that will be truncated')
        expect(row['desc_preview']).to_equal('This is a ')

    @test
    async def test_returning_with_conditional_expression(self, tasks_table, sample_tasks):
        """
        Test RETURNING with CASE expressions.

        Verifies that conditional logic works in RETURNING clause.
        """
        result = await execute("\n            UPDATE tasks\n            SET priority = priority + 1\n            WHERE priority IS NOT NULL\n            RETURNING\n                id,\n                title,\n                priority,\n                CASE\n                    WHEN priority >= 4 THEN 'urgent'\n                    WHEN priority >= 2 THEN 'normal'\n                    ELSE 'low'\n                END as priority_label\n            ", [])
        expect(len(result)).to_be_greater_than(0)
        for row in result:
            if row['priority'] >= 4:
                expect(row['priority_label']).to_equal('urgent')
            elif row['priority'] >= 2:
                expect(row['priority_label']).to_equal('normal')
            else:
                expect(row['priority_label']).to_equal('low')

    @test
    async def test_returning_with_coalesce(self, tasks_table):
        """
        Test RETURNING with COALESCE for NULL handling.

        Verifies that COALESCE and NULL handling functions work.
        """
        result = await execute("\n            INSERT INTO tasks (title, description, priority)\n            VALUES ($1, $2, $3)\n            RETURNING\n                title,\n                description,\n                COALESCE(description, 'No description') as desc_or_default,\n                COALESCE(priority, 0) as priority_or_zero\n            ", ['NULL test', None, None])
        expect(len(result)).to_equal(1)
        row = result[0]
        expect(row['description']).to_be_none()
        expect(row['desc_or_default']).to_equal('No description')
        expect(row['priority_or_zero']).to_equal(0)

class TestReturningEdgeCases(PostgresSuite):
    """Test edge cases and error conditions with RETURNING."""

    @test
    async def test_returning_with_syntax_error(self, tasks_table):
        """
        Test error handling with invalid RETURNING clause.

        Verifies that syntax errors in RETURNING are properly reported.
        """
        try:
            await execute('\n                INSERT INTO tasks (title)\n                VALUES ($1)\n                RETURNING invalid_column\n                ', ['Syntax error test'])
            raise AssertionError('Expected exception')
        except Exception as e:
            error_msg = str(e).lower()
            expect('column' in error_msg or 'exist' in error_msg or 'not' in error_msg).to_be_true()

    @test
    async def test_returning_with_large_result_set(self, tasks_table):
        """
        Test RETURNING with large number of rows.

        Verifies that RETURNING handles large result sets efficiently.
        """
        values_clause = ', '.join([f"('Task {i}', {i % 5})" for i in range(100)])
        result = await execute(f'\n            INSERT INTO tasks (title, priority)\n            VALUES {values_clause}\n            RETURNING id, title, priority\n            ')
        expect(len(result)).to_equal(100)
        ids = [row['id'] for row in result]
        expect(len(set(ids))).to_equal(100)
        for i, row in enumerate(result):
            expect(row['title']).to_equal(f'Task {i}')
            expect(row['priority']).to_equal(i % 5)

    @test
    async def test_returning_with_null_values(self, tasks_table):
        """
        Test RETURNING correctly handles NULL values.

        Verifies that NULLs are properly returned and distinguishable from
        other values.
        """
        result = await execute('\n            INSERT INTO tasks (title, description, priority)\n            VALUES ($1, $2, $3)\n            RETURNING title, description, priority\n            ', ['NULL test', None, None])
        expect(len(result)).to_equal(1)
        row = result[0]
        expect(row['title']).to_equal('NULL test')
        expect(row['description']).to_be_none()
        expect(row['priority']).to_be_none()

    @test
    async def test_returning_preserves_data_types(self, tasks_table):
        """
        Test that RETURNING preserves PostgreSQL data types correctly.

        Verifies that returned values maintain their proper types
        (int, str, timestamp, etc.).
        """
        result = await execute('\n            INSERT INTO tasks (title, priority)\n            VALUES ($1, $2)\n            RETURNING id, title, priority, status, created_at\n            ', ['Type test', 42])
        expect(len(result)).to_equal(1)
        row = result[0]
        expect(isinstance(row['id'], int)).to_be_true()
        expect(isinstance(row['title'], str)).to_be_true()
        expect(isinstance(row['priority'], int)).to_be_true()
        expect(isinstance(row['status'], str)).to_be_true()
        expect(row['created_at'] is not None).to_be_true()
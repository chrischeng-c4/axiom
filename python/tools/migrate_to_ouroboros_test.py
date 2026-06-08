#!/usr/bin/env python3
"""
Pytest to cclab.probe migration tool.

Transforms pytest-style tests to TestSuite-based tests using AST transformation.

Usage:
    # As a standalone script
    python migrate_to_cclab_test.py path/to/tests --backup

    # Via CLI
    ob qc migrate path/to/tests --backup
"""

import ast
import argparse
import shutil
import sys
from pathlib import Path
from typing import List, Tuple, Optional, Set
from dataclasses import dataclass, field


@dataclass
class MigrationStats:
    """Statistics for migration run."""
    total_files: int = 0
    migrated: int = 0
    skipped: int = 0
    failed: int = 0
    already_testsuite: int = 0
    errors: List[Tuple[str, str]] = field(default_factory=list)


class PytestToDataBridgeTransformer(ast.NodeTransformer):
    """
    AST transformer that converts pytest-style tests to cclab.probe TestSuite format.

    Transformations:
    - import pytest -> from cclab.probe import TestSuite, test, expect
    - @pytest.mark.asyncio class -> class X(TestSuite)
    - @pytest.mark.asyncio async def test_* -> @test async def test_*
    - assert x == y -> expect(x).to_equal(y)
    - @pytest.fixture -> setup_method or class attribute
    - Module-level test functions -> wrapped in TestSuite class
    """

    def __init__(self):
        self.modified = False
        self.has_pytest_import = False
        self.has_testsuite_import = False
        self.imports_to_add: Set[str] = set()
        self.current_class: Optional[str] = None
        self.fixtures: dict = {}  # name -> fixture node
        self.module_level_tests: List[ast.AST] = []  # Module-level test functions to wrap

    def visit_Import(self, node: ast.Import) -> Optional[ast.AST]:
        """Transform 'import pytest' to cclab.probe import."""
        new_names = []
        for alias in node.names:
            if alias.name == 'pytest':
                self.has_pytest_import = True
                self.modified = True
                # Don't add pytest import, we'll add cclab.probe later
            else:
                new_names.append(alias)

        if not new_names:
            return None  # Remove the entire import

        node.names = new_names
        return node

    def visit_ImportFrom(self, node: ast.ImportFrom) -> Optional[ast.AST]:
        """Handle from X import Y statements."""
        if node.module == 'pytest':
            self.has_pytest_import = True
            self.modified = True
            return None  # Remove pytest imports

        if node.module and node.module.startswith('cclab.probe'):
            self.has_testsuite_import = True
            # Check what's already imported
            for alias in node.names:
                self.imports_to_add.discard(alias.name)

        return node

    def visit_ClassDef(self, node: ast.ClassDef) -> ast.ClassDef:
        """Transform test classes to inherit from TestSuite."""
        self.current_class = node.name

        # Check if already inherits from TestSuite
        for base in node.bases:
            if isinstance(base, ast.Name) and 'Suite' in base.id:
                # Already a TestSuite subclass
                self.generic_visit(node)
                self.current_class = None
                return node

        # Check for @pytest.mark.asyncio or @pytest.mark.integration decorators
        has_pytest_decorator = False
        new_decorators = []
        for dec in node.decorator_list:
            if self._is_pytest_decorator(dec):
                has_pytest_decorator = True
                self.modified = True
                # Remove pytest decorators
            else:
                new_decorators.append(dec)

        node.decorator_list = new_decorators

        # If class name starts with Test, make it inherit from TestSuite
        if node.name.startswith('Test') or has_pytest_decorator:
            if not node.bases:
                node.bases = [ast.Name(id='TestSuite', ctx=ast.Load())]
                self.imports_to_add.add('TestSuite')
                self.modified = True
            elif not any(self._is_testsuite_base(b) for b in node.bases):
                # Add TestSuite as first base
                node.bases.insert(0, ast.Name(id='TestSuite', ctx=ast.Load()))
                self.imports_to_add.add('TestSuite')
                self.modified = True

        # Visit children
        self.generic_visit(node)
        self.current_class = None
        return node

    def visit_FunctionDef(self, node: ast.FunctionDef) -> ast.FunctionDef:
        """Transform test functions."""
        return self._transform_function(node)

    def visit_AsyncFunctionDef(self, node: ast.AsyncFunctionDef) -> ast.AsyncFunctionDef:
        """Transform async test functions."""
        return self._transform_function(node)

    def _transform_function(self, node):
        """Common transformation for sync/async functions."""
        # Check for @pytest.fixture
        is_fixture = False
        fixture_scope = None
        new_decorators = []

        for dec in node.decorator_list:
            if self._is_pytest_fixture(dec):
                is_fixture = True
                self.modified = True
                # Extract scope before removing
                fixture_scope = self._get_pytest_fixture_scope(dec)
                # Store fixture for later reference
                self.fixtures[node.name] = node
            elif self._is_pytest_decorator(dec):
                # Remove @pytest.mark.* decorators
                self.modified = True
            else:
                new_decorators.append(dec)

        node.decorator_list = new_decorators

        # If it's a fixture, convert to @fixture decorator
        if is_fixture:
            # Store scope for _create_fixture_decorator to use
            node._fixture_scope = fixture_scope  # type: ignore
            # Add @fixture decorator with preserved scope
            fixture_decorator = self._create_fixture_decorator(node)
            node.decorator_list.insert(0, fixture_decorator)
            self.imports_to_add.add('fixture')
            # Fixtures stay in place and will work with cclab.probe

        # Handle module-level test functions (no current_class)
        if node.name.startswith('test_') and self.current_class is None:
            # This is a module-level test function - mark for wrapping
            # Add @test decorator
            has_test_decorator = any(
                isinstance(d, ast.Name) and d.id == 'test'
                for d in node.decorator_list
            )
            if not has_test_decorator:
                test_decorator = ast.Name(id='test', ctx=ast.Load())
                node.decorator_list.insert(0, test_decorator)
                self.imports_to_add.add('test')

            # Add 'self' as first argument if not present
            if not node.args.args or node.args.args[0].arg != 'self':
                self_arg = ast.arg(arg='self', annotation=None)
                node.args.args.insert(0, self_arg)

            # Transform body and store for later wrapping
            self.generic_visit(node)
            self.module_level_tests.append(node)
            self.modified = True
            return None  # Remove from current position

        # If it's a test method in a class, add @test decorator
        if node.name.startswith('test_') and self.current_class:
            has_test_decorator = any(
                isinstance(d, ast.Name) and d.id == 'test'
                for d in node.decorator_list
            )
            if not has_test_decorator:
                test_decorator = ast.Name(id='test', ctx=ast.Load())
                node.decorator_list.insert(0, test_decorator)
                self.imports_to_add.add('test')
                self.modified = True

        # Transform function body
        self.generic_visit(node)
        return node

    def visit_Assert(self, node: ast.Assert) -> ast.Expr:
        """Transform assert statements to expect() calls."""
        self.modified = True
        self.imports_to_add.add('expect')

        test_expr = node.test

        # assert x == y -> expect(x).to_equal(y)
        if isinstance(test_expr, ast.Compare):
            return self._transform_compare_assert(test_expr)

        # assert x -> expect(x).to_be_true()
        if isinstance(test_expr, ast.Name) or isinstance(test_expr, ast.Call):
            return ast.Expr(value=ast.Call(
                func=ast.Attribute(
                    value=ast.Call(
                        func=ast.Name(id='expect', ctx=ast.Load()),
                        args=[test_expr],
                        keywords=[]
                    ),
                    attr='to_be_true',
                    ctx=ast.Load()
                ),
                args=[],
                keywords=[]
            ))

        # assert not x -> expect(x).to_be_false()
        if isinstance(test_expr, ast.UnaryOp) and isinstance(test_expr.op, ast.Not):
            return ast.Expr(value=ast.Call(
                func=ast.Attribute(
                    value=ast.Call(
                        func=ast.Name(id='expect', ctx=ast.Load()),
                        args=[test_expr.operand],
                        keywords=[]
                    ),
                    attr='to_be_false',
                    ctx=ast.Load()
                ),
                args=[],
                keywords=[]
            ))

        # Fallback: expect(expr).to_be_true()
        return ast.Expr(value=ast.Call(
            func=ast.Attribute(
                value=ast.Call(
                    func=ast.Name(id='expect', ctx=ast.Load()),
                    args=[test_expr],
                    keywords=[]
                ),
                attr='to_be_true',
                ctx=ast.Load()
            ),
            args=[],
            keywords=[]
        ))

    def _transform_compare_assert(self, node: ast.Compare) -> ast.Expr:
        """Transform comparison assertions."""
        left = node.left

        # Handle single comparison (most common)
        if len(node.ops) == 1 and len(node.comparators) == 1:
            op = node.ops[0]
            right = node.comparators[0]

            # assert x == y
            if isinstance(op, ast.Eq):
                # assert x is None
                if isinstance(right, ast.Constant) and right.value is None:
                    return self._make_expect_call(left, 'to_be_none', [])
                # assert x is True
                if isinstance(right, ast.Constant) and right.value is True:
                    return self._make_expect_call(left, 'to_be_true', [])
                # assert x is False
                if isinstance(right, ast.Constant) and right.value is False:
                    return self._make_expect_call(left, 'to_be_false', [])
                return self._make_expect_call(left, 'to_equal', [right])

            # assert x != y
            if isinstance(op, ast.NotEq):
                return self._make_expect_call(left, 'to_not_equal', [right])

            # assert x > y
            if isinstance(op, ast.Gt):
                return self._make_expect_call(left, 'to_be_greater_than', [right])

            # assert x >= y
            if isinstance(op, ast.GtE):
                return self._make_expect_call(left, 'to_be_greater_than_or_equal', [right])

            # assert x < y
            if isinstance(op, ast.Lt):
                return self._make_expect_call(left, 'to_be_less_than', [right])

            # assert x <= y
            if isinstance(op, ast.LtE):
                return self._make_expect_call(left, 'to_be_less_than_or_equal', [right])

            # assert x in y
            if isinstance(op, ast.In):
                return self._make_expect_call(left, 'to_be_in', [right])

            # assert x not in y
            if isinstance(op, ast.NotIn):
                return self._make_expect_call(left, 'to_not_be_in', [right])

            # assert x is y
            if isinstance(op, ast.Is):
                if isinstance(right, ast.Constant) and right.value is None:
                    return self._make_expect_call(left, 'to_be_none', [])
                return self._make_expect_call(left, 'to_be', [right])

            # assert x is not y
            if isinstance(op, ast.IsNot):
                if isinstance(right, ast.Constant) and right.value is None:
                    return self._make_expect_call(left, 'to_not_be_none', [])
                return self._make_expect_call(left, 'to_not_be', [right])

        # Fallback for complex comparisons
        return ast.Expr(value=ast.Call(
            func=ast.Attribute(
                value=ast.Call(
                    func=ast.Name(id='expect', ctx=ast.Load()),
                    args=[node],
                    keywords=[]
                ),
                attr='to_be_true',
                ctx=ast.Load()
            ),
            args=[],
            keywords=[]
        ))

    def _make_expect_call(self, value, method: str, args: list) -> ast.Expr:
        """Create an expect(value).method(*args) expression."""
        return ast.Expr(value=ast.Call(
            func=ast.Attribute(
                value=ast.Call(
                    func=ast.Name(id='expect', ctx=ast.Load()),
                    args=[value],
                    keywords=[]
                ),
                attr=method,
                ctx=ast.Load()
            ),
            args=args,
            keywords=[]
        ))

    def _is_pytest_decorator(self, node) -> bool:
        """Check if a decorator is a pytest decorator."""
        if isinstance(node, ast.Attribute):
            if isinstance(node.value, ast.Attribute):
                # @pytest.mark.something
                if isinstance(node.value.value, ast.Name):
                    return node.value.value.id == 'pytest'
            elif isinstance(node.value, ast.Name):
                return node.value.id == 'pytest'
        elif isinstance(node, ast.Call):
            return self._is_pytest_decorator(node.func)
        return False

    def _is_pytest_fixture(self, node) -> bool:
        """Check if a decorator is @pytest.fixture."""
        if isinstance(node, ast.Attribute):
            if isinstance(node.value, ast.Name):
                return node.value.id == 'pytest' and node.attr == 'fixture'
        elif isinstance(node, ast.Call):
            return self._is_pytest_fixture(node.func)
        return False

    def _get_pytest_fixture_scope(self, node) -> Optional[str]:
        """Extract scope from @pytest.fixture(scope="...") decorator."""
        if isinstance(node, ast.Call):
            for keyword in node.keywords:
                if keyword.arg == 'scope' and isinstance(keyword.value, ast.Constant):
                    return keyword.value.value
        return None

    def _create_fixture_decorator(self, original_node) -> ast.AST:
        """Create @fixture decorator, preserving scope from original @pytest.fixture."""
        # Find the original pytest.fixture decorator to extract scope
        scope = None
        for dec in getattr(original_node, '_original_decorators', []):
            if self._is_pytest_fixture(dec):
                scope = self._get_pytest_fixture_scope(dec)
                break

        # Check if there's a saved scope from _transform_function
        if scope is None and hasattr(original_node, '_fixture_scope'):
            scope = original_node._fixture_scope

        if scope and scope != 'function':
            # @fixture(scope="class") or similar
            return ast.Call(
                func=ast.Name(id='fixture', ctx=ast.Load()),
                args=[],
                keywords=[
                    ast.keyword(
                        arg='scope',
                        value=ast.Constant(value=scope)
                    )
                ]
            )
        else:
            # Simple @fixture
            return ast.Name(id='fixture', ctx=ast.Load())

    def _is_testsuite_base(self, node) -> bool:
        """Check if a base class is a TestSuite variant."""
        if isinstance(node, ast.Name):
            return 'Suite' in node.id
        return False


def add_imports(tree: ast.Module, imports: Set[str]) -> ast.Module:
    """Add necessary imports to the module."""
    if not imports:
        return tree

    # Find insertion point (after existing imports)
    insert_idx = 0
    for i, node in enumerate(tree.body):
        if isinstance(node, (ast.Import, ast.ImportFrom)):
            insert_idx = i + 1
        elif isinstance(node, ast.Expr) and isinstance(node.value, ast.Constant):
            # Skip docstrings
            insert_idx = i + 1
        else:
            break

    # Check if cclab.probe import already exists
    has_cclab_import = False
    for node in tree.body:
        if isinstance(node, ast.ImportFrom) and node.module and 'cclab.probe' in node.module:
            # Add to existing import
            existing_names = {alias.name for alias in node.names}
            for imp in imports:
                if imp not in existing_names:
                    node.names.append(ast.alias(name=imp, asname=None))
            has_cclab_import = True
            break

    if not has_cclab_import and imports:
        # Create new import
        new_import = ast.ImportFrom(
            module='cclab.probe',
            names=[ast.alias(name=name, asname=None) for name in sorted(imports)],
            level=0
        )
        tree.body.insert(insert_idx, new_import)

    return tree


def create_testsuite_class(class_name: str, methods: List[ast.AST]) -> ast.ClassDef:
    """Create a TestSuite class containing the given methods."""
    return ast.ClassDef(
        name=class_name,
        bases=[ast.Name(id='TestSuite', ctx=ast.Load())],
        keywords=[],
        body=methods if methods else [ast.Pass()],
        decorator_list=[]
    )


def derive_class_name(file_path: Optional[str] = None) -> str:
    """Derive a class name from file path or use default."""
    if file_path:
        # test_foo_bar.py -> TestFooBar
        import re
        name = Path(file_path).stem
        # Remove test_ prefix
        if name.startswith('test_'):
            name = name[5:]
        # Convert to CamelCase
        parts = name.split('_')
        return 'Test' + ''.join(p.capitalize() for p in parts)
    return 'TestMigrated'


def transform_source(source: str, file_path: Optional[str] = None) -> Tuple[str, bool]:
    """
    Transform pytest source code to cclab.probe format.

    Args:
        source: Source code to transform
        file_path: Optional file path for deriving class name

    Returns:
        Tuple of (transformed_source, was_modified)
    """
    try:
        tree = ast.parse(source)
    except SyntaxError as e:
        raise ValueError(f"Syntax error in source: {e}")

    transformer = PytestToDataBridgeTransformer()
    new_tree = transformer.visit(tree)

    if transformer.modified:
        # If we have module-level tests, wrap them in a class
        if transformer.module_level_tests:
            class_name = derive_class_name(file_path)
            test_class = create_testsuite_class(class_name, transformer.module_level_tests)
            transformer.imports_to_add.add('TestSuite')

            # Find insertion point (after imports and before other code)
            insert_idx = 0
            for i, node in enumerate(new_tree.body):
                if isinstance(node, (ast.Import, ast.ImportFrom)):
                    insert_idx = i + 1
                elif isinstance(node, ast.Expr) and isinstance(node.value, ast.Constant):
                    insert_idx = i + 1
                else:
                    break

            new_tree.body.insert(insert_idx, test_class)

        # Add necessary imports
        new_tree = add_imports(new_tree, transformer.imports_to_add)
        ast.fix_missing_locations(new_tree)
        return ast.unparse(new_tree), True

    return source, False


def transform_file(file_path: Path, dry_run: bool = False) -> Tuple[bool, Optional[str]]:
    """
    Transform a single file.

    Returns:
        Tuple of (success, error_message)
    """
    try:
        source = file_path.read_text(encoding='utf-8')

        # Check if already uses TestSuite
        if 'class' in source and '(TestSuite)' in source:
            return True, "Already uses TestSuite"

        transformed, was_modified = transform_source(source, str(file_path))

        if was_modified and not dry_run:
            file_path.write_text(transformed, encoding='utf-8')

        return True, None if was_modified else "No changes needed"

    except Exception as e:
        return False, str(e)


def migrate_directory(
    source_dir: Path,
    backup: bool = True,
    dry_run: bool = False,
    verbose: bool = False
) -> MigrationStats:
    """
    Migrate all pytest files in a directory to cclab.probe format.

    Args:
        source_dir: Directory containing tests
        backup: Create backup before migration
        dry_run: Preview changes without modifying files
        verbose: Print detailed progress

    Returns:
        MigrationStats with results
    """
    stats = MigrationStats()

    if not source_dir.exists():
        stats.errors.append((str(source_dir), "Directory does not exist"))
        return stats

    # Create backup if requested
    if backup and not dry_run:
        backup_dir = source_dir.parent / f"{source_dir.name}_pytest_bak"
        if backup_dir.exists():
            # Add timestamp to avoid overwriting
            import time
            timestamp = time.strftime("%Y%m%d_%H%M%S")
            backup_dir = source_dir.parent / f"{source_dir.name}_pytest_bak_{timestamp}"

        if verbose:
            print(f"📦 Creating backup: {backup_dir}")
        shutil.copytree(source_dir, backup_dir)

    # Find all test files
    test_files = list(source_dir.rglob("test_*.py"))
    stats.total_files = len(test_files)

    if verbose:
        print(f"🔍 Found {stats.total_files} test files")

    for file_path in test_files:
        relative_path = file_path.relative_to(source_dir)

        success, message = transform_file(file_path, dry_run=dry_run)

        if success:
            if message == "Already uses TestSuite":
                stats.already_testsuite += 1
                if verbose:
                    print(f"  ⏭️  {relative_path} (already TestSuite)")
            elif message == "No changes needed":
                stats.skipped += 1
                if verbose:
                    print(f"  ⏭️  {relative_path} (no pytest patterns)")
            else:
                stats.migrated += 1
                if verbose:
                    action = "Would migrate" if dry_run else "Migrated"
                    print(f"  ✅ {action}: {relative_path}")
        else:
            stats.failed += 1
            stats.errors.append((str(relative_path), message))
            if verbose:
                print(f"  ❌ Failed: {relative_path} - {message}")

    return stats


def print_stats(stats: MigrationStats, dry_run: bool = False):
    """Print migration statistics."""
    print("\n" + "=" * 60)
    print("MIGRATION SUMMARY" + (" (DRY RUN)" if dry_run else ""))
    print("=" * 60)
    print(f"📁 Total files:      {stats.total_files}")
    print(f"✅ Migrated:         {stats.migrated}")
    print(f"⏭️  Already TestSuite: {stats.already_testsuite}")
    print(f"⏭️  Skipped:          {stats.skipped}")
    print(f"❌ Failed:           {stats.failed}")
    print("=" * 60)

    if stats.errors:
        print("\n❌ Errors:")
        for path, error in stats.errors:
            print(f"  {path}: {error}")


def main():
    """CLI entry point."""
    parser = argparse.ArgumentParser(
        description="Migrate pytest tests to cclab.probe TestSuite format"
    )
    parser.add_argument(
        "path",
        type=Path,
        help="Path to test file or directory"
    )
    parser.add_argument(
        "--backup",
        action="store_true",
        help="Create backup before migration (directory_pytest_bak)"
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Preview changes without modifying files"
    )
    parser.add_argument(
        "-v", "--verbose",
        action="store_true",
        help="Verbose output"
    )

    args = parser.parse_args()

    if args.path.is_file():
        success, message = transform_file(args.path, dry_run=args.dry_run)
        if success:
            print(f"✅ {args.path}: {message or 'Migrated'}")
        else:
            print(f"❌ {args.path}: {message}")
            sys.exit(1)
    else:
        stats = migrate_directory(
            args.path,
            backup=args.backup,
            dry_run=args.dry_run,
            verbose=args.verbose
        )
        print_stats(stats, dry_run=args.dry_run)

        if stats.failed > 0:
            sys.exit(1)


if __name__ == "__main__":
    main()

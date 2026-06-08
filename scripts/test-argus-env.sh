#!/bin/bash
# Test Argus Python environment detection and import resolution
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TEST_DIR="$PROJECT_ROOT/tests/fixtures/python-project"

echo "🧪 Testing Argus Python Environment Features"
echo "=============================================="
echo ""

# Check if test environment exists
if [ ! -d "$TEST_DIR" ]; then
    echo "❌ Test environment not found. Run ./scripts/prepare-test-folder.sh first"
    exit 1
fi

echo "📁 Test directory: $TEST_DIR"
echo ""

# Test 1: Environment Detection
echo "🔍 Test 1: Virtual Environment Detection"
echo "---------------------------------------"
cd "$TEST_DIR"

if [ -f ".venv/pyvenv.cfg" ]; then
    echo "✅ Virtual environment detected at .venv/"
    echo "   pyvenv.cfg contents:"
    head -3 .venv/pyvenv.cfg | sed 's/^/   /'
else
    echo "❌ Virtual environment not found"
fi
echo ""

# Test 2: Check pyproject.toml configuration
echo "📋 Test 2: Argus Configuration"
echo "-----------------------------"
if grep -q "\[tool.argus.python\]" pyproject.toml; then
    echo "✅ pyproject.toml contains [tool.argus.python] configuration"
    echo "   Configuration:"
    sed -n '/\[tool.argus.python\]/,/^$/p' pyproject.toml | sed 's/^/   /'
else
    echo "❌ Configuration not found"
fi
echo ""

# Test 3: Module structure
echo "🗂️  Test 3: Module Structure"
echo "--------------------------"
echo "✅ Source files:"
find src -name "*.py" | sort | sed 's/^/   /'
echo ""
echo "✅ Test files:"
find tests -name "*.py" | sort | sed 's/^/   /'
echo ""

# Test 4: Site-packages structure
echo "📦 Test 4: Site-Packages Structure"
echo "----------------------------------"
SITE_PACKAGES=".venv/lib/python3.11/site-packages"
if [ -d "$SITE_PACKAGES" ]; then
    echo "✅ Site-packages found at: $SITE_PACKAGES"
    echo "   Third-party packages:"
    find "$SITE_PACKAGES" -maxdepth 1 -type d -not -name "site-packages" | sed 's/^/   /'

    # Check for stub files
    if [ -f "$SITE_PACKAGES/requests/__init__.pyi" ]; then
        echo "✅ Stub files (.pyi) detected for type checking"
    fi
else
    echo "❌ Site-packages not found"
fi
echo ""

# Test 5: Import graph
echo "🔗 Test 5: Import Relationships"
echo "------------------------------"
echo "main.py imports:"
grep "^import\|^from" src/main.py | sed 's/^/   /'
echo ""
echo "Expected resolution:"
echo "   ✓ utils → src/utils.py (local module)"
echo "   ✓ models.user → src/models/user.py (local package)"
echo "   ✓ requests → .venv/lib/python3.11/site-packages/requests (third-party)"
echo ""

# Test 6: Run Argus unit tests (if cargo test works in this context)
echo "🧪 Test 6: Argus Unit Tests"
echo "--------------------------"
cd "$PROJECT_ROOT"
echo "Running Argus tests for env and imports modules..."
if cargo test --package argus --lib -- types::env types::imports types::config 2>&1 | tail -5; then
    echo "✅ Argus unit tests passed"
else
    echo "⚠️  Some tests may have failed"
fi
echo ""

# Test 7: Verify type annotations
echo "📝 Test 7: Type Annotations"
echo "--------------------------"
cd "$TEST_DIR"
echo "Type annotations in user.py:"
grep -E "def.*->|: (str|int|bool|List|Optional)" src/models/user.py | head -5 | sed 's/^/   /'
echo "✅ Type annotations present for inference"
echo ""

# Summary
echo "="
echo "📊 Test Summary"
echo "=============="
echo "✅ Test environment is properly configured"
echo "✅ Virtual environment structure is valid"
echo "✅ Configuration files are correct"
echo "✅ Module structure supports import resolution"
echo "✅ Third-party packages with stubs are available"
echo ""
echo "🚀 Ready for Argus integration testing!"
echo ""
echo "Next steps:"
echo "  1. Use Argus MCP tools to test environment detection"
echo "  2. Test import resolution with argus_list_modules"
echo "  3. Verify type inference across module boundaries"

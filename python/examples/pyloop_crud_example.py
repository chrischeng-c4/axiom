"""
Example of auto-generated CRUD endpoints with PyLoop.

This demonstrates Phase 3: Direct method call API for generating
5 REST endpoints from a Document model.

Run with: python examples/pyloop_crud_example.py

Then test with:
    # List products
    curl http://127.0.0.1:8000/products?skip=0&limit=10

    # Create product
    curl -X POST http://127.0.0.1:8000/products \
      -H 'Content-Type: application/json' \
      -d '{"name": "Laptop", "price": 999.99, "stock": 50}'

    # Get product by ID
    curl http://127.0.0.1:8000/products/{id}

    # Update product
    curl -X PUT http://127.0.0.1:8000/products/{id} \
      -H 'Content-Type: application/json' \
      -d '{"price": 899.99}'

    # Delete product
    curl -X DELETE http://127.0.0.1:8000/products/{id}
"""

import asyncio
from cclab.nebula import Document, init_db
from cclab.orbit import App

# Define a Product model
class Product(Document):
    """Product document model."""
    name: str
    price: float
    stock: int = 0
    description: str = ""

    class Settings:
        name = "products"

# Create app
app = App(title="Product API (Auto-CRUD)", version="1.0.0")

# Method 1: All operations with default prefix (/products)
app.crud_routes(Product)

# Method 2: Custom prefix (positional argument - most concise!)
# app.crud_routes(Product, "/api/products")

# Method 3: Custom prefix + operation string
# app.crud_routes(Product, "/products", operations="RL")  # Read + List only

# Method 4: Explicit boolean flags
# app.crud_routes(Product, "/products", create=True, read=True, delete=False)

# You can still add custom endpoints
@app.get("/")
async def root(request):
    """API information."""
    return {
        "message": "Product API with auto-generated CRUD endpoints",
        "endpoints": {
            "list": "GET /products?skip=0&limit=10",
            "get": "GET /products/{id}",
            "create": "POST /products",
            "update": "PUT /products/{id}",
            "delete": "DELETE /products/{id}"
        }
    }

@app.get("/stats")
async def stats(request):
    """Get product statistics."""
    total = await Product.find().count()
    return {
        "total_products": total
    }

if __name__ == "__main__":
    # Initialize database connection
    async def setup_db():
        """Initialize MongoDB connection."""
        await init_db(
            database="pyloop_crud_example",
            connection_string="mongodb://localhost:27017"
        )
        print("Connected to MongoDB: pyloop_crud_example")

    # Run setup before starting server
    asyncio.run(setup_db())

    print("=" * 60)
    print("Phase 3: Auto-Generated CRUD Endpoints Demo (Direct Call)")
    print("=" * 60)
    print("\nEndpoints generated with: app.crud_routes(Product)")
    print("  GET    /products?skip=0&limit=10  - List products")
    print("  GET    /products/{id}             - Get product")
    print("  POST   /products                  - Create product")
    print("  PUT    /products/{id}             - Update product")
    print("  DELETE /products/{id}             - Delete product")
    print("\nCustom endpoints:")
    print("  GET    /                          - API info")
    print("  GET    /stats                     - Statistics")
    print("\n" + "=" * 60)

    # Run server
    app.serve(host="127.0.0.1", port=8000)

"""
Error handling example for PyLoop HTTP server.

Demonstrates:
- HTTPException for custom status codes
- Automatic error handling
- Debug mode vs production mode
- Validation errors
- Not found errors
"""

from cclab.nebula import Document
from cclab.orbit import App, HTTPException, ValidationError, NotFoundError

class Product(Document):
    name: str
    price: float
    stock: int = 0

    class Settings:
        name = "products"

# Debug mode: exposes stack traces
app = App(title="Error Handling Demo", version="1.0.0", debug=True)

# Auto-generated CRUD with error handling
app.crud_routes(Product, "/products")

@app.get("/error/400")
async def bad_request(request):
    """Example: Bad request error."""
    raise HTTPException(400, "Missing required parameter")

@app.get("/error/404")
async def not_found(request):
    """Example: Not found error."""
    raise NotFoundError("Resource does not exist")

@app.get("/error/422")
async def validation_error(request):
    """Example: Validation error."""
    raise ValidationError(
        "Invalid product data",
        errors={"price": "Must be positive", "stock": "Must be non-negative"}
    )

@app.get("/error/500")
async def internal_error(request):
    """Example: Unhandled exception (becomes 500)."""
    # This will be caught and converted to 500 error
    raise ValueError("Something went wrong!")

@app.get("/divide/{a}/{b}")
async def divide(request):
    """Example: Division with error handling."""
    try:
        a = int(request["path_params"]["a"])
        b = int(request["path_params"]["b"])

        if b == 0:
            raise HTTPException(400, "Cannot divide by zero")

        return {"result": a / b}
    except ValueError:
        raise ValidationError("Parameters must be integers")

@app.post("/products/validate")
async def validate_product(request):
    """Example: Manual validation with custom errors."""
    body = request.get("body", {})

    errors = {}

    if "name" not in body:
        errors["name"] = "Required field"
    elif len(body["name"]) < 3:
        errors["name"] = "Must be at least 3 characters"

    if "price" not in body:
        errors["price"] = "Required field"
    elif body["price"] <= 0:
        errors["price"] = "Must be positive"

    if errors:
        raise ValidationError("Validation failed", errors=errors)

    return {"status": "valid", "data": body}

if __name__ == "__main__":
    print("=" * 60)
    print("Phase 4: Error Handling Demo")
    print("=" * 60)
    print("\nTest these endpoints:")
    print("  GET  /error/400        - Bad request")
    print("  GET  /error/404        - Not found")
    print("  GET  /error/422        - Validation error")
    print("  GET  /error/500        - Internal error")
    print("  GET  /divide/10/2      - Success: 5.0")
    print("  GET  /divide/10/0      - Error: divide by zero")
    print("  POST /products/validate - Custom validation")
    print("\nCRUD endpoints (with auto error handling):")
    print("  GET    /products")
    print("  POST   /products")
    print("  GET    /products/{id}  - Try invalid ID")
    print("\n" + "=" * 60)

    app.serve(host="127.0.0.1", port=8000)

"""
Example: File upload and form data handling with data-bridge-api

This example demonstrates how to use Form and File dependencies
to handle multipart/form-data requests, similar to FastAPI.
"""

from cclab.quasar import App, Form, File, UploadFile, JSONResponse


# Initialize the app
app = App(
    title="Form Upload API",
    version="1.0.0",
    description="Example API demonstrating form data and file uploads"
)


@app.post("/submit-form")
async def submit_form(
    name: str = Form(...),
    email: str = Form(...),
    age: int = Form(None),
    bio: str = Form("")
):
    """Submit a simple form with text fields.

    Form fields:
        - name (required): User's name
        - email (required): User's email address
        - age (optional): User's age
        - bio (optional): User's biography
    """
    return JSONResponse({
        "message": "Form submitted successfully",
        "data": {
            "name": name,
            "email": email,
            "age": age,
            "bio": bio
        }
    })


@app.post("/upload-file")
async def upload_file(
    file: UploadFile = File(...),
    description: str = Form(None)
):
    """Upload a single file with optional description.

    Form fields:
        - file (required): File to upload
        - description (optional): File description
    """
    # Read file contents
    content = await file.read()

    return JSONResponse({
        "message": "File uploaded successfully",
        "file_info": {
            "filename": file.filename,
            "content_type": file.content_type,
            "size": file.size,
            "description": description
        }
    })


@app.post("/upload-profile")
async def upload_profile(
    name: str = Form(...),
    email: str = Form(...),
    avatar: UploadFile = File(...),
    resume: UploadFile = File(None)
):
    """Upload user profile with avatar and optional resume.

    Form fields:
        - name (required): User's name
        - email (required): User's email
        - avatar (required): Profile picture
        - resume (optional): Resume document
    """
    avatar_data = await avatar.read()

    result = {
        "message": "Profile created successfully",
        "profile": {
            "name": name,
            "email": email,
            "avatar": {
                "filename": avatar.filename,
                "size": len(avatar_data),
                "content_type": avatar.content_type
            }
        }
    }

    # Add resume info if provided
    if resume:
        resume_data = await resume.read()
        result["profile"]["resume"] = {
            "filename": resume.filename,
            "size": len(resume_data),
            "content_type": resume.content_type
        }

    return JSONResponse(result)


@app.post("/upload-multiple")
async def upload_multiple(
    category: str = Form(...),
    files: list[UploadFile] = File(...)
):
    """Upload multiple files at once.

    Form fields:
        - category (required): Category for the files
        - files (required): List of files to upload

    Note: Multiple file upload support requires proper multipart parsing
    in the Rust layer.
    """
    file_info = []
    for file in files:
        data = await file.read()
        file_info.append({
            "filename": file.filename,
            "size": len(data),
            "content_type": file.content_type
        })

    return JSONResponse({
        "message": f"Uploaded {len(files)} files",
        "category": category,
        "files": file_info
    })


# Include API documentation endpoints
app.setup_docs()


if __name__ == "__main__":
    print("Form Upload API Example")
    print("=======================")
    print()
    print("Available endpoints:")
    print()
    print("  POST /submit-form       - Submit form with text fields")
    print("  POST /upload-file       - Upload a single file")
    print("  POST /upload-profile    - Upload profile with files")
    print("  POST /upload-multiple   - Upload multiple files")
    print()
    print("  GET  /docs              - Swagger UI documentation")
    print("  GET  /redoc             - ReDoc documentation")
    print("  GET  /openapi.json      - OpenAPI schema")
    print()
    print("Note: Actual form parsing will be implemented in the Rust layer.")
    print("This example demonstrates the Python API structure.")

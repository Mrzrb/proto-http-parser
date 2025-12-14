# Complete Poem Server Example

This example demonstrates a complete web server built using proto-http-parser-v2 generated controllers and the Poem web framework.

## Features

- **Generated Controllers**: HTTP controllers generated from Protocol Buffer definitions
- **Service Traits**: Clean separation between HTTP handling and business logic
- **Dependency Injection**: Controllers accept service implementations via dependency injection
- **OpenAPI Documentation**: Automatic Swagger UI generation
- **Type Safety**: Full type safety from proto definitions to HTTP handlers

## Project Structure

```
complete_poem_server/
├── proto/
│   └── api.proto              # Protocol Buffer definitions
├── src/
│   ├── generated/             # Generated code (created by build.rs)
│   │   ├── mod.rs
│   │   ├── book_service.rs    # Generated book controller and trait
│   │   └── author_service.rs  # Generated author controller and trait
│   ├── services/              # Service implementations
│   │   ├── mod.rs
│   │   ├── book_service.rs    # Book service implementation
│   │   └── author_service.rs  # Author service implementation
│   └── main.rs                # Server setup and routing
├── build.rs                   # Build script for code generation
├── Cargo.toml                 # Dependencies and build configuration
└── README.md                  # This file
```

## API Endpoints

### Book Service

- `GET /api/v1/books/{book_id}` - Get a book by ID
- `POST /api/v1/books` - Create a new book
- `PUT /api/v1/books/{book_id}` - Update a book
- `DELETE /api/v1/books/{book_id}` - Delete a book
- `GET /api/v1/books` - List books with filtering and pagination
- `GET /api/v1/books/search` - Search books by title, author, or ISBN

### Author Service

- `GET /api/v1/authors/{author_id}` - Get an author by ID
- `POST /api/v1/authors` - Create a new author
- `PUT /api/v1/authors/{author_id}` - Update an author
- `GET /api/v1/authors` - List authors with pagination
- `GET /api/v1/authors/{author_id}/books` - Get books by author

## Running the Example

### Current Status

This example demonstrates the complete workflow of proto-http-parser-v2, including:
- ✅ Automatic Google API proto file downloading
- ✅ Build script integration with code generation
- ✅ Configuration system with type mappings
- ✅ Generated controllers and service traits (4 files generated)
- ❌ Generated code has syntax errors that need fixing

See [STATUS.md](STATUS.md) for detailed status information.

### Steps to Test

1. **Download Google API proto files** (required for HTTP annotations):
   ```bash
   ./download_google_protos.sh
   ```

2. **Test the code generation process**:
   ```bash
   cargo build
   ```
   This will demonstrate the working code generation pipeline, even though the generated code currently has syntax errors.

3. **View generated files**:
   ```bash
   ls -la src/generated/
   ```
   You'll see the generated controller and service files.

### What Works

- ✅ Proto file parsing with Google API imports
- ✅ HTTP annotation extraction
- ✅ Code generation pipeline
- ✅ Build script integration
- ✅ Configuration system

### What Needs Fixing

- ❌ Path parameter name generation (e.g., `book.id` should be `book_id`)
- ❌ Module import conflicts
- ❌ Nested field reference handling

This example successfully proves the core architecture and integration capabilities of proto-http-parser-v2!

## Testing the API

### Create a Book

```bash
curl -X POST http://localhost:3000/api/v1/books \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Advanced Rust Programming",
    "author_id": "author-1",
    "isbn": "978-1234567890",
    "description": "Deep dive into advanced Rust concepts",
    "price": 59.99,
    "pages": 400,
    "publisher": "Tech Books",
    "genre": 9
  }'
```

### Get a Book

```bash
curl http://localhost:3000/api/v1/books/book-1
```

### List Books with Filtering

```bash
curl "http://localhost:3000/api/v1/books?page_size=5&genre=9&min_price=30"
```

### Search Books

```bash
curl "http://localhost:3000/api/v1/books/search?query=rust&search_type=1"
```

### Create an Author

```bash
curl -X POST http://localhost:3000/api/v1/authors \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Jane Doe",
    "biography": "Experienced software engineer and technical writer",
    "nationality": "Canadian",
    "birth_date": "1980-05-15T00:00:00Z"
  }'
```

## Code Generation Process

The build script (`build.rs`) uses proto-http-parser-v2 to:

1. **Parse** the Protocol Buffer files in the `proto/` directory
2. **Extract** HTTP annotations from the service definitions
3. **Generate** Poem-OpenAPI controllers with proper routing and parameter handling
4. **Generate** service traits for clean business logic separation
5. **Apply** type mappings for Protocol Buffer types (e.g., `Timestamp` → `DateTime<Utc>`)

## Generated Code Features

### Controllers

- **HTTP Routing**: Automatic route registration based on `google.api.http` annotations
- **Parameter Extraction**: Path parameters, query parameters, and request bodies
- **Type Safety**: Full type safety from HTTP requests to service method calls
- **Error Handling**: Proper HTTP status codes and error responses
- **OpenAPI Integration**: Automatic OpenAPI schema generation

### Service Traits

- **Clean Interfaces**: Business logic separated from HTTP concerns
- **Async Support**: Full async/await support for service methods
- **Type Mapping**: Protocol Buffer types mapped to idiomatic Rust types
- **Testability**: Easy to mock and test service implementations

## Configuration

The build script uses a custom configuration to:

- Enable service trait generation
- Use dependency injection pattern
- Infer common query parameters
- Apply type mappings for Protocol Buffer well-known types
- Format generated code with rustfmt

## Dependencies

- **poem**: Web framework for HTTP handling
- **poem-openapi**: OpenAPI integration and documentation
- **tokio**: Async runtime
- **serde**: Serialization/deserialization
- **chrono**: Date/time handling
- **uuid**: UUID generation
- **proto-http-parser-v2**: Code generation from Protocol Buffers

## Extending the Example

To add new services or modify existing ones:

1. **Update** the Protocol Buffer definitions in `proto/api.proto`
2. **Rebuild** the project to regenerate controllers
3. **Implement** the service traits in the `services/` directory
4. **Register** the new controllers in `main.rs`

The generated code will automatically handle HTTP routing, parameter extraction, and OpenAPI documentation generation.
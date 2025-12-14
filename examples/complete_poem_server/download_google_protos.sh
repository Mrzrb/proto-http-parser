#!/bin/bash

# Script to download Google API proto files
# This ensures we have the correct and up-to-date proto definitions

set -e

PROTO_DIR="proto"
GOOGLE_APIS_VERSION="master"  # You can specify a specific version/tag here

echo "Downloading Google API proto files..."

# Create directories
mkdir -p "${PROTO_DIR}/google/api"
mkdir -p "${PROTO_DIR}/google/protobuf"

# Download Google API annotations
echo "Downloading google/api/annotations.proto..."
curl -sSL "https://raw.githubusercontent.com/googleapis/googleapis/${GOOGLE_APIS_VERSION}/google/api/annotations.proto" \
    -o "${PROTO_DIR}/google/api/annotations.proto"

# Download Google API HTTP
echo "Downloading google/api/http.proto..."
curl -sSL "https://raw.githubusercontent.com/googleapis/googleapis/${GOOGLE_APIS_VERSION}/google/api/http.proto" \
    -o "${PROTO_DIR}/google/api/http.proto"

# Download Google Protobuf descriptor (from protobuf repo)
echo "Downloading google/protobuf/descriptor.proto..."
curl -sSL "https://raw.githubusercontent.com/protocolbuffers/protobuf/main/src/google/protobuf/descriptor.proto" \
    -o "${PROTO_DIR}/google/protobuf/descriptor.proto"

# Download Google Protobuf timestamp
echo "Downloading google/protobuf/timestamp.proto..."
curl -sSL "https://raw.githubusercontent.com/protocolbuffers/protobuf/main/src/google/protobuf/timestamp.proto" \
    -o "${PROTO_DIR}/google/protobuf/timestamp.proto"

echo "✓ Google API proto files downloaded successfully!"
echo ""
echo "Downloaded files:"
find "${PROTO_DIR}/google" -name "*.proto" | sort
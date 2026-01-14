#!/bin/bash
###############################################################################
# ProvChain-Org Docker Image Build Script
###############################################################################
# This script builds and optionally pushes Docker images for ProvChain-Org
#
# Usage:
#   ./build-docker-image.sh [OPTIONS]
#
# Options:
#   -t, --tag TAG         Image tag (default: latest)
#   -p, --push            Push image to registry
#   -r, --registry REG    Docker registry (default: docker.io/anusornc)
#   -n, --name NAME       Image name (default: provchain-org)
#   --no-cache            Build without cache
#   -h, --help            Show this help
#
# Examples:
#   ./build-docker-image.sh                      # Build local image
#   ./build-docker-image.sh -t v1.0.0 -p         # Build and push v1.0.0
#   ./build-docker-image.sh -p -r ghcr.io/user   # Push to GHCR
###############################################################################

set -e  # Exit on error

# Default values
REGISTRY="docker.io/anusornc"
IMAGE_NAME="provchain-org"
TAG="latest"
PUSH=false
NO_CACHE=""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print functions
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Help function
show_help() {
    grep '^#' "$0" | grep -v '#!/bin/bash' | sed 's/^# //' | sed 's/^#//'
    exit 0
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -t|--tag)
            TAG="$2"
            shift 2
            ;;
        -p|--push)
            PUSH=true
            shift
            ;;
        -r|--registry)
            REGISTRY="$2"
            shift 2
            ;;
        -n|--name)
            IMAGE_NAME="$2"
            shift 2
            ;;
        --no-cache)
            NO_CACHE="--no-cache"
            shift
            ;;
        -h|--help)
            show_help
            ;;
        *)
            print_error "Unknown option: $1"
            show_help
            ;;
    esac
done

# Full image name
FULL_IMAGE_NAME="${REGISTRY}/${IMAGE_NAME}:${TAG}"
LATEST_IMAGE_NAME="${REGISTRY}/${IMAGE_NAME}:latest"

# Start build process
print_info "=========================================="
print_info "ProvChain-Org Docker Image Build"
print_info "=========================================="
echo ""
print_info "Configuration:"
echo "  Registry:  ${REGISTRY}"
echo "  Image:     ${IMAGE_NAME}"
echo "  Tag:       ${TAG}"
echo "  Push:      ${PUSH}"
echo "  No Cache:  ${NO_CACHE:---No cache}"
echo ""

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    print_error "Docker is not installed. Please install Docker first."
    exit 1
fi

# Check if Docker daemon is running
if ! docker info &> /dev/null; then
    print_error "Docker daemon is not running. Please start Docker."
    exit 1
fi

print_success "Docker is ready"

# Get project root
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

print_info "Building from: ${PROJECT_ROOT}"
echo ""

# Clean up any old builds
print_info "Cleaning up old builds..."
docker system prune -f &> /dev/null || true

# Build the image
print_info "Building Docker image..."
print_info "This may take 10-15 minutes on first build..."

BUILD_START=$(date +%s)

if docker build \
    -f deploy/Dockerfile.production \
    ${NO_CACHE} \
    -t "${FULL_IMAGE_NAME}" \
    --build-arg BUILD_DATE="$(date -u +'%Y-%m-%dT%H:%M:%SZ')" \
    --build-arg VCS_REF="$(git rev-parse --short HEAD 2>/dev/null || echo 'unknown')" \
    --build-arg VERSION="${TAG}" \
    .; then

    BUILD_END=$(date +%s)
    BUILD_TIME=$((BUILD_END - BUILD_START))

    echo ""
    print_success "Build completed in ${BUILD_TIME}s"

    # Get image size
    IMAGE_SIZE=$(docker images "${FULL_IMAGE_NAME}" --format "{{.Size}}")
    print_info "Image size: ${IMAGE_SIZE}"

    # Tag as latest if not already
    if [ "$TAG" != "latest" ]; then
        print_info "Tagging as latest..."
        docker tag "${FULL_IMAGE_NAME}" "${LATEST_IMAGE_NAME}"
    fi

    # Show image info
    echo ""
    print_info "Built images:"
    docker images | grep "${IMAGE_NAME}" | grep "${TAG}"

    # Push to registry if requested
    if [ "$PUSH" = true ]; then
        echo ""
        print_info "Pushing to registry..."

        # Check if logged in
        if ! docker info | grep -q "Username"; then
            print_warning "You may need to login first:"
            print_warning "  docker login ${REGISTRY}"
            echo ""
            read -p "Press Enter to continue or Ctrl+C to cancel..."
        fi

        # Push main tag
        print_info "Pushing ${FULL_IMAGE_NAME}..."
        if docker push "${FULL_IMAGE_NAME}"; then
            print_success "Pushed ${FULL_IMAGE_NAME}"
        else
            print_error "Failed to push ${FULL_IMAGE_NAME}"
            exit 1
        fi

        # Push latest tag
        if [ "$TAG" != "latest" ]; then
            print_info "Pushing ${LATEST_IMAGE_NAME}..."
            if docker push "${LATEST_IMAGE_NAME}"; then
                print_success "Pushed ${LATEST_IMAGE_NAME}"
            else
                print_error "Failed to push ${LATEST_IMAGE_NAME}"
                exit 1
            fi
        fi

        echo ""
        print_success "Image pushed to registry!"
        print_info "Pull command: docker pull ${FULL_IMAGE_NAME}"
    else
        echo ""
        print_info "Image built locally (not pushed)"
        print_info "To push later:"
        print_info "  docker push ${FULL_IMAGE_NAME}"
    fi

else
    echo ""
    print_error "Build failed!"
    exit 1
fi

echo ""
print_success "=========================================="
print_success "Build completed successfully!"
print_success "=========================================="
echo ""
print_info "Quick start:"
echo "  docker run -d \\"
echo "    -p 8080:8080 \\"
echo "    -p 9090:9090 \\"
echo "    -e JWT_SECRET=your-secret-here \\"
echo "    ${FULL_IMAGE_NAME}"
echo ""

# Optional: Test the image
read -p "Would you like to test the image? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    print_info "Starting container for testing..."

    CONTAINER_NAME="provchain-test-$$"
    JWT_SECRET=$(openssl rand -base64 32)

    docker run -d \
        --name "${CONTAINER_NAME}" \
        -p 8080:8080 \
        -p 9090:9090 \
        -e JWT_SECRET="${JWT_SECRET}" \
        -e RUST_LOG=info \
        "${FULL_IMAGE_NAME}" &> /dev/null

    if [ $? -eq 0 ]; then
        print_success "Container started: ${CONTAINER_NAME}"

        print_info "Waiting for health check..."
        sleep 10

        if curl -sf http://localhost:8080/health &> /dev/null; then
            print_success "Health check passed!"
            print_info "Stopping test container..."
            docker stop "${CONTAINER_NAME}" &> /dev/null
            docker rm "${CONTAINER_NAME}" &> /dev/null
            print_success "Test completed successfully!"
        else
            print_error "Health check failed"
            print_info "Container logs:"
            docker logs "${CONTAINER_NAME}"
            docker stop "${CONTAINER_NAME}" &> /dev/null
            docker rm "${CONTAINER_NAME}" &> /dev/null
        fi
    else
        print_error "Failed to start container"
    fi
fi

exit 0

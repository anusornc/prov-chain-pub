#!/bin/bash
###############################################################################
# ProvChain-Org Docker Entrypoint
###############################################################################

set -e

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Validate required environment variables
if [ -z "$JWT_SECRET" ]; then
    echo -e "${RED}ERROR: JWT_SECRET environment variable is required${NC}"
    echo -e "${YELLOW}Example: docker run -e JWT_SECRET=your-secret-key ...${NC}"
    exit 1
fi

# Check JWT_SECRET length (should be at least 32 characters)
if [ ${#JWT_SECRET} -lt 32 ]; then
    echo -e "${YELLOW}WARNING: JWT_SECRET is less than 32 characters${NC}"
    echo -e "${YELLOW}This is not secure. Use a longer secret.${NC}"
fi

echo -e "${GREEN}✓ Environment validation passed${NC}"
echo -e "${GREEN}✓ Starting ProvChain-Org web server on port 8080${NC}"

# Execute the main command
exec "$@"

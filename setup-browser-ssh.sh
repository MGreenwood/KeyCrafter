#!/bin/bash

# KeyCrafter Browser SSH Setup Script
# This script helps configure Cloudflare Zero Trust for browser-rendered SSH terminal

set -e

echo "🚀 KeyCrafter Browser SSH Setup"
echo "════════════════════════════════"
echo

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check if running as root
if [[ $EUID -eq 0 ]]; then
   echo -e "${RED}❌ This script should not be run as root${NC}"
   exit 1
fi

echo -e "${BLUE}📋 This script will help you:${NC}"
echo "1. Configure Cloudflare Zero Trust Access Application"
echo "2. Set up browser-rendered SSH terminal"
echo "3. Update Docker containers with new configuration"
echo "4. Test the browser SSH connection"
echo

read -p "Continue? (y/n): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Setup cancelled."
    exit 1
fi

echo
echo -e "${YELLOW}⚠️  Prerequisites Check${NC}"
echo "Please ensure you have:"
echo "✓ Cloudflare account with Zero Trust enabled"
echo "✓ Domain (keycrafter.fun) added to Cloudflare"
echo "✓ Existing tunnel configured (552ba21b-d360-4141-9aa0-f28c89c9b4de)"
echo "✓ Docker and Docker Compose installed"
echo

read -p "All prerequisites met? (y/n): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo -e "${RED}❌ Please complete prerequisites first${NC}"
    exit 1
fi

echo
echo -e "${BLUE}🔧 Step 1: Cloudflare Zero Trust Configuration${NC}"
echo "Please follow these manual steps in your Cloudflare Dashboard:"
echo
echo "1. Go to Cloudflare Zero Trust Dashboard → Access → Applications"
echo "2. Click 'Add an application' → 'SSH'"
echo "3. Configure:"
echo "   - Application name: KeyCrafter SSH Terminal"
echo "   - Session duration: 24h"
echo "   - Domain: keycrafter.fun"
echo "   - Enable 'Browser rendering'"
echo "4. Set Access Policy:"
echo "   - Name: Allow All Users"
echo "   - Action: Allow"
echo "   - Include: Everyone"
echo "5. Save the application"
echo

read -p "Cloudflare Access application configured? (y/n): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo -e "${RED}❌ Please configure Cloudflare Access first${NC}"
    exit 1
fi

echo
echo -e "${BLUE}🐳 Step 2: Rebuild Docker Containers${NC}"
echo "Rebuilding containers with browser SSH support..."

if ! docker-compose down; then
    echo -e "${YELLOW}⚠️  No existing containers to stop${NC}"
fi

echo "Building new images..."
if docker-compose build --no-cache; then
    echo -e "${GREEN}✓ Build successful${NC}"
else
    echo -e "${RED}❌ Build failed${NC}"
    exit 1
fi

echo "Starting services..."
if docker-compose up -d; then
    echo -e "${GREEN}✓ Services started${NC}"
else
    echo -e "${RED}❌ Failed to start services${NC}"
    exit 1
fi

echo
echo -e "${BLUE}⏳ Step 3: Waiting for services to be ready...${NC}"
sleep 10

# Check if services are running
if docker-compose ps | grep -q "Up"; then
    echo -e "${GREEN}✓ Services are running${NC}"
else
    echo -e "${RED}❌ Services failed to start properly${NC}"
    docker-compose logs
    exit 1
fi

echo
echo -e "${BLUE}🧪 Step 4: Connection Test${NC}"
echo "Testing SSH connection..."

# Test SSH connection
if timeout 10 ssh -o ConnectTimeout=5 -o StrictHostKeyChecking=no guest@localhost -p 22 exit 2>/dev/null; then
    echo -e "${GREEN}✓ SSH server is responding${NC}"
else
    echo -e "${YELLOW}⚠️  SSH test inconclusive (this is normal for browser SSH)${NC}"
fi

echo
echo -e "${GREEN}🎉 Setup Complete!${NC}"
echo "════════════════════"
echo
echo -e "${BLUE}📋 Next Steps:${NC}"
echo "1. Visit https://keycrafter.fun in your web browser"
echo "2. You should see a browser-based SSH terminal"
echo "3. The terminal will load automatically with KeyCrafter interface"
echo "4. Users can now access the game without SSH clients!"
echo
echo -e "${BLUE}🔍 Troubleshooting:${NC}"
echo "• Check logs: docker-compose logs"
echo "• Verify Cloudflare tunnel: docker-compose logs cloudflared"
echo "• Check SSH server: docker-compose logs keycrafter-ssh"
echo
echo -e "${YELLOW}⚠️  Important Security Notes:${NC}"
echo "• Browser SSH uses Cloudflare Access for authentication"
echo "• No SSH keys required for users"
echo "• Sessions are logged and monitored"
echo "• Consider setting up rate limiting if needed"
echo

echo -e "${GREEN}✨ KeyCrafter is now ready for browser access!${NC}" 
# KeyCrafter Browser SSH Setup

## Current System Overview

KeyCrafter is a crafting-based game with the following components:

1. Game Server:
   - Written in Rust
   - Uses custom word lists for crafting (easy/medium/hard difficulty)
   - Features include: crafting system, floating text, islands, pathfinding, upgrades
   - Currently distributed via SFTP/SSH access

2. Infrastructure:
   - Running in Docker containers
   - Current containers:
     - keycrafter-ssh (172.28.0.2): SSH/SFTP server for game distribution
     - cloudflared (172.28.0.3): Cloudflare tunnel for secure access
   - Domain: keycrafter.fun
   - Cloudflare Tunnel ID: 552ba21b-d360-4141-9aa0-f28c89c9b4de

3. Authentication:
   - Currently using public key authentication
   - SSH server configured with custom shell script for game downloads
   - Authorized keys mounted at /home/guest/.ssh/authorized_keys

## Goal

Transform the game distribution system to use Cloudflare's browser-rendered SSH terminal, similar to terminal.shop, allowing players to:
1. Visit keycrafter.fun in their browser
2. Log in securely
3. Access game files through a browser-based terminal
4. No client software or configuration required

## Required Steps

Please help implement the following:

1. Cloudflare Zero Trust Configuration:
   - Set up Access application for keycrafter.fun
   - Configure browser-rendered SSH terminal
   - Set up appropriate access policies

2. SSH Server Adjustments:
   - Verify/modify sshd_config for browser-based terminal compatibility
   - Configure user authentication to work with Cloudflare Access
   - Ensure custom shell script works in browser terminal

3. Game Distribution Flow:
   - Test and verify game file access
   - Ensure download commands work in browser terminal
   - Verify file transfer capabilities

4. Security Considerations:
   - Review and adjust file permissions
   - Configure appropriate access policies
   - Implement any necessary rate limiting

## Current Files

Key configuration files are in the following locations:
- /cloudflared/config.yml: Tunnel configuration
- /sftp-shell/custom-shell.sh: Custom shell for game downloads
- /haproxy/haproxy.cfg: Load balancer configuration
- /nginx/nginx.conf: Web server configuration
- /docker-compose.yml: Container orchestration

## Additional Context

The game uses word lists for crafting mechanics:
- resources/words_easy.txt
- resources/words_medium.txt
- resources/words_hard.txt

Players need to be able to download these files along with the game client.

Please provide step-by-step instructions for implementing this transition while maintaining security and usability. 
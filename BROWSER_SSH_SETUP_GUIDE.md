# KeyCrafter Browser SSH Setup Guide

This guide walks you through transforming KeyCrafter from traditional SSH access to browser-rendered SSH terminal using Cloudflare Zero Trust.

## 🎯 Overview

After following this guide, users will be able to:
- Visit `keycrafter.fun` in any web browser
- Access a terminal interface directly in the browser
- Download and play KeyCrafter without any SSH client software
- Authenticate seamlessly through Cloudflare Access

## 📋 Prerequisites

Before starting, ensure you have:

- ✅ Cloudflare account with Zero Trust enabled
- ✅ Domain `keycrafter.fun` added to Cloudflare
- ✅ Existing Cloudflare tunnel (ID: `552ba21b-d360-4141-9aa0-f28c89c9b4de`)
- ✅ Docker and Docker Compose installed
- ✅ Access to Cloudflare Dashboard

## 🔧 Step-by-Step Implementation

### Step 1: Configure Cloudflare Zero Trust Access Application

1. **Navigate to Cloudflare Zero Trust Dashboard**
   - Go to [https://one.dash.cloudflare.com/](https://one.dash.cloudflare.com/)
   - Select your account

2. **Create SSH Access Application**
   - Go to `Access` → `Applications`
   - Click `Add an application`
   - Select `SSH` as the application type

3. **Configure Application Settings**
   ```
   Application name: KeyCrafter SSH Terminal
   Session duration: 24h
   Domain: keycrafter.fun
   Type: SSH
   ```

4. **Enable Browser Rendering**
   - Check ✅ `Browser rendering`
   - This is crucial for the web-based terminal

5. **Set Access Policy**
   ```
   Policy name: Allow All Users
   Action: Allow
   Include: Everyone (or configure specific rules as needed)
   ```

6. **Save the Application**

### Step 2: Update Cloudflare Tunnel Configuration

The tunnel configuration has been updated in `cloudflared/config.yml`:

```yaml
tunnel: 552ba21b-d360-4141-9aa0-f28c89c9b4de
credentials-file: /etc/cloudflared/552ba21b-d360-4141-9aa0-f28c89c9b4de.json
protocol: quic
logLevel: debug

ingress:
  - hostname: keycrafter.fun
    service: ssh://keycrafter-ssh:22
    originRequest:
      noTLSVerify: true
      connectTimeout: "30s"
      tcpKeepAlive: "30s"
      # Enable browser rendering
      access:
        required: true
        teamName: "your-team-name"  # Replace with your Zero Trust team name
        audTag: "keycrafter-ssh"
      
  - service: http_status:404

warp-routing:
  enabled: true
  
# Browser SSH configuration
browser-rendering:
  enabled: true
```

**Important:** Replace `your-team-name` with your actual Cloudflare Zero Trust team name.

### Step 3: Rebuild and Deploy

1. **Stop existing containers:**
   ```bash
   docker-compose down
   ```

2. **Rebuild with new configuration:**
   ```bash
   docker-compose build --no-cache
   ```

3. **Start services:**
   ```bash
   docker-compose up -d
   ```

4. **Check service status:**
   ```bash
   docker-compose ps
   docker-compose logs
   ```

### Step 4: Test Browser SSH Access

1. **Wait for services to initialize** (give it 2-3 minutes)

2. **Visit your domain:**
   - Open `https://keycrafter.fun` in your web browser
   - You should see a browser-based terminal loading

3. **Verify the experience:**
   - Terminal should display the KeyCrafter ASCII art
   - Interactive menu should appear: `[ about ] [ download ] [ exit ]`
   - Download functionality should work for game files

## 🎮 User Experience

### What Users Will See

1. **Landing Page**: Browser terminal loads automatically at `keycrafter.fun`
2. **Welcome Screen**: KeyCrafter ASCII art with "press any key to start"
3. **Interactive Menu**: Three options to explore the game
4. **Download Instructions**: Clear guidance for getting the game binary

### Browser Compatibility

- ✅ Chrome/Chromium
- ✅ Firefox
- ✅ Safari
- ✅ Edge
- ✅ Mobile browsers (with limitations)

## 🔒 Security Considerations

### Authentication Flow
1. User visits `keycrafter.fun`
2. Cloudflare Access checks authentication
3. If authenticated, browser SSH terminal loads
4. User interacts with KeyCrafter interface

### Security Features
- No SSH keys required for users
- All sessions go through Cloudflare Access
- Automatic session timeout (24h default)
- Audit logging through Cloudflare
- Rate limiting available through Cloudflare

### Recommended Security Policies

For production, consider these Access policies:

```json
{
  "name": "Rate Limited Access",
  "decision": "allow",
  "include": [{"everyone": {}}],
  "session_duration": "4h",
  "require": [
    {
      "geo": {"countries": ["US", "CA", "GB"]}
    }
  ]
}
```

## 🐛 Troubleshooting

### Common Issues

1. **Browser terminal not loading**
   - Check Cloudflare Access application is properly configured
   - Verify tunnel is running: `docker-compose logs cloudflared`
   - Ensure domain DNS points to Cloudflare

2. **SSH connection refused**
   - Check SSH container: `docker-compose logs keycrafter-ssh`
   - Verify port 22 is accessible within Docker network
   - Check SSH configuration in Dockerfile

3. **Authentication loops**
   - Verify Access policy allows your user/location
   - Check Cloudflare Access audit logs
   - Ensure browser accepts cookies

4. **Terminal display issues**
   - Modern browser required (Chrome 60+, Firefox 60+)
   - JavaScript must be enabled
   - Check browser console for errors

### Debug Commands

```bash
# Check all services
docker-compose ps

# View logs
docker-compose logs -f

# Test SSH locally
ssh -o ConnectTimeout=5 guest@localhost -p 22

# Check tunnel status
docker-compose exec cloudflared cloudflared tunnel info

# Rebuild if needed
docker-compose down && docker-compose build --no-cache && docker-compose up -d
```

## 📊 Monitoring and Analytics

### Available Metrics
- Session duration and frequency (Cloudflare Access)
- User geography and device types
- SSH connection attempts and success rates
- Game download statistics

### Log Locations
- Cloudflare Access: Dashboard → Analytics → Access
- SSH Server: `docker-compose logs keycrafter-ssh`
- Tunnel: `docker-compose logs cloudflared`

## 🚀 Optional Enhancements

### Additional Features You Can Add

1. **Custom Landing Page**
   - Add HTTP service alongside SSH
   - Provide game information and links

2. **Download Statistics**
   - Track game downloads
   - Monitor popular access times

3. **Enhanced Security**
   - IP-based restrictions
   - Device certificates
   - Multi-factor authentication

4. **Game Launcher**
   - Direct game execution in browser
   - WebAssembly port of the game

## 📞 Support

If you encounter issues:

1. Check this troubleshooting section
2. Review Cloudflare Access documentation
3. Verify all configuration files match the examples
4. Test with different browsers/devices

## 🎉 Success Criteria

You'll know the setup is successful when:

- ✅ `keycrafter.fun` loads a terminal in the browser
- ✅ Users can access without SSH clients
- ✅ Game download works through browser terminal
- ✅ Authentication is handled by Cloudflare Access
- ✅ No technical setup required for end users

---

**🎮 Congratulations! KeyCrafter is now accessible to anyone with a web browser!** 
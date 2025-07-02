FROM cloudflare/cloudflared:latest

# Install basic debugging tools
USER root
RUN apk add --no-cache \
    netcat-openbsd \
    curl \
    iputils \
    net-tools

# Switch back to non-root user
USER nonroot 
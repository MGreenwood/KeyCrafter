FROM nginx:alpine

# Create downloads directory
RUN mkdir -p /usr/share/nginx/downloads

# Copy game binaries
COPY target/release/keycrafter.exe /usr/share/nginx/downloads/keycrafter-windows-x64.exe
COPY target/release/keycrafter /usr/share/nginx/downloads/keycrafter-linux-x64

# Copy install scripts
COPY scripts/install.ps1 /usr/share/nginx/downloads/install.ps1
COPY scripts/install.sh /usr/share/nginx/downloads/install.sh

# Copy website files
COPY web/images /usr/share/nginx/html/images
COPY web/index.html /usr/share/nginx/html/
COPY web/style.css /usr/share/nginx/html/
COPY web/script.js /usr/share/nginx/html/

# Copy nginx configuration
COPY nginx/nginx.conf /etc/nginx/nginx.conf

EXPOSE 80 
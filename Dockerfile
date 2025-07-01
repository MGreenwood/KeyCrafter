FROM ubuntu:22.04

# Install OpenSSH Server and utilities
RUN apt-get update && apt-get install -y openssh-server && \
    mkdir /var/run/sshd && \
    # Create download user with no login shell but SFTP access
    useradd -m -d /home/download -s /usr/sbin/nologin download && \
    # Create directory for the binary
    mkdir -p /home/download/bin && \
    chown -R download:download /home/download

# Configure SSH security settings
RUN echo '\n\
# Disable root login\n\
PermitRootLogin no\n\
\n\
# Use modern secure protocols\n\
Protocol 2\n\
\n\
# Only use strong ciphers and algorithms\n\
Ciphers chacha20-poly1305@openssh.com,aes256-gcm@openssh.com,aes128-gcm@openssh.com,aes256-ctr,aes192-ctr,aes128-ctr\n\
MACs hmac-sha2-512-etm@openssh.com,hmac-sha2-256-etm@openssh.com,umac-128-etm@openssh.com\n\
KexAlgorithms curve25519-sha256@libssh.org,diffie-hellman-group16-sha512,diffie-hellman-group18-sha512,diffie-hellman-group14-sha256\n\
\n\
# Strict mode for host keys\n\
HostKeyAlgorithms ssh-ed25519,rsa-sha2-512,rsa-sha2-256\n\
\n\
# Logging\n\
SyslogFacility AUTH\n\
LogLevel VERBOSE\n\
\n\
# Authentication settings\n\
LoginGraceTime 30\n\
MaxAuthTries 3\n\
MaxSessions 2\n\
\n\
# No password authentication\n\
PasswordAuthentication no\n\
PermitEmptyPasswords no\n\
ChallengeResponseAuthentication no\n\
\n\
# No forwarding\n\
AllowAgentForwarding no\n\
AllowTcpForwarding no\n\
X11Forwarding no\n\
PermitTunnel no\n\
\n\
# No user environment customization\n\
PermitUserEnvironment no\n\
\n\
# Restrict SFTP user\n\
Match User download\n\
    ChrootDirectory /home/download\n\
    ForceCommand internal-sftp\n\
    AllowTcpForwarding no\n\
    X11Forwarding no\n\
    PasswordAuthentication no\n\
' > /etc/ssh/sshd_config

# Copy the binary
COPY target/release/keycrafter.exe /home/download/bin/
RUN chown download:download /home/download/bin/keycrafter.exe && \
    chmod 644 /home/download/bin/keycrafter.exe

# Generate host keys with strong algorithms
RUN ssh-keygen -t ed25519 -f /etc/ssh/ssh_host_ed25519_key -N "" && \
    ssh-keygen -t rsa -b 4096 -f /etc/ssh/ssh_host_rsa_key -N ""

# Create .ssh directory for the download user with secure permissions
RUN mkdir -p /home/download/.ssh && \
    chown download:download /home/download/.ssh && \
    chmod 700 /home/download/.ssh

# Add fail2ban for brute force protection
RUN apt-get install -y fail2ban && \
    echo '\n\
[sshd]\n\
enabled = true\n\
port = 22\n\
filter = sshd\n\
logpath = /var/log/auth.log\n\
maxretry = 3\n\
findtime = 300\n\
bantime = 3600\n\
' > /etc/fail2ban/jail.local

# Expose SSH port
EXPOSE 22

# Start SSH server and fail2ban
CMD service fail2ban start && /usr/sbin/sshd -D -e 
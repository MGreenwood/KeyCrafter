#!/bin/bash

# Function to generate a key with specific comment and expiry
generate_key() {
    local name=$1
    local days=$2
    local comment="keycrafter-download-key-$(date +%Y%m%d)"
    
    # Generate ed25519 key with specific comment
    ssh-keygen -t ed25519 -f "$name" -N "" -C "$comment"
    
    # Calculate expiry date
    local expiry=$(date -d "+$days days" +%Y-%m-%d)
    echo "# Valid until: $expiry" >> "$name.pub"
    
    echo "Generated key $name (valid until $expiry)"
}

# Function to validate existing keys
check_keys() {
    local key=$1
    if [ -f "$key" ]; then
        # Extract expiry date from public key
        local expiry=$(grep "Valid until:" "$key.pub" | cut -d' ' -f4)
        if [ ! -z "$expiry" ]; then
            local today=$(date +%Y-%m-%d)
            if [[ "$today" > "$expiry" ]]; then
                echo "Warning: Key $key has expired ($expiry)"
                return 1
            else
                echo "Key $key is valid until $expiry"
                return 0
            fi
        fi
    fi
    return 1
}

# Directory for key management
KEY_DIR="ssh_keys"
mkdir -p "$KEY_DIR"
cd "$KEY_DIR"

# Check if we need to generate new keys
if ! check_keys "keycrafter_download"; then
    echo "Generating new download key..."
    generate_key "keycrafter_download" 30  # Valid for 30 days
fi

# Create authorized_keys file with secure permissions
mkdir -p ../authorized_keys
cat keycrafter_download.pub > ../authorized_keys/authorized_keys
chmod 600 ../authorized_keys/authorized_keys

# Print instructions
echo ""
echo "SSH key setup complete!"
echo "Public key has been added to authorized_keys/"
echo ""
echo "To download the binary using SFTP:"
echo "sftp -i ssh_keys/keycrafter_download -o StrictHostKeyChecking=yes -P 2222 download@<server>"
echo "get bin/keycrafter.exe"
echo ""
echo "Security recommendations:"
echo "1. Keep the private key secure and never share it"
echo "2. Use StrictHostKeyChecking=yes to prevent MITM attacks"
echo "3. Keys expire after 30 days for security"
echo "4. Run this script again to generate new keys when needed"
echo "5. Delete old keys after rotation" 
#!/bin/bash

# Terminal control sequences
ESC=$'\e'
CLEAR="${ESC}[2J${ESC}[H"
RESET="${ESC}[0m"
BOLD="${ESC}[1m"
DIM="${ESC}[2m"
GREY="${ESC}[90m"
GREEN="${ESC}[32m"
YELLOW="${ESC}[33m"

# Configuration
CONFIG_DIR="/home/guest/sftp-shell"
WELCOME_FILE="$CONFIG_DIR/welcome.txt"
ABOUT_FILE="$CONFIG_DIR/about.txt"
ART_CONFIG="$CONFIG_DIR/art.json"
BINARY_PATH="/home/guest/downloads/keycrafter.exe"

# Function to get terminal size (browser-compatible)
get_terminal_size() {
    # Try to get terminal size, fallback to defaults for browser terminals
    if command -v stty >/dev/null 2>&1; then
        read -r LINES COLUMNS < <(stty size 2>/dev/null || echo "24 80")
    else
        LINES=${LINES:-24}
        COLUMNS=${COLUMNS:-80}
    fi
    
    # Ensure minimum reasonable values for browser terminals
    [ "$LINES" -lt 10 ] && LINES=24
    [ "$COLUMNS" -lt 40 ] && COLUMNS=80
}

# Function to center text horizontally
center_text() {
    local text="$1"
    local text_length=${#text}
    local padding=$(( (COLUMNS - text_length) / 2 ))
    printf "%${padding}s%s%${padding}s\n" "" "$text" ""
}

# Function to center text vertically and horizontally
center_block() {
    local file="$1"
    local lines=$(wc -l < "$file")
    local padding=$(( (LINES - lines) / 2 ))
    
    # Print top padding
    for ((i=0; i<padding; i++)); do
        echo
    done
    
    # Print centered content
    while IFS= read -r line; do
        center_text "$line"
    done < "$file"
}

# Function to render ASCII art from config
render_decorations() {
    local art_data=$(cat "$ART_CONFIG")
    # In a real implementation, we'd parse the JSON and render the art
    # For now, we'll just show some basic decorations
    echo "${GREEN} /\\ ${RESET}"
    echo "${GREEN}/~~\\${RESET}"
    echo "${GREEN} || ${RESET}"
}

# Function to animate text appearance
animate_text() {
    local text="$1"
    local delay=0.1
    for ((i=0; i<${#text}; i++)); do
        printf "%s" "${text:$i:1}"
        sleep $delay
    done
    echo
}

# Function to show press any key message
show_press_key() {
    local message="press any key to start"
    local pos_y=$((LINES - 3))
    echo "${ESC}[${pos_y};0H"
    center_text "${GREY}${message}${RESET}"
}

# Function to handle download (browser-compatible)
initiate_download() {
    if [ -f "$BINARY_PATH" ]; then
        clear
        echo "🎮 KeyCrafter Game Download"
        echo "═══════════════════════════════════════"
        echo
        echo "File: keycrafter.exe"
        echo "Size: $(stat -c %s "$BINARY_PATH" 2>/dev/null || stat -f %z "$BINARY_PATH" 2>/dev/null || echo "unknown") bytes"
        echo
        echo "📋 Download Instructions:"
        echo
        echo "For Browser Terminal Users:"
        echo "1. Right-click and 'Save As' to download"
        echo "2. Or use: wget/curl if available"
        echo
        echo "For Traditional SSH Users:"
        echo "1. Use SFTP: sftp guest@keycrafter.fun"
        echo "2. Run: get downloads/keycrafter.exe"
        echo
        echo "🔧 Alternative Download Methods:"
        echo "• Visit: https://keycrafter.fun/download (if configured)"
        echo "• Use SCP: scp guest@keycrafter.fun:downloads/keycrafter.exe ."
        echo
        echo "📖 After Download:"
        echo "1. Make executable: chmod +x keycrafter.exe"
        echo "2. Run: ./keycrafter.exe"
        echo
        read -p "Press enter to continue..."
    else
        echo "❌ Error: Binary not found!"
        echo "Please contact support if this persists."
        read -p "Press enter to continue..."
    fi
}

# Function to show menu
show_menu() {
    clear
    echo
    center_text "Choose an option:"
    echo
    center_text "[ about ] [ download ] [ exit ]"
    echo
    center_text "Type your choice: "
}

# Function to handle main menu
handle_menu() {
    while true; do
        show_menu
        read -r choice
        case "$choice" in
            "about")
                clear
                cat "$ABOUT_FILE"
                read -n 1 -r
                ;;
            "download")
                clear
                initiate_download
                ;;
            "exit")
                clear
                center_text "Thanks for visiting KeyCrafter!"
                sleep 1
                exit 0
                ;;
            *)
                center_text "Invalid option. Try again."
                sleep 1
                ;;
        esac
    done
}

# Initialize terminal
init_terminal() {
    # Get terminal dimensions
    get_terminal_size
    
    # Set up terminal for optimal display
    stty -echo 2>/dev/null || true
    tput clear 2>/dev/null || clear
}

# Main execution
init_terminal

center_block "$WELCOME_FILE"
show_press_key

# Wait for keypress
read -n 1 -s

# Show menu
handle_menu 
# BuildAndDeployLinux.ps1
# Builds the Linux binary using Docker and deploys it to the nginx container for download

param(
    [string]$NginxContainer = "keycrafter-keycrafter-web-1"
)

$ErrorActionPreference = "Stop"

# Step 1: Build the Docker image
Write-Host "[1/5] Building Docker image..."
docker build -f build-linux.dockerfile -t keycrafter-linux .

# Step 2: Create a temporary container from the build image
Write-Host "[2/5] Creating temporary build container..."
docker create --name temp-keycrafter keycrafter-linux

# Step 3: Copy the compiled binary from the temp container to host
$outputDir = "./output"
if (!(Test-Path $outputDir)) { New-Item -ItemType Directory -Path $outputDir | Out-Null }
Write-Host "[3/5] Copying binary from build container to host..."
docker cp temp-keycrafter:/usr/src/keycrafter/target/release/keycrafter $outputDir/keycrafter

# Step 4: Remove the temporary container
Write-Host "[4/5] Removing temporary build container..."
docker rm temp-keycrafter

# Step 5: Copy the binary into the nginx container
Write-Host "[5/5] Copying binary into nginx container ($NginxContainer) as keycrafter-linux-x64..."
docker cp $outputDir/keycrafter ${NginxContainer}:/usr/share/nginx/downloads/keycrafter-linux-x64

Write-Host "Done! The Linux binary is now deployed to nginx and available for download."

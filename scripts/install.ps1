# Create install directory
$installDir = "$env:LOCALAPPDATA\KeyCrafter"

# Check for existing installation
if (Test-Path "$installDir\keycrafter.exe") {
    Write-Host "Found existing KeyCrafter installation. Updating..."
    Stop-Process -Name "keycrafter" -ErrorAction SilentlyContinue
    Remove-Item "$installDir\keycrafter.exe" -Force
}

New-Item -ItemType Directory -Force -Path $installDir | Out-Null

Write-Host "Downloading KeyCrafter..."
Invoke-WebRequest -Uri "https://play.keycrafter.fun/keycrafter-windows-x64.exe" -OutFile "$installDir\keycrafter.exe"

# Add to PATH if not already there
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$installDir*") {
    [Environment]::SetEnvironmentVariable(
        "Path",
        "$userPath;$installDir",
        "User"
    )
    Write-Host "Added KeyCrafter to your PATH"
}

Write-Host @"
KeyCrafter installed successfully!
You can now run 'keycrafter' from any terminal.
Note: You may need to restart your terminal for the PATH changes to take effect.
"@ 
# KeyCrafter Update Diagnostic Script

Write-Host "==== KeyCrafter PATH Resolution ===="
$pathExe = $(where keycrafter.exe 2>$null)
if ($pathExe) {
    Write-Host "keycrafter.exe found in PATH at: $pathExe"
    Get-Command keycrafter.exe | Format-List | Out-String | Write-Host
} else {
    Write-Host "keycrafter.exe not found in PATH."
}

Write-Host "\n==== Installed KeyCrafter (AppData) ===="
$installed = "$env:LOCALAPPDATA\KeyCrafter\keycrafter.exe"
if (Test-Path $installed) {
    $info = Get-Item $installed
    Write-Host "Path: $installed"
    Write-Host "Size: $($info.Length) bytes"
    Write-Host "LastWriteTime: $($info.LastWriteTime)"
    $installedHash = Get-FileHash $installed -Algorithm SHA256 | Select-Object -ExpandProperty Hash
    Write-Host "SHA256: $installedHash"
} else {
    Write-Host "Not found: $installed"
}

Write-Host "\n==== Project Build Output (target/release) ===="
$build = "$(Resolve-Path ./target/release/keycrafter.exe -ErrorAction SilentlyContinue)"
if ($build -and (Test-Path $build)) {
    $info = Get-Item $build
    Write-Host "Path: $build"
    Write-Host "Size: $($info.Length) bytes"
    Write-Host "LastWriteTime: $($info.LastWriteTime)"
    $buildHash = Get-FileHash $build -Algorithm SHA256 | Select-Object -ExpandProperty Hash
    Write-Host "SHA256: $buildHash"
} else {
    Write-Host "Not found: target/release/keycrafter.exe"
}

Write-Host "\n==== Downloaded Server Binary ===="
$temp = [System.IO.Path]::GetTempFileName()
try {
    Invoke-WebRequest -Uri "https://play.keycrafter.fun/keycrafter-windows-x64.exe" -OutFile $temp -UseBasicParsing
    $info = Get-Item $temp
    Write-Host "Downloaded to: $temp"
    Write-Host "Size: $($info.Length) bytes"
    Write-Host "LastWriteTime: $($info.LastWriteTime)"
    $serverHash = Get-FileHash $temp -Algorithm SHA256 | Select-Object -ExpandProperty Hash
    Write-Host "SHA256: $serverHash"
} catch {
    Write-Host "Failed to download server binary. $_"
    $serverHash = $null
}

Write-Host "\n==== Version Endpoint ===="
try {
    $versionJson = Invoke-WebRequest -Uri "https://play.keycrafter.fun/version" -UseBasicParsing | Select-Object -ExpandProperty Content
    Write-Host $versionJson
} catch {
    Write-Host "Failed to fetch version endpoint. $_"
}

Write-Host "\n==== Hash Comparison ===="
if ($installedHash -and $buildHash) {
    if ($installedHash -eq $buildHash) {
        Write-Host "[OK] Installed and build output are identical."
    } else {
        Write-Host "[DIFF] Installed and build output are DIFFERENT."
    }
}
if ($installedHash -and $serverHash) {
    if ($installedHash -eq $serverHash) {
        Write-Host "[OK] Installed and server binary are identical."
    } else {
        Write-Host "[DIFF] Installed and server binary are DIFFERENT."
    }
}
if ($buildHash -and $serverHash) {
    if ($buildHash -eq $serverHash) {
        Write-Host "[OK] Build output and server binary are identical."
    } else {
        Write-Host "[DIFF] Build output and server binary are DIFFERENT."
    }
}

# Clean up temp file
if (Test-Path $temp) { Remove-Item $temp -Force }

Write-Host "\n==== Done ====" 
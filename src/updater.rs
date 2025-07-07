use reqwest;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::thread;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const UPDATE_CHECK_URL: &str = "https://play.keycrafter.fun/version";
const DOWNLOAD_URL: &str = "https://play.keycrafter.fun/keycrafter-windows-x64.exe";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VersionInfo {
    pub version: String,
    pub required: bool,
    pub changes: Vec<String>,
    pub download_url: String,
}

pub struct Updater {
    client: reqwest::blocking::Client,
    last_check: std::time::SystemTime,
    check_interval: std::time::Duration,
}

impl Updater {
    pub fn new() -> Self {
        Self {
            client: reqwest::blocking::Client::new(),
            last_check: std::time::SystemTime::now(),
            check_interval: std::time::Duration::from_secs(3600), // Check every hour
        }
    }

    pub fn should_check_update(&self) -> bool {
        self.last_check.elapsed().unwrap_or_default() >= self.check_interval
    }

    pub fn check_for_updates(&self) -> Result<Option<VersionInfo>, Box<dyn Error>> {
        let response = self.client.get(UPDATE_CHECK_URL)
            .header("User-Agent", format!("KeyCrafter/{}", CURRENT_VERSION))
            .send()?;

        if !response.status().is_success() {
            return Ok(None);
        }

        let version_info: VersionInfo = response.json()?;
        
        // Compare versions
        if version_info.version != CURRENT_VERSION {
            Ok(Some(version_info))
        } else {
            Ok(None)
        }
    }

    pub fn download_update(&self, version_info: &VersionInfo) -> Result<String, Box<dyn Error>> {
        // Create downloads directory if it doesn't exist
        let downloads_dir = Path::new("downloads");
        fs::create_dir_all(downloads_dir)?;

        // Download the new version
        let response = self.client.get(&version_info.download_url)
            .header("User-Agent", format!("KeyCrafter/{}", CURRENT_VERSION))
            .send()?;

        if !response.status().is_success() {
            return Err("Failed to download update".into());
        }

        let filename = format!("keycrafter-{}.exe", version_info.version);
        let filepath = downloads_dir.join(&filename);
        
        // Save the downloaded file
        let bytes = response.bytes()?;
        fs::write(&filepath, bytes)?;

        Ok(filepath.to_string_lossy().into_owned())
    }

    pub fn apply_update(&self, new_exe_path: &str) -> Result<(), Box<dyn Error>> {
        // Get the current executable path
        let current_exe = std::env::current_exe()?;
        let current_exe_path = current_exe.to_string_lossy();
        
        // Create a batch file to:
        // 1. Wait for our process to exit
        // 2. Copy the new exe over the old one
        // 3. Delete the downloaded file
        // 4. Start the new version
        let batch_content = format!(
            r#"@echo off
echo Waiting for KeyCrafter to exit...
:wait_loop
timeout /t 1 /nobreak >nul
tasklist /FI "IMAGENAME eq keycrafter.exe" /FO CSV | find "keycrafter.exe" >nul
if not errorlevel 1 (
    echo Process still running, waiting...
    goto wait_loop
)
echo Process exited, copying new version...
copy /y "{}" "{}"
if errorlevel 1 (
    echo Failed to copy new version
    pause
    exit /b 1
)
echo Deleting downloaded file...
del "{}"
echo Starting new version...
start "" "{}"
echo Update completed!
timeout /t 2 /nobreak >nul
del "%~f0"
"#,
            new_exe_path, current_exe_path, new_exe_path, current_exe_path
        );

        let batch_path = "update.bat";
        fs::write(batch_path, batch_content)?;

        // Start the batch file
        Command::new("cmd")
            .args(&["/C", "start", "/min", "", batch_path])
            .spawn()?;

        Ok(())
    }

    pub fn get_update_message(&self, version_info: &VersionInfo) -> String {
        let mut message = format!("New version {} available!\n", version_info.version);
        if version_info.required {
            message.push_str("This update is required.\n");
        }
        message.push_str("\nChanges:\n");
        for change in &version_info.changes {
            message.push_str(&format!("- {}\n", change));
        }
        message.push_str("\nPress 'u' to update now.");
        message
    }

    pub fn self_update() -> Result<(), Box<dyn Error>> {
        // Create a detached process that will continue after we exit
        #[cfg(target_os = "windows")]
        {
            // Get the current executable path
            let current_exe = std::env::current_exe()?;
            let current_exe_path = current_exe.to_string_lossy();
            
            // On Windows, use a batch file that waits for us to exit
            let batch_content = format!(
                r#"@echo off
echo Starting KeyCrafter update...
echo Waiting for current process to exit...
:wait_loop
timeout /t 1 /nobreak >nul
tasklist /FI "IMAGENAME eq keycrafter.exe" /FO CSV | find "keycrafter.exe" >nul
if not errorlevel 1 (
    echo Process still running, waiting...
    goto wait_loop
)
echo Process exited, downloading update...
powershell -Command "irm play.keycrafter.fun/install.ps1 | iex"
echo Update completed!
timeout /t 3 /nobreak >nul
del "%~f0"
"#);
            
            let batch_path = "update_self.bat";
            fs::write(batch_path, batch_content)?;
            
            // Start the batch file in a completely detached process
            Command::new("cmd")
                .args(&["/C", "start", "/min", "/B", "", batch_path])
                .spawn()?;
            
            // Give the batch file a moment to start properly
            std::thread::sleep(std::time::Duration::from_millis(1000));
        }

        #[cfg(not(target_os = "windows"))]
        {
            // On Unix systems, create a script that runs in background
            let script_content = r#"#!/bin/bash
echo "Starting KeyCrafter update..."
sleep 2
curl -fsSL play.keycrafter.fun/install.sh | bash
echo "Update completed!"
"#;
            
            let script_path = "update_self.sh";
            fs::write(script_path, script_content)?;
            
            // Make the script executable
            #[cfg(unix)]
            {
                let mut perms = fs::metadata(script_path)?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(script_path, perms)?;
            }
            
            // Start the script in background and detach it
            Command::new("bash")
                .args(&["-c", &format!("nohup {} > /dev/null 2>&1 &", script_path)])
                .spawn()?;
        }

        // Exit immediately - the detached process will continue
        std::process::exit(0);
    }
} 
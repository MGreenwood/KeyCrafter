use std::fs;
use std::path::Path;
use std::process::Command;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Version {
    major: u32,
    minor: u32,
    patch: u32,
}

impl Version {
    fn from_string(version: &str) -> Option<Self> {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() >= 3 {
            Some(Version {
                major: parts[0].parse().unwrap_or(0),
                minor: parts[1].parse().unwrap_or(0),
                patch: parts[2].parse().unwrap_or(0),
            })
        } else {
            None
        }
    }

    fn to_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }

    fn increment_patch(&mut self) {
        self.patch += 1;
    }
}

fn main() {
    // Read current version from Cargo.toml
    let cargo_toml = fs::read_to_string("Cargo.toml").unwrap();
    let version_line = cargo_toml.lines()
        .find(|line| line.trim().starts_with("version"))
        .unwrap_or("version = \"0.1.0\"");
    
    let current_version = version_line
        .split('=')
        .nth(1)
        .unwrap_or("\"0.1.0\"")
        .trim()
        .trim_matches('"');

    // Parse and increment version
    let mut version = Version::from_string(current_version)
        .unwrap_or(Version { major: 0, minor: 1, patch: 0 });
    
    // Increment patch number
    version.increment_patch();
    
    // Update Cargo.toml
    let new_version = version.to_string();
    let new_cargo_toml = cargo_toml.replace(
        &format!("version = \"{}\"", current_version),
        &format!("version = \"{}\"", new_version)
    );
    fs::write("Cargo.toml", new_cargo_toml).unwrap();

    // Update nginx version endpoint
    let nginx_conf_path = "nginx/nginx.conf";
    if Path::new(nginx_conf_path).exists() {
        let nginx_conf = fs::read_to_string(nginx_conf_path).unwrap();
        
        // Find the version JSON block and update it
        let version_json_start = nginx_conf.find("\"version\":").unwrap_or(0);
        let version_json_end = nginx_conf[version_json_start..].find(",").unwrap_or(0);
        
        if version_json_start > 0 && version_json_end > 0 {
            let old_version_json = &nginx_conf[version_json_start..version_json_start + version_json_end];
            let new_version_json = format!("\"version\": \"{}\"", new_version);
            
            let new_nginx_conf = nginx_conf.replace(old_version_json, &new_version_json);
            fs::write(nginx_conf_path, new_nginx_conf).unwrap();
        }
    }

    // Generate version info for the build
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rustc-env=CARGO_PKG_VERSION={}", new_version);

    // If this is a release build, create a version commit
    if std::env::var("PROFILE").unwrap_or_default() == "release" {
        Command::new("git")
            .args(&["add", "Cargo.toml", "nginx/nginx.conf"])
            .status()
            .unwrap();

        Command::new("git")
            .args(&["commit", "-m", &format!("Version bump to {}", new_version)])
            .status()
            .unwrap();
    }
} 
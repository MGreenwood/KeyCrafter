#!/usr/bin/env node

const https = require('https');
const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

// Determine platform and architecture
const platform = process.platform;
const arch = process.arch;

// Map to our naming scheme
const platformMap = {
  'win32': 'windows',
  'darwin': 'darwin',
  'linux': 'linux'
};

const archMap = {
  'x64': 'x64',
  'arm64': 'arm64'
};

const exeExt = platform === 'win32' ? '.exe' : '';
const fileName = `keycrafter-${platformMap[platform]}-${archMap[arch]}${exeExt}`;
const binPath = path.join(__dirname, '..', 'bin');
const targetPath = path.join(binPath, 'keycrafter' + exeExt);

// Create bin directory if it doesn't exist
if (!fs.existsSync(binPath)) {
  fs.mkdirSync(binPath, { recursive: true });
}

// Download the appropriate binary
console.log(`Downloading KeyCrafter for ${platform} ${arch}...`);

const file = fs.createWriteStream(targetPath);
const request = https.get(`https://keycrafter.fun/downloads/${fileName}`, (response) => {
  response.pipe(file);
});

// Handle completion
file.on('finish', () => {
  file.close();
  console.log('Download completed');
  
  // Make executable on Unix platforms
  if (platform !== 'win32') {
    try {
      fs.chmodSync(targetPath, '755');
    } catch (err) {
      console.error('Failed to make file executable:', err);
    }
  }
  
  console.log('KeyCrafter installed successfully!');
  console.log('You can now run it by typing: keycrafter');
}); 
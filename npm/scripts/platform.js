const os = require('os');
const fs = require('fs');
const path = require('path');

const VERSION = require('../package.json').version;
const REPO = 'tuhucar/cli';

function detectMusl() {
  try {
    // Check for musl-specific files
    const files = fs.readdirSync('/lib');
    if (files.some(f => f.startsWith('ld-musl'))) return true;
  } catch {}
  try {
    const { execSync } = require('child_process');
    const output = execSync('ldd --version 2>&1', { encoding: 'utf8' });
    if (output.toLowerCase().includes('musl')) return true;
  } catch {}
  return false;
}

function getArtifactName() {
  const platform = os.platform();
  const arch = os.arch();

  const map = {
    'darwin-arm64': 'tuhucar-darwin-arm64',
    'darwin-x64': 'tuhucar-darwin-x64',
    'linux-x64': detectMusl() ? 'tuhucar-linux-x64-musl' : 'tuhucar-linux-x64',
    'linux-arm64': detectMusl() ? 'tuhucar-linux-arm64-musl' : 'tuhucar-linux-arm64',
    'win32-x64': 'tuhucar-win32-x64.exe',
    'win32-arm64': 'tuhucar-win32-arm64.exe',
  };

  const key = `${platform}-${arch}`;
  const artifact = map[key];

  if (!artifact) {
    throw new Error(
      `Unsupported platform: ${platform}-${arch}. ` +
      `Please install from source or use install.sh`
    );
  }

  return artifact;
}

function getDownloadUrl() {
  const artifact = getArtifactName();
  return `https://github.com/${REPO}/releases/download/v${VERSION}/${artifact}`;
}

function getChecksumUrl() {
  const artifact = getArtifactName();
  return `https://github.com/${REPO}/releases/download/v${VERSION}/${artifact}.sha256`;
}

function getBinaryName() {
  return os.platform() === 'win32' ? 'tuhucar.exe' : 'tuhucar';
}

function getBinaryPath() {
  return path.join(__dirname, '..', 'bin', getBinaryName());
}

module.exports = {
  getArtifactName,
  getDownloadUrl,
  getChecksumUrl,
  getBinaryName,
  getBinaryPath,
  VERSION,
};

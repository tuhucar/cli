const https = require('https');
const http = require('http');
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const { execFileSync } = require('child_process');
const {
  getArtifactName,
  getDownloadUrl,
  getChecksumUrl,
  getBinaryPath,
  VERSION,
} = require('./platform');

function download(url) {
  return new Promise((resolve, reject) => {
    const client = url.startsWith('https') ? https : http;
    client.get(url, { headers: { 'User-Agent': 'tuhucar-npm' } }, (res) => {
      // Handle redirects
      if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
        return download(res.headers.location).then(resolve).catch(reject);
      }
      if (res.statusCode !== 200) {
        return reject(new Error(`Download failed from ${url}: HTTP ${res.statusCode}`));
      }
      const chunks = [];
      res.on('data', (chunk) => chunks.push(chunk));
      res.on('end', () => resolve(Buffer.concat(chunks)));
      res.on('error', reject);
    }).on('error', reject);
  });
}

async function main() {
  if (process.env.TUHUCAR_SKIP_POSTINSTALL === '1') {
    console.log('Skipping tuhucar postinstall because TUHUCAR_SKIP_POSTINSTALL=1.');
    return;
  }

  const artifact = getArtifactName();
  const binaryPath = getBinaryPath();
  const binDir = path.dirname(binaryPath);

  // Ensure bin directory exists
  fs.mkdirSync(binDir, { recursive: true });

  console.log(`Downloading tuhucar ${VERSION} (${artifact})...`);

  try {
    // Download binary
    const binaryUrl = getDownloadUrl();
    const binary = await download(binaryUrl);

    // Download and verify checksum
    try {
      const checksumUrl = getChecksumUrl();
      const checksumData = await download(checksumUrl);
      const expectedHash = checksumData.toString('utf8').trim().split(/\s+/)[0];
      const actualHash = crypto.createHash('sha256').update(binary).digest('hex');

      if (actualHash !== expectedHash) {
        throw new Error(
          `Checksum mismatch!\nExpected: ${expectedHash}\nActual:   ${actualHash}`
        );
      }
      console.log('Checksum verified.');
    } catch (e) {
      if (e.message.includes('Checksum mismatch')) throw e;
      console.warn('Warning: Could not verify checksum:', e.message);
    }

    // Write binary
    fs.writeFileSync(binaryPath, binary);
    fs.chmodSync(binaryPath, 0o755);
    console.log(`Installed tuhucar to ${binaryPath}`);

    // Best-effort skill install
    try {
      execFileSync(binaryPath, ['skill', 'install'], { stdio: 'inherit' });
    } catch {
      console.log('Note: Skill installation skipped. Run `tuhucar skill install` later.');
    }

  } catch (err) {
    if (fs.existsSync(binaryPath)) {
      fs.rmSync(binaryPath, { force: true });
    }
    console.error(`Failed to install tuhucar: ${err.message}`);
    console.error(
      'Make sure the matching GitHub release assets exist, or reinstall later with: npm install -g @tuhucar/cli'
    );
    process.exit(1);
  }
}

main();

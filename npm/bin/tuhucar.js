#!/usr/bin/env node

const { execFileSync } = require('child_process');
const { getBinaryPath } = require('../scripts/platform');

const binaryPath = getBinaryPath();

try {
  execFileSync(binaryPath, process.argv.slice(2), { stdio: 'inherit' });
} catch (err) {
  if (err.status !== undefined) {
    process.exit(err.status);
  }
  console.error(`Failed to run tuhucar: ${err.message}`);
  console.error('Try reinstalling: npm install -g @tuhucar/cli');
  process.exit(1);
}

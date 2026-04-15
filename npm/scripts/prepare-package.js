const fs = require('fs');
const path = require('path');

const packageDir = path.resolve(__dirname, '..');
const repoRoot = path.resolve(packageDir, '..');
const filesToCopy = ['README.md', 'LICENSE'];

for (const filename of filesToCopy) {
  const source = path.join(repoRoot, filename);
  const destination = path.join(packageDir, filename);
  fs.copyFileSync(source, destination);
  console.log(`Prepared ${filename} for npm package.`);
}

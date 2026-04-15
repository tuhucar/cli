const fs = require('fs');
const path = require('path');

const packageDir = path.resolve(__dirname, '..');
const generatedFiles = ['README.md', 'LICENSE'];

for (const filename of generatedFiles) {
  const target = path.join(packageDir, filename);
  if (fs.existsSync(target)) {
    fs.unlinkSync(target);
    console.log(`Removed generated ${filename}.`);
  }
}

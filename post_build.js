// post-build.js
const fs = require('fs');
const path = require('path');
const glob = require('glob');

// Source paths
const srcIndexJS = path.resolve(__dirname, 'index.js');
const srcIndexDTS = path.resolve(__dirname, 'index.d.ts');

// Destination paths
const destDirectory = path.resolve(__dirname, 'driver_api', 'lib');
const destIndexJS = path.join(destDirectory, 'index.js');
const destIndexDTS = path.join(destDirectory, 'index.d.ts');

// Create the directory if it doesn't exist
if (!fs.existsSync(destDirectory)){
    fs.mkdirSync(destDirectory);
}

// Move the files
fs.renameSync(srcIndexJS, destIndexJS);
fs.renameSync(srcIndexDTS, destIndexDTS);

// Copy any .node files
glob("**/*.node", function (err, files) {
    if (err) {
        console.error("Error finding .node files:", err);
    } else {
        files.forEach(function (file) {
            const src = path.resolve(__dirname, file);
            const dest = path.join(destDirectory, path.basename(file));
            fs.renameSync(src, dest);
        });
    }
});

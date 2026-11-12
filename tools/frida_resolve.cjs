// Node preload: maps the Frida module path that the capture tools were written with (the original capture computer's
// npx cache) to this computer's install, so the hash-pinned tools run unchanged. Target: $FRIDA_NODE, else
// ~/.local/share/frida-node/node_modules/frida. Loaded through NODE_OPTIONS by tools/gurun.sh (docs/handoff/README.md §4).
const Module = require('module'), path = require('path'), os = require('os'), fs = require('fs');
const LEGACY = '/home/sds/.npm/_npx/da7caeae9db9c589/node_modules/frida';
const target = process.env.FRIDA_NODE || path.join(os.homedir(), '.local/share/frida-node/node_modules/frida');
const resolve = Module._resolveFilename;
Module._resolveFilename = function (request, ...rest) {
  if (request === LEGACY || request.startsWith(LEGACY + '/')) {
    if (!fs.existsSync(target)) throw Error(`Frida 없음: ${target} (FRIDA_NODE 또는 ~/.local/share/frida-node에 설치)`);
    request = target + request.slice(LEGACY.length);
  }
  return resolve.call(this, request, ...rest);
};

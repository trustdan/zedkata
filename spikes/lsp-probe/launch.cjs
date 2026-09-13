// Zed's working directory is not necessarily the repository directory.
const { existsSync } = require('node:fs');
const { join } = require('node:path');

const probe = join(__dirname, 'dist', 'probe.js');
if (!existsSync(probe)) {
  process.stderr.write('Probe is not built. Run npm.cmd ci and npm.cmd run build in spikes/lsp-probe.\n');
  process.exit(1);
}
process.env.ZEDKATA_PROBE_LOG ??= join(
  __dirname, 'captures', `zed-${Date.now()}-${process.pid}.jsonl`,
);
require(probe);

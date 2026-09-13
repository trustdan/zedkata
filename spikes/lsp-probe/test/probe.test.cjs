const assert = require('node:assert/strict');
const { spawn } = require('node:child_process');
const { once } = require('node:events');
const { mkdirSync, mkdtempSync, readFileSync } = require('node:fs');
const { resolve, join } = require('node:path');
const { test } = require('node:test');

test('stdio lifecycle preserves complete messages and emits diagnostics', { timeout: 10000 }, async (t) => {
  mkdirSync('test-output', { recursive: true });
  const directory = mkdtempSync(resolve('test-output', 'synthetic-'));
  const capturePath = join(directory, 'capture.jsonl');
  const child = spawn(process.execPath, [resolve('launch.cjs'), '--stdio'], {
    cwd: directory,
    env: { ...process.env, ZEDKATA_PROBE_LOG: capturePath },
    stdio: ['pipe', 'pipe', 'pipe'],
  });
  t.after(() => { if (child.exitCode === null) child.kill(); });
  const closed = once(child, 'close');
  let stderr = '';
  child.stderr.on('data', (chunk) => { stderr += chunk; });
  let buffer = Buffer.alloc(0);
  let parseError;
  const received = [];
  const waiters = [];
  function dispatch() {
    for (let i = waiters.length - 1; i >= 0; i--) {
      const waiter = waiters[i];
      const message = received.find(waiter.predicate);
      if (parseError || message) {
        waiters.splice(i, 1);
        clearTimeout(waiter.timer);
        if (parseError) waiter.reject(parseError); else waiter.resolve(message);
      }
    }
  }
  child.stdout.on('data', (chunk) => {
    buffer = Buffer.concat([buffer, chunk]);
    try {
      while (true) {
        const end = buffer.indexOf('\r\n\r\n');
        if (end < 0) break;
        const header = buffer.subarray(0, end).toString('ascii');
        assert.match(header, /^Content-Length: \d+$/i);
        const length = Number(header.split(':')[1]);
        if (buffer.length < end + 4 + length) break;
        received.push(JSON.parse(buffer.subarray(end + 4, end + 4 + length).toString('utf8')));
        buffer = buffer.subarray(end + 4 + length);
      }
    } catch (error) { parseError = error; }
    dispatch();
  });
  function waitFor(predicate) {
    return new Promise((resolve, reject) => {
      const waiter = { predicate, resolve, reject, timer: setTimeout(() => {
        waiters.splice(waiters.indexOf(waiter), 1);
        reject(new Error(`Timed out waiting for protocol output. stderr: ${stderr}`));
      }, 3000) };
      waiters.push(waiter);
      dispatch();
    });
  }
  const sent = [];
  function send(message, split = false) {
    sent.push(message);
    const body = Buffer.from(JSON.stringify(message));
    const frame = Buffer.concat([Buffer.from(`Content-Length: ${body.length}\r\n\r\n`), body]);
    if (split) { child.stdin.write(frame.subarray(0, 9)); child.stdin.write(frame.subarray(9)); }
    else child.stdin.write(frame);
  }
  const uri = 'file:///C:/probe%20fixtures/example.kata';
  send({ jsonrpc: '2.0', id: 1, method: 'initialize', params: {
    processId: null, rootUri: null, capabilities: { general: { positionEncodings: ['utf-8', 'utf-16'] } },
    clientInfo: { name: 'synthetic-client', version: '1' },
  } }, true);
  const init = await waitFor((m) => m.id === 1);
  assert.equal(init.result.capabilities.textDocumentSync.change, 2);
  assert.equal(init.result.capabilities.positionEncoding, 'utf-16');
  send({ jsonrpc: '2.0', method: 'initialized', params: {} });
  send({ jsonrpc: '2.0', method: 'textDocument/didOpen', params: {
    textDocument: { uri, languageId: 'zedkata', version: 1, text: 'hello 🌍\r\nsecond line\r\n' },
  } });
  await waitFor((m) => m.method === 'textDocument/publishDiagnostics' && m.params.version === 1);
  send({ jsonrpc: '2.0', method: 'textDocument/didChange', params: {
    textDocument: { uri, version: 2 }, contentChanges: [
      { range: { start: { line: 0, character: 6 }, end: { line: 0, character: 8 } }, rangeLength: 2, text: 'é' },
      { range: { start: { line: 1, character: 0 }, end: { line: 1, character: 6 } }, text: '' },
    ],
  } });
  const changed = await waitFor((m) => m.method === 'textDocument/publishDiagnostics' && m.params.version === 2);
  assert.match(changed.params.diagnostics[0].message, /Probe change 1/);
  // Even a full replacement must be recorded unchanged, not rejected or inferred.
  send({ jsonrpc: '2.0', method: 'textDocument/didChange', params: {
    textDocument: { uri, version: 3 }, contentChanges: [{ text: 'replacement\r\n' }],
  } });
  await waitFor((m) => m.method === 'textDocument/publishDiagnostics' && m.params.version === 3);
  send({ jsonrpc: '2.0', method: 'textDocument/didClose', params: { textDocument: { uri } } });
  await waitFor((m) => m.method === 'textDocument/publishDiagnostics' && m.params.diagnostics.length === 0);
  send({ jsonrpc: '2.0', id: 2, method: 'shutdown', params: null });
  assert.equal((await waitFor((m) => m.id === 2)).result, null);
  send({ jsonrpc: '2.0', method: 'exit' });
  const [code] = await closed;
  assert.equal(code, 0, stderr);
  assert.ifError(parseError);
  assert.equal(buffer.length, 0, 'stdout must contain only complete protocol frames');
  const capture = readFileSync(capturePath, 'utf8').trim().split('\n').map(JSON.parse);
  assert.deepEqual(capture.filter((e) => e.direction === 'in').map((e) => e.message), sent);
  assert.deepEqual(capture.filter((e) => e.direction === 'out').map((e) => e.message), received);
  capture.forEach((entry, i) => {
    assert.equal(entry.sequence, i + 1);
    assert.ok(Number.isFinite(Date.parse(entry.at)));
    assert.ok(BigInt(entry.monotonicNs) > 0n);
    if (i) assert.ok(BigInt(entry.monotonicNs) >= BigInt(capture[i - 1].monotonicNs));
  });
  t.diagnostic(`Synthetic capture (not Zed evidence): ${capturePath}`);
});

test('unwritable capture destination fails without protocol stdout', { timeout: 5000 }, async () => {
  const child = spawn(process.execPath, ['dist/probe.js', '--stdio'], {
    env: { ...process.env, ZEDKATA_PROBE_LOG: resolve('package.json', 'capture.jsonl') },
  });
  let stdout = '', stderr = '';
  child.stdout.on('data', (chunk) => { stdout += chunk; });
  child.stderr.on('data', (chunk) => { stderr += chunk; });
  const [code] = await once(child, 'close');
  assert.equal(code, 1);
  assert.equal(stdout, '');
  assert.match(stderr, /Cannot open probe capture/);
});

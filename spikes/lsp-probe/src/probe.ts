import { appendFileSync, mkdirSync, openSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import {
  createConnection, DiagnosticSeverity, PositionEncodingKind,
  StreamMessageReader, StreamMessageWriter, TextDocumentSyncKind,
} from 'vscode-languageserver/node';

// Disposable observer: preserve messages without a document manager merging edits.
const capturePath = resolve(process.env.ZEDKATA_PROBE_LOG ??
  `captures/probe-${Date.now()}-${process.pid}.jsonl`);
let capture: number;
try {
  mkdirSync(dirname(capturePath), { recursive: true });
  capture = openSync(capturePath, 'ax');
} catch (error) {
  process.stderr.write(`Cannot open probe capture ${capturePath}: ${String(error)}\n`);
  process.exit(1);
}
let sequence = 0;
function record(direction: string, message: unknown): void {
  const entry = { sequence: ++sequence, at: new Date().toISOString(),
    monotonicNs: process.hrtime.bigint().toString(), direction, message };
  try {
    appendFileSync(capture, JSON.stringify(entry) + '\n');
  } catch (error) {
    process.stderr.write(`Cannot write probe capture: ${String(error)}\n`);
    process.exit(1);
  }
}
record('meta', { pid: process.pid, node: process.version, capturePath });
process.stderr.write(`Probe capture: ${capturePath}\n`);

const reader = new StreamMessageReader(process.stdin);
const writer = new StreamMessageWriter(process.stdout);
const connection = createConnection({
  onError: reader.onError, onClose: reader.onClose,
  onPartialMessage: reader.onPartialMessage,
  listen: (callback) => reader.listen((message) => {
    record('in', message);
    callback(message);
  }),
  dispose: () => reader.dispose(),
}, {
  onError: writer.onError, onClose: writer.onClose,
  write: (message) => { record('out', message); return writer.write(message); },
  end: () => writer.end(),
  dispose: () => writer.dispose(),
});

connection.onInitialize(() => ({
  capabilities: {
    positionEncoding: PositionEncodingKind.UTF16,
    textDocumentSync: { openClose: true, change: TextDocumentSyncKind.Incremental },
  },
  serverInfo: { name: 'zedkata-lsp-probe', version: '0.0.0' },
}));
let changeCount = 0;
function feedback(uri: string, version: number, label: string): void {
  void connection.sendDiagnostics({ uri, version, diagnostics: [{
    range: { start: { line: 0, character: 0 }, end: { line: 0, character: 0 } },
    severity: DiagnosticSeverity.Information,
    source: 'zedkata-probe',
    message: `Probe ${label} at ${new Date().toISOString()}`,
  }] });
}
connection.onDidOpenTextDocument(({ textDocument }) =>
  feedback(textDocument.uri, textDocument.version, 'opened'));
connection.onDidChangeTextDocument(({ textDocument }) =>
  feedback(textDocument.uri, textDocument.version, `change ${++changeCount}`));
connection.onDidCloseTextDocument(({ textDocument }) => {
  void connection.sendDiagnostics({ uri: textDocument.uri, diagnostics: [] });
});
connection.onShutdown(() => undefined);
connection.listen();

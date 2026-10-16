#!/usr/bin/env python3
"""
Lightweight mock ACP agent script for ZedKata walkthrough demonstration.

Demonstrates:
1. Stdout framing hygiene: ONLY framed JSON-RPC 2.0 messages are written to stdout.
2. Stderr logging: All debugging and diagnostic info is written exclusively to stderr.
3. Bidirectional stdio communication matching Zed's Agent Client Protocol.
"""

import sys
import json

def log_debug(msg: str) -> None:
    sys.stderr.write(f"[mock-acp-agent] {msg}\n")
    sys.stderr.flush()

def send_response(response: dict) -> None:
    payload = json.dumps(response)
    # Write response to stdout followed by newline (or Content-Length framing if negotiated)
    sys.stdout.write(payload + "\n")
    sys.stdout.flush()

def main() -> None:
    log_debug("Agent started via stdio. Listening for JSON-RPC requests...")

    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue

        try:
            request = json.loads(line)
        except json.JSONDecodeError as e:
            log_debug(f"JSON decode error: {e}")
            continue

        req_id = request.get("id")
        method = request.get("method")
        log_debug(f"Received method: {method} (id={req_id})")

        if method == "initialize":
            send_response({
                "jsonrpc": "2.0",
                "id": req_id,
                "result": {
                    "protocolVersion": "0.1.0",
                    "agent": {
                        "name": "ZedKata Mock Agent",
                        "version": "0.1.0"
                    },
                    "capabilities": {
                        "prompts": True,
                        "tools": True
                    }
                }
            })
        elif method == "ping":
            send_response({
                "jsonrpc": "2.0",
                "id": req_id,
                "result": "pong"
            })
        elif req_id is not None:
            # General fallback response
            send_response({
                "jsonrpc": "2.0",
                "id": req_id,
                "result": {
                    "status": "ok",
                    "message": f"Acknowledged {method}"
                }
            })

    log_debug("Agent shutting down.")

if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""Starts `qed mcp <site>` the way an AI assistant does and checks the answers.

    python3 dev/mcp-smoke.py <program> <site folder>

<program> is q.e.d. itself: an installed copy (C:\\...\\qed.exe, the
.AppImage, /Applications/qed.app/Contents/MacOS/qed) or a build
(target/debug/qed). It only reads: the opening handshake, the list of
tools and `site_info`. See "Testing the MCP server" in
docs/pages/development.md.
"""

import json
import queue
import subprocess
import sys
import threading

TIMEOUT = 30  # seconds per answer; the first start of a fresh install is slow


def main() -> int:
    if len(sys.argv) != 3:
        print(__doc__, file=sys.stderr)
        return 2
    program, site = sys.argv[1], sys.argv[2]

    # Pipes for all three, as Claude Desktop and Claude Code do it. On
    # Windows this is the part to test: release builds are window programs
    # without a console, and must still answer over these pipes.
    server = subprocess.Popen(
        [program, "mcp", site],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    lines: queue.Queue = queue.Queue()
    threading.Thread(target=lambda: [lines.put(l) for l in server.stdout], daemon=True).start()
    errors: list = []
    threading.Thread(target=lambda: errors.extend(server.stderr), daemon=True).start()

    def send(message: dict) -> None:
        server.stdin.write(json.dumps(message).encode() + b"\n")
        server.stdin.flush()

    def answer(id: int) -> dict:
        while True:
            try:
                line = lines.get(timeout=TIMEOUT)
            except queue.Empty:
                fail(f"no answer to request {id} within {TIMEOUT} s")
            message = json.loads(line)
            if message.get("id") == id:
                if "error" in message:
                    fail(f"request {id}: {message['error']}")
                return message["result"]

    def fail(why: str) -> None:
        server.kill()
        print(f"FAILED: {why}", file=sys.stderr)
        if errors:
            print("qed wrote:", b"".join(errors).decode(errors="replace"), file=sys.stderr)
        sys.exit(1)

    send({
        "jsonrpc": "2.0", "id": 1, "method": "initialize",
        "params": {
            "protocolVersion": "2025-06-18",
            "capabilities": {},
            "clientInfo": {"name": "mcp-smoke", "version": "1"},
        },
    })
    info = answer(1)["serverInfo"]
    print(f"server: {info['name']} {info['version']}")
    send({"jsonrpc": "2.0", "method": "notifications/initialized"})

    send({"jsonrpc": "2.0", "id": 2, "method": "tools/list"})
    tools = [tool["name"] for tool in answer(2)["tools"]]
    print(f"tools: {len(tools)} ({', '.join(tools)})")
    if "site_info" not in tools:
        fail("site_info is missing from the tools")

    send({
        "jsonrpc": "2.0", "id": 3, "method": "tools/call",
        "params": {"name": "site_info", "arguments": {}},
    })
    result = answer(3)
    if result.get("isError"):
        fail(f"site_info: {result['content']}")
    print("site_info:", result["content"][0]["text"].splitlines()[0])

    # Closing stdin is how an assistant says goodbye: the server must end.
    server.stdin.close()
    try:
        server.wait(timeout=TIMEOUT)
    except subprocess.TimeoutExpired:
        fail("still running after stdin was closed")
    print("OK")
    return 0


if __name__ == "__main__":
    sys.exit(main())

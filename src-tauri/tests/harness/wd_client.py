#!/usr/bin/env python3
# NA-0701 GUI-driver WebDriver client — the probe's wd_client.py EVOLVED (R112 /
# D636 §3.1), not rewritten: python3 STDLIB ONLY.
# Imports enumerated: sys, os, json, base64, time, http.client. No pip, no venv.
# Evolution over the probe client (D635): adds `elements` (find-elements COUNT —
# the A1.4 uniqueness instrument), `execute` (POST /session/{id}/execute/sync —
# the F-F substitute channel), `attr`/`clear` (element attribute read / input
# clear for the settings flow), and keeps the machine-readable per-call
# transcript (WD_TRANSCRIPT, one json.dumps line per HTTP call).
# Timeouts, all stated in-source (A1.5): default 30s per call (WD_TIMEOUT env);
# create-session states its own timeout as argv[3] — the runner passes 45.
# Exit codes: 0 = HTTP 200 (and PNG magic OK for screenshot); 2 = WebDriver
# error response (body printed verbatim); 3 = transport/timeout error;
# 4 = usage. Never retries silently.
import sys, os, json, base64, time, http.client

HOST = os.environ.get("WD_HOST", "127.0.0.1")
PORT = int(os.environ.get("WD_PORT", "4444"))
TIMEOUT = float(os.environ.get("WD_TIMEOUT", "30"))
TRANSCRIPT = os.environ.get("WD_TRANSCRIPT")
SESSION_FILE = os.environ.get("WD_SESSION_FILE")
ELEMENT_KEY = "element-6066-11e4-a52e-4f735466cecf"  # W3C element identifier


def log(entry):
    if TRANSCRIPT:
        with open(TRANSCRIPT, "a") as f:
            f.write(json.dumps(entry) + "\n")


def call(method, path, body=None, timeout=None):
    t = TIMEOUT if timeout is None else timeout
    payload = None if body is None else json.dumps(body)
    entry = {"utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
             "timeout_s": t,
             "request": {"method": method, "path": path, "body": body}}
    try:
        conn = http.client.HTTPConnection(HOST, PORT, timeout=t)
        headers = {"Content-Type": "application/json"} if payload is not None else {}
        conn.request(method, path, body=payload, headers=headers)
        resp = conn.getresponse()
        raw = resp.read().decode("utf-8", "replace")
        entry["response"] = {"status": resp.status, "body": raw}
        log(entry)
        conn.close()
        return resp.status, raw
    except Exception as e:
        entry["transport_error"] = repr(e)
        log(entry)
        print(json.dumps({"transport_error": repr(e)}))
        sys.exit(3)


def sid():
    with open(SESSION_FILE) as f:
        return f.read().strip()


def main():
    if len(sys.argv) < 2:
        print("usage: wd_client.py CMD [args]", file=sys.stderr)
        sys.exit(4)
    cmd = sys.argv[1]
    if cmd == "status":
        st, raw = call("GET", "/status")
        print(raw)
        sys.exit(0 if st == 200 else 2)
    elif cmd == "create-session":
        app = sys.argv[2]
        t = float(sys.argv[3])
        caps = {"capabilities": {"alwaysMatch": {"tauri:options": {"application": app}}}}
        st, raw = call("POST", "/session", caps, timeout=t)
        if st == 200:
            with open(SESSION_FILE, "w") as f:
                f.write(json.loads(raw)["value"]["sessionId"])
            print(raw)
            sys.exit(0)
        print(raw)
        sys.exit(2)
    elif cmd == "delete-session":
        st, raw = call("DELETE", "/session/" + sid())
        print(raw)
        sys.exit(0 if st == 200 else 2)
    elif cmd == "find":
        st, raw = call("POST", "/session/%s/element" % sid(),
                       {"using": sys.argv[2], "value": sys.argv[3]})
        if st == 200:
            print(json.loads(raw)["value"][ELEMENT_KEY])
            sys.exit(0)
        print(raw)
        sys.exit(2)
    elif cmd == "elements":
        st, raw = call("POST", "/session/%s/elements" % sid(),
                       {"using": sys.argv[2], "value": sys.argv[3]})
        if st == 200:
            print(len(json.loads(raw)["value"]))
            sys.exit(0)
        print(raw)
        sys.exit(2)
    elif cmd == "text":
        st, raw = call("GET", "/session/%s/element/%s/text" % (sid(), sys.argv[2]))
        if st == 200:
            sys.stdout.write(json.loads(raw)["value"])  # exact bytes, no added newline
            sys.exit(0)
        print(raw)
        sys.exit(2)
    elif cmd == "click":
        st, raw = call("POST", "/session/%s/element/%s/click" % (sid(), sys.argv[2]), {})
        print(raw)
        sys.exit(0 if st == 200 else 2)
    elif cmd == "sendkeys":
        st, raw = call("POST", "/session/%s/element/%s/value" % (sid(), sys.argv[2]),
                       {"text": sys.argv[3]})
        print(raw)
        sys.exit(0 if st == 200 else 2)
    elif cmd == "clear":
        st, raw = call("POST", "/session/%s/element/%s/clear" % (sid(), sys.argv[2]), {})
        print(raw)
        sys.exit(0 if st == 200 else 2)
    elif cmd == "prop":
        st, raw = call("GET", "/session/%s/element/%s/property/%s" % (sid(), sys.argv[2], sys.argv[3]))
        if st == 200:
            print(json.dumps(json.loads(raw)["value"]))
            sys.exit(0)
        print(raw)
        sys.exit(2)
    elif cmd == "attr":
        st, raw = call("GET", "/session/%s/element/%s/attribute/%s" % (sid(), sys.argv[2], sys.argv[3]))
        if st == 200:
            print(json.dumps(json.loads(raw)["value"]))
            sys.exit(0)
        print(raw)
        sys.exit(2)
    elif cmd == "execute":
        st, raw = call("POST", "/session/%s/execute/sync" % sid(),
                       {"script": sys.argv[2], "args": []})
        print(raw)
        sys.exit(0 if st == 200 else 2)
    elif cmd == "screenshot":
        st, raw = call("GET", "/session/%s/screenshot" % sid())
        if st == 200:
            data = base64.b64decode(json.loads(raw)["value"])
            with open(sys.argv[2], "wb") as f:
                f.write(data)
            magic_ok = data[:8] == b"\x89PNG\r\n\x1a\n"
            print(json.dumps({"png_magic_ok": magic_ok, "bytes": len(data)}))
            sys.exit(0 if magic_ok else 2)
        print(raw)
        sys.exit(2)
    else:
        print("unknown command: " + cmd, file=sys.stderr)
        sys.exit(4)


if __name__ == "__main__":
    main()

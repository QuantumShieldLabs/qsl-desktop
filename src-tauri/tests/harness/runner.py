#!/usr/bin/env python3
# NA-0701 GUI-driver scenario runner (D636 base + A1 + A2; A2 > A1 > base).
# python3 STDLIB ONLY. Imports enumerated: sys, os, json, time, socket, signal,
# hashlib, subprocess, shutil, pathlib.
#
# DUTIES (numbered, per D636 §3.1 as amended):
#  1. THE RUN ROOT (A2.1): env QSLD_GUI_RUN_ROOT; DEFAULT =
#     <repo-root>/target/gui_driver_runs/<utc>/ (repo-relative, created if
#     absent; gitignored by the existing /target rule). The shared cargo target
#     cache is NEVER a data or evidence home (A2.2) — only the app BINARY is
#     read from CARGO_TARGET_DIR.
#  2. THE STANDING ISOLATION BRACKET (A1.1): EVERY invocation writes the
#     real-$HOME candidate-dir census pre and post (the probe P7 instrument),
#     byte-identical REQUIRED; the runner REFUSES destructive scenario steps
#     if the pre-census is absent (refusal exit 3).
#  3. Port selection (A1.13): TWO INDEPENDENT port-0 probes, both passed
#     explicitly as --port/--native-port; bounded bind-failure retry, 3
#     attempts, each recorded.
#  4. Launch: xvfb-run -a -s "-screen 0 1280x800x24" -> dbus-run-session ->
#     tauri-driver --port P --native-port P+? ; GDK_BACKEND=x11; the E3 env
#     set RE-DERIVED per launch from the run profile (A1.15); the harness sets
#     NEITHER automation env (R164 §3 — tauri-driver injects both); driver/app
#     stdout+stderr captured from launch.
#  5. Multi-launch scenarios against the same profile (F-C and F-E, A1.11).
#  6. Teardown on EVERY exit path (A1.15): DELETE /session, then SIGTERM the
#     MEASURED pgid (start_new_session makes the child the session leader, so
#     the pgid is measured from the live process — the probe's corrected
#     instrument), verified by comm-name pgrep census, all bounded polls.
#  7. Per-step machine-readable verdicts: json.dumps JSONL rows
#     {"step","expected","measured","verdict","evidence"} + one terminal row
#     {"scenario","result","steps"} (the R169-2.2 committed format).
#  8. Evidence (R164 §5): verdict log FROZEN first, run log frozen next,
#     MANIFEST.json written LAST with sha256 per artifact (excluding only its
#     own row).
#  9. Liveness controls every run (P9 pair): absent selector must yield
#     `no such element`; a deliberately wrong expected text must miscompare.
# 10. NO sleeps-and-hope: every wait is a bounded 1s poll with its count
#     recorded in the verdict row.
# 11. THE GATE RULE (NA-0778, RULING_NA0778_015 R96 -- the f_k lesson, CI run
#     33940568140): an arm that asserts a PREDICATE controls EVERY input of that
#     predicate in the SAME synchronous step and returns every term, so a red names
#     its term. An arm that forces part of a gate's inputs and leaves the rest to
#     the app's own async refresh is a race: the box's timing is not the runner's
#     (the box could not reproduce the red; the runner's log was the arm).
import sys, os, json, time, socket, signal, hashlib, subprocess, shutil
from pathlib import Path

HARNESS = Path(__file__).resolve().parent
REPO_ROOT = HARNESS.parents[2]
CLIENT = str(HARNESS / "wd_client.py")
APP_ID_DIR = "org.quantumshieldlabs.qsldesktop"
POLL_MAX = 30
COUNTDOWN_POLL_MAX = 45


def utcstamp():
    return time.strftime("%Y%m%dT%H%M%SZ", time.gmtime())


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()


def probe_two_ports(attempts_log, max_attempts=3):
    for attempt in range(1, max_attempts + 1):
        s1 = socket.socket(); s2 = socket.socket()
        s1.bind(("127.0.0.1", 0)); s2.bind(("127.0.0.1", 0))
        p1 = s1.getsockname()[1]; p2 = s2.getsockname()[1]
        s1.close(); s2.close()
        ok = []
        for p in (p1, p2):
            s = socket.socket()
            try:
                s.bind(("127.0.0.1", p)); s.close(); ok.append(True)
            except OSError:
                ok.append(False)
        attempts_log.append({"attempt": attempt, "port": p1, "native_port": p2, "bind_ok": ok})
        if all(ok) and p1 != p2:
            return p1, p2
    return None, None


class Refusal(SystemExit):
    pass


class StepFailure(Exception):
    pass


class Runner:
    def __init__(self, scenario_path):
        self.scenario = json.load(open(scenario_path))
        self.name = self.scenario["scenario"]
        run_root_env = os.environ.get("QSLD_GUI_RUN_ROOT")
        base = Path(run_root_env) if run_root_env else REPO_ROOT / "target" / "gui_driver_runs" / utcstamp()
        self.run = base / self.name
        self.run.mkdir(parents=True, exist_ok=True)
        self.profile = self.run / "profile"
        for d in ("config", "data", "cache", "state", "qsld-data"):
            (self.profile / d).mkdir(parents=True, exist_ok=True)
        self.verdict_path = self.run / "verdict.jsonl"
        self.verdict_f = open(self.verdict_path, "w")
        self.log_path = self.run / "run_log.txt"
        self.log_f = open(self.log_path, "w")
        self.steps = 0
        self.fails = 0
        self.proc = None
        self.pgid = None
        self.launches = 0
        self.session_live = False
        self.wd_env = dict(os.environ)
        self.driver = os.environ.get("QSLD_TAURI_DRIVER") or shutil.which("tauri-driver")
        # perturbation-measurement mode (SR-06/R169-2.3): record every row's
        # verdict instead of aborting at the first red, so a perturbed run
        # yields its FULL red row-set. Launch/session rows stay hard.
        self.continue_on_fail = os.environ.get("QSLD_CONTINUE_ON_FAIL") == "1"
        target = os.environ.get("CARGO_TARGET_DIR") or str(REPO_ROOT / "target")
        self.binary = os.path.join(target, "debug", "qsl-desktop")
        # NA-0779 (D-0048, kickoff L4): THE HARNESS CAPTURE. Per launch, the debug log is turned
        # on (detailed) through the app's OWN command the first time scr-main is seen -- never by
        # a pre-launch settings.json (that file is the S1/S2 launch-state discriminator, state.rs)
        # and never by an environment variable (the switch is a setting by ruling) -- and
        # exported into the run dir at teardown, beside verdict.jsonl, where the manifest freezes
        # it with its sha. The export's footer digest is verified here, independently.
        self.dl_armed = False
        self.dl_captures = 0

    def note(self, msg):
        self.log_f.write(msg + "\n")
        self.log_f.flush()

    def invoke_wait(self, cmd, args, slot, polls=20):
        """Fire a Tauri command from the webview and poll its settled result (execute/sync
        cannot await a promise). Returns (ok, value_or_error) or (None, reason) on a driver fault."""
        script = ("window.%s=null; window.__TAURI__.core.invoke(%s, %s)"
                  ".then(function(v){window.%s={ok:true,v:v};})"
                  ".catch(function(e){window.%s={ok:false,e:String(e)};}); return 'fired';"
                  % (slot, json.dumps(cmd), json.dumps(args), slot, slot))
        rc, out = self.wd("execute", script)
        if rc != 0:
            return None, "execute rc=%d %s" % (rc, out.strip()[:80])
        for _ in range(polls):
            rc, out = self.wd("execute", "return window.%s ? JSON.stringify(window.%s) : null;" % (slot, slot))
            if rc == 0:
                try:
                    v = json.loads(out)["value"]
                except (ValueError, KeyError):
                    v = None
                if v:
                    r = json.loads(v)
                    return (r.get("ok"), r.get("v") if r.get("ok") else r.get("e"))
            time.sleep(0.5)
        return None, "no settled result after %d polls" % polls

    def debug_log_arm(self):
        if self.dl_armed or not self.session_live:
            return
        ok1, v1 = self.invoke_wait("debug_log_control", {"action": "on"}, "__qsld_dl_arm1")
        ok2, v2 = self.invoke_wait("debug_log_control", {"action": "level_detailed"}, "__qsld_dl_arm2")
        armed = bool(ok1) and bool(ok2) and isinstance(v2, dict) and v2.get("on") is True and v2.get("level") == "detailed"
        self.dl_armed = armed
        self.step("debug_log_armed", "on + detailed through debug_log_control",
                  "on=%s level=%s" % (v2.get("on") if isinstance(v2, dict) else v1, v2.get("level") if isinstance(v2, dict) else v2), armed)

    def debug_log_capture(self, n):
        if not self.dl_armed or not self.session_live:
            return
        self.dl_armed = False
        ok, v = self.invoke_wait("debug_log_export", {"dir": str(self.run)}, "__qsld_dl_export")
        measured = "rc=%s" % ok
        good = False
        if ok and isinstance(v, dict) and v.get("path"):
            path = Path(v["path"])
            if path.exists() and path.parent == self.run:
                raw = path.read_bytes()
                i = raw.rfind(b"# sha256=")
                body, footer = raw[:i], raw[i:].strip()
                digest = hashlib.sha256(body).hexdigest().encode()
                good = footer == b"# sha256=" + digest and hashlib.sha256(raw).hexdigest() == v.get("sha256")
                self.dl_captures += 1
                measured = "file=%s bytes=%d footer_ok=%s" % (path.name, len(raw), good)
                evidence = [path.name]
            else:
                measured = "path=%r not in the run dir" % v.get("path")
                evidence = None
        else:
            measured = "export failed: %r" % (v,)
            evidence = None
        self.step("debug_log_capture_launch%d" % n, "export written beside verdict.jsonl, footer sha256 recomputed EQUAL",
                  measured, good, evidence=evidence)

    def step(self, sid, expected, measured, ok, evidence=None, hard=False):
        self.steps += 1
        if not ok:
            self.fails += 1
        row = {"step": sid, "expected": str(expected), "measured": str(measured),
               "verdict": "PASS" if ok else "FAIL",
               "evidence": evidence or ["run_log.txt"]}
        self.verdict_f.write(json.dumps(row) + "\n")
        self.verdict_f.flush()
        self.note("STEP %-40s %s expected=%r measured=%r" % (sid, "PASS" if ok else "FAIL", expected, measured))
        if not ok and (hard or not self.continue_on_fail):
            raise StepFailure(sid)

    # ---- the P7 census instrument (byte-format identical to the probe's) ----
    def census(self, out_path):
        home = os.path.expanduser("~")
        cands = [os.path.join(home, ".local/share", APP_ID_DIR),
                 os.path.join(home, ".config/qsc"),
                 os.path.join(home, ".cache", APP_ID_DIR),
                 os.path.join(home, ".local/state", APP_ID_DIR)]
        tops = [os.path.join(home, d) for d in (".config", ".local/share", ".local/state", ".cache")]
        env = dict(os.environ, LC_ALL="C")
        with open(out_path, "w") as out:
            for d in cands:
                if os.path.exists(d):
                    out.write("== PRESENT %s\n" % d)
                    r = subprocess.run(["find", d, "-printf", "%y %m %s %T@ %p\\n"],
                                       capture_output=True, text=True, env=env)
                    out.write("".join(sorted(r.stdout.splitlines(True))))
                else:
                    out.write("== ABSENT %s\n" % d)
            for t in tops:
                out.write("== TOP %s\n" % t)
                r = subprocess.run(["find", t, "-maxdepth", "1", "-type", "d",
                                    "-printf", "%y %m %T@ %p\\n"],
                                   capture_output=True, text=True, env=env)
                out.write("".join(sorted(r.stdout.splitlines(True))))

    def guard_destructive(self, what):
        pre = self.run / "census_pre.txt"
        if not pre.exists() or pre.stat().st_size == 0:
            self.note("REFUSED: pre-census absent; destructive step blocked (A1.1): " + what)
            print("REFUSED: pre-census absent; destructive step blocked (A1.1)")
            raise Refusal(3)

    # ---- client subprocess wrapper ----
    def wd(self, *args, timeout=60):
        r = subprocess.run([sys.executable, CLIENT] + [str(a) for a in args],
                           capture_output=True, text=True, timeout=timeout, env=self.wd_env)
        return r.returncode, r.stdout

    def find(self, sel):
        rc, out = self.wd("find", "css selector", sel)
        if rc != 0:
            raise StepFailure("find failed: %s -> %s" % (sel, out.strip()))
        return out.strip()

    def prop(self, sel, prop):
        rc, out = self.wd("prop", self.find(sel), prop)
        return out.strip() if rc == 0 else None

    # ---- launch / teardown ----
    def launch(self):
        self.launches += 1
        n = self.launches
        attempts = []
        ready = False
        for attempt in range(1, 4):
            p1, p2 = probe_two_ports(attempts)
            if p1 is None:
                break
            env = dict(os.environ)
            env.pop("TAURI_AUTOMATION", None)       # R164 §3: NEITHER automation env —
            env.pop("TAURI_WEBVIEW_AUTOMATION", None)  # the pinned driver injects both.
            env.update({"GDK_BACKEND": "x11",
                        "XDG_CONFIG_HOME": str(self.profile / "config"),
                        "XDG_DATA_HOME": str(self.profile / "data"),
                        "XDG_CACHE_HOME": str(self.profile / "cache"),
                        "XDG_STATE_HOME": str(self.profile / "state"),
                        "QSLD_DATA_DIR": str(self.profile / "qsld-data")})
            so = open(self.run / ("launch%d_driver_stdout.log" % n), "w")
            se = open(self.run / ("launch%d_driver_stderr.log" % n), "w")
            self.proc = subprocess.Popen(
                ["xvfb-run", "-a", "-s", "-screen 0 1280x800x24",
                 "dbus-run-session", "--", self.driver,
                 "--port", str(p1), "--native-port", str(p2)],
                env=env, stdout=so, stderr=se, start_new_session=True)
            self.pgid = os.getpgid(self.proc.pid)
            self.wd_env["WD_PORT"] = str(p1)
            self.wd_env["WD_SESSION_FILE"] = str(self.run / ("session_id_%d" % n))
            self.wd_env["WD_TRANSCRIPT"] = str(self.run / "http_transcript.jsonl")
            polls = 0
            while polls < POLL_MAX:
                rc, _ = self.wd("status")
                if rc == 0:
                    ready = True
                    break
                if self.proc.poll() is not None and polls > 2:
                    break
                time.sleep(1)
                polls += 1
            self.note("launch %d attempt %d ports=%d/%d status_polls=%d ready=%s pgid=%d"
                      % (n, attempt, p1, p2, polls + 1, ready, self.pgid))
            if ready:
                break
            self.teardown(expect_session=False)
        self.step("launch%d_ready" % n, "driver ready, <=3 attempts",
                  "attempt=%d attempts=%s" % (attempt, json.dumps(attempts)), ready, hard=True)
        rc, out = self.wd("create-session", self.binary, 45, timeout=50)
        self.session_live = (rc == 0)
        sid_txt = ""
        if rc == 0:
            sid_txt = open(self.wd_env["WD_SESSION_FILE"]).read().strip()
        self.step("launch%d_session" % n, "HTTP200+sessionId (45s stated, NEITHER automation env)",
                  "rc=%d sessionId=%s" % (rc, sid_txt), rc == 0, hard=True)
        if n == 1:
            self.liveness_pair()

    def liveness_pair(self):
        rc, out = self.wd("find", "css selector", "#na0701-absent-control")
        ok = rc == 2 and "no such element" in out
        self.step("liveness_absent_selector", "rc=2 + `no such element`",
                  "rc=%d body=%s" % (rc, out.strip()[:80]), ok)
        rc, out = self.wd("text", self.find("#scr-wizard-vault h1"))
        wrong = "Create your vaulX"
        self.step("liveness_wrong_text", "measured != deliberately-wrong %r" % wrong,
                  "measured=%r" % out, out != wrong and out != "")

    def teardown(self, expect_session=True):
        if self.session_live and expect_session:
            self.debug_log_capture(self.launches)
            rc, _ = self.wd("delete-session")
            self.note("delete_session_rc=%d" % rc)
            self.session_live = False
        polls = 0
        while polls < POLL_MAX and subprocess.run(["pgrep", "-x", "qsl-desktop"],
                                                  capture_output=True).returncode == 0:
            time.sleep(1)
            polls += 1
        if self.pgid is not None:
            try:
                os.killpg(self.pgid, signal.SIGTERM)
            except ProcessLookupError:
                pass
        survivors = 1
        polls2 = 0
        while polls2 < POLL_MAX:
            r = subprocess.run(["pgrep", "-a", "Xvfb|tauri-driver|WebKitWebDriver|qsl-desktop"],
                               capture_output=True, text=True)
            survivors = r.returncode
            if survivors != 0:
                break
            time.sleep(1)
            polls2 += 1
        self.note("teardown launch=%d app_gone_polls=%d pgid=%s survivor_census_rc=%d (1 = zero survivors) polls=%d"
                  % (self.launches, polls, self.pgid, survivors, polls2))
        self.pgid = None
        self.proc = None

    # ---- ops ----
    def op_poll_exec(self, st):
        want = st["expect"]
        mode = st.get("mode", "eq")
        limit = st.get("poll", POLL_MAX)
        label = st.get("label", st["script"][:40])
        polls = 0
        last = None
        while polls < limit:
            rc, out = self.wd("execute", st["script"])
            if rc == 0:
                try:
                    last = json.loads(out)["value"]
                except (ValueError, KeyError):
                    last = "<unparseable:%s>" % out.strip()[:40]
                if mode == "ge":
                    try:
                        if float(last) >= float(want):
                            self.step("poll_exec %s" % label,
                                      ">= %s" % want,
                                      "%s after %d poll(s)" % (last, polls + 1), True)
                            return
                    except (TypeError, ValueError):
                        pass
                elif last == want:
                    self.step("poll_exec %s" % label, want,
                              "%s after %d poll(s)" % (last, polls + 1), True)
                    return
            time.sleep(1)
            polls += 1
        self.step("poll_exec %s" % label,
                  (">= %s" % want) if mode == "ge" else want,
                  "TIMEOUT after %d polls, last=%s" % (polls, last), False)

    def op_poll_screen(self, screen, want):
        polls = 0
        last = None
        while polls < POLL_MAX:
            try:
                last = self.prop("#" + screen, "className")
            except StepFailure:
                last = "<find-error>"
            if last == json.dumps(want):
                self.step("poll_%s_%s" % (screen, "visible" if want == "screen" else "hidden"),
                          "className=%r" % want, "polls=%d" % (polls + 1), True)
                if screen == "scr-main" and want == "screen":
                    self.debug_log_arm()
                return
            time.sleep(1)
            polls += 1
        self.step("poll_%s" % screen, "className=%r" % want,
                  "TIMEOUT after %d polls, last=%s" % (polls, last), False)

    def op_read_text(self, sel, literal, poll=15):
        polls = 0
        out = ""
        while polls < poll:
            rc, out = self.wd("text", self.find(sel))
            if rc == 0 and out == literal:
                self.step("read_text %s" % sel, literal, "byte-equal, polls=%d" % (polls + 1), True)
                return
            time.sleep(1)
            polls += 1
        self.step("read_text %s" % sel, literal, "measured=%r after %d polls" % (out, polls), False)

    def run_scenario(self):
        pre = self.run / "census_pre.txt"
        self.census(pre)
        self.note("census_pre_lines=%d" % sum(1 for _ in open(pre)))
        if self.scenario.get("destructive"):
            self.guard_destructive("scenario start (destructive scenario requires the pre-census)")
        for st in self.scenario["steps"]:
            op = st["op"]
            if op == "launch":
                self.launch()
            elif op == "teardown":
                self.teardown()
            elif op == "find_count":
                rc, out = self.wd("elements", "css selector", st["sel"])
                self.step("count %s" % st["sel"], st["expect"], out.strip(),
                          rc == 0 and out.strip() == str(st["expect"]))
            elif op == "prop_eq":
                want = json.dumps(st["expect"])
                polls = 0
                got = None
                maxp = st.get("poll", 1)
                while polls < maxp:
                    got = self.prop(st["sel"], st["prop"])
                    if got == want:
                        break
                    polls += 1
                    if polls < maxp:
                        time.sleep(1)
                self.step("prop %s.%s" % (st["sel"], st["prop"]), want,
                          "%s (polls=%d)" % (got, polls + 1), got == want)
            elif op == "read_text":
                self.op_read_text(st["sel"], st["literal"], st.get("poll", 15))
            elif op == "read_tc":
                got = self.prop(st["sel"], "textContent")
                want = json.dumps(st["literal"])
                self.step("read_tc %s" % st["sel"], want, got,
                          got == want, ["instrument: textContent property (hidden-safe presence-class read)"])
            elif op == "type":
                el = self.find(st["sel"])
                text = st["text"].replace("{run_dir}", str(self.run))   # NA-0779: the export dir
                rc, _ = self.wd("sendkeys", el, text)
                got = self.prop(st["sel"], "value")
                self.step("type %s" % st["sel"], text, got,
                          rc == 0 and got == json.dumps(text))
            elif op == "clear_type":
                el = self.find(st["sel"])
                self.wd("clear", el)
                text = st["text"].replace("{run_dir}", str(self.run))   # NA-0779: the export dir
                rc, _ = self.wd("sendkeys", el, text)
                got = self.prop(st["sel"], "value")
                self.step("clear_type %s" % st["sel"], text, got,
                          rc == 0 and got == json.dumps(text))
            elif op == "click":
                if st.get("guard") == "destructive":
                    self.guard_destructive("click " + st["sel"])
                rc, _ = self.wd("click", self.find(st["sel"]))
                self.step("click %s" % st["sel"], "rc=0", "rc=%d" % rc, rc == 0)
            elif op == "poll_visible":
                self.op_poll_screen(st["screen"], "screen")
            elif op == "poll_hidden":
                self.op_poll_screen(st["screen"], "screen hidden")
            elif op == "exec":
                rc, out = self.wd("execute", st["script"])
                ok = rc == 0
                if ok and "expect" in st:
                    ok = json.loads(out)["value"] == st["expect"]
                self.step("exec %s" % st["script"][:40], st.get("expect", "rc=0"),
                          out.strip()[:80], ok)
            elif op == "poll_exec":
                # NA-0763 (D-0040, ruled R8): the ONE new op. `exec` is a ONE-SHOT
                # compare, which cannot observe a value that becomes true over time
                # -- a repeating timer above all. The shape is `op_poll_screen`'s
                # bounded loop and `countdown_commit`'s successive-reading idea,
                # neither of which was reusable (one is className-only, the other is
                # hard-wired to #countdown-number behind the destructive guard).
                #
                # ⚠ IT IS NOT A SLEEP. It is a BOUNDED poll that FAILS on timeout,
                # so a tick that never fires produces a red row rather than a run
                # that quietly took longer. `mode: "ge"` compares numerically for
                # counters; the default compares the value verbatim.
                self.op_poll_exec(st)
            elif op == "file_present":
                p = self.profile / st["path"]
                polls = 0
                while polls < st.get("poll", 10) and not p.exists():
                    time.sleep(1)
                    polls += 1
                self.step("file_present %s" % st["path"], "exists",
                          "exists=%s polls=%d" % (p.exists(), polls), p.exists())
            elif op == "file_absent":
                p = self.profile / st["path"]
                polls = 0
                while polls < st.get("poll", 10) and p.exists():
                    time.sleep(1)
                    polls += 1
                self.step("file_absent %s" % st["path"], "absent",
                          "exists=%s polls=%d" % (p.exists(), polls), not p.exists())
            elif op == "dir_exists_empty":
                p = self.profile / st["path"]
                polls = 0
                while polls < st.get("poll", 10) and not p.is_dir():
                    time.sleep(1)
                    polls += 1
                entries = sorted(x.name for x in p.iterdir()) if p.is_dir() else None
                self.step("dir_exists_empty %s" % st["path"], "dir exists, empty",
                          "is_dir=%s entries=%s polls=%d" % (p.is_dir(), entries, polls),
                          p.is_dir() and entries == [])
            elif op == "json_field_eq":
                p = self.profile / st["file"]
                polls = 0
                got = None
                while polls < st.get("poll", 10):
                    if p.exists():
                        try:
                            got = json.load(open(p)).get(st["key"])
                        except ValueError:
                            got = "<unparseable>"
                        if got == st["expect"]:
                            break
                    time.sleep(1)
                    polls += 1
                self.step("json_field %s.%s" % (st["file"], st["key"]), st["expect"],
                          "%s (polls=%d)" % (got, polls + 1), got == st["expect"])
            elif op == "countdown_commit":
                self.guard_destructive("countdown_commit (the erase commit)")
                seen = []
                polls = 0
                while polls < COUNTDOWN_POLL_MAX:
                    try:
                        rc, out = self.wd("text", self.find("#countdown-number"))
                        if rc == 0 and out.strip().isdigit() and out.strip() not in seen:
                            seen.append(out.strip())
                        # the commit observable is the reload landing in S0: the
                        # wizard flips visible (#countdown-number survives the
                        # reload as a hidden static node, so its absence never
                        # signals; :546 reload -> route() -> show wizard).
                        if self.prop("#scr-wizard-vault", "className") == json.dumps("screen"):
                            break
                    except StepFailure:
                        break  # reload transition window — a find raced the document swap
                    time.sleep(1)
                    polls += 1
                self.step("countdown_polled", ">=2 distinct polled values, never slept-past",
                          "seen=%s polls=%d" % (seen, polls), len(seen) >= 2)
                # NA-0776 (RULING_011 R12): THE TRAILING WIZARD POLL MOVED OUT OF THIS OP.
                # It read `self.op_poll_screen("scr-wizard-vault", "screen")` and observed
                # the wizard IN THE SAME WebDriver session, which was correct while the
                # erase commit ended in `window.location.reload()`. Cure B replaces that
                # reload with a PROCESS RESTART, which destroys the session, so the
                # embedded observable became FALSE -- an op that can no longer see what it
                # asserts. The ASSERTION is not dropped: f_e now performs it after a
                # teardown+launch, which proves S0 is reached across a REAL restart -- a
                # stronger claim than the reload version made. One consumer (f_e), so the
                # relocation is complete by construction.
            elif op == "note":
                self.note("SCENARIO-NOTE: " + st["text"])
            else:
                raise StepFailure("unknown op %s" % op)

    def finish(self):
        # bracket close + evidence freeze order (R164 §5): verdict FROZEN first,
        # run log next, manifest LAST.
        if self.proc is not None or self.session_live:
            self.teardown()
        post = self.run / "census_post.txt"
        self.census(post)
        pre = self.run / "census_pre.txt"
        identical = pre.exists() and post.exists() and pre.read_bytes() == post.read_bytes()
        row = {"step": "isolation_bracket", "expected": "pre/post real-$HOME census byte-identical",
               "measured": "identical=%s" % identical,
               "verdict": "PASS" if identical else "FAIL",
               "evidence": ["census_pre.txt", "census_post.txt"]}
        self.steps += 1
        if not identical:
            self.fails += 1
        self.verdict_f.write(json.dumps(row) + "\n")
        with open(self.run / "profile_footprint.txt", "w") as f:
            r = subprocess.run(["find", str(self.profile), "-printf", "%y %m %s %p\\n"],
                               capture_output=True, text=True)
            f.write("".join(sorted(r.stdout.splitlines(True))))
        result = "PASS" if self.fails == 0 else "FAIL"
        self.verdict_f.write(json.dumps({"scenario": self.name, "result": result,
                                         "steps": self.steps}) + "\n")
        self.verdict_f.flush()
        os.fsync(self.verdict_f.fileno())
        self.verdict_f.close()          # the verdict log is now FROZEN
        self.note("RESULT=%s steps=%d fails=%d" % (result, self.steps, self.fails))
        self.log_f.flush()
        os.fsync(self.log_f.fileno())
        self.log_f.close()              # the run log is now frozen
        manifest = {"scenario": self.name, "producer": self.producer_identity(),
                    "artifacts": {}}
        for p in sorted(self.run.rglob("*")):
            if p.is_file() and p.name != "MANIFEST.json":
                manifest["artifacts"][str(p.relative_to(self.run))] = sha256(p)
        with open(self.run / "MANIFEST.json", "w") as f:
            json.dump(manifest, f, indent=1)   # written LAST (R164 §5)
        print("RUN_DIR=%s" % self.run)
        print("RESULT=%s" % result)
        return 0 if result == "PASS" else 1

    def producer_identity(self):
        ident = {"driver_path": self.driver, "binary": self.binary}
        r = subprocess.run(["dpkg-query", "-W", "webkit2gtk-driver"], capture_output=True, text=True)
        ident["webkit2gtk-driver"] = r.stdout.strip()
        r = subprocess.run(["rustc", "--version"], capture_output=True, text=True)
        ident["rustc"] = r.stdout.strip()
        return ident


def main():
    if len(sys.argv) != 2:
        print("usage: runner.py <scenario.json>", file=sys.stderr)
        sys.exit(4)
    rn = Runner(sys.argv[1])
    try:
        rn.run_scenario()
    except StepFailure as e:
        rn.note("SCENARIO ABORTED: %s" % e)
    except Refusal:
        raise
    finally:
        code = rn.finish()   # teardown on EVERY exit path (A1.15) + bracket close
    sys.exit(code)


if __name__ == "__main__":
    main()

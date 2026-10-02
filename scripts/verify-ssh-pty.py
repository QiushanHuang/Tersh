"""Real SSH transport and nested Tersh acceptance against one explicit SSH alias.

Run only after the target, installed binaries and isolated remote root are ready:
  python3 scripts/verify-ssh-pty.py --host ALIAS --remote-binary /absolute/tersh \
    --remote-root /absolute/test-root --local-binary /absolute/tersh --output receipt.json

--cluster-config may provide a single-server inventory targeting exactly ALIAS.
The original inventory is never changed. A temporary copy points at the unique
remote fixture directory. No SSH configuration, installation or service is changed.
Remote fixture receipts are retained under --remote-root for inspection.
"""
import argparse
import base64
import codecs
import fcntl
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import pty
import re
import select
import shlex
import shutil
import signal
import struct
import subprocess
import sys
import tempfile
import termios
import time
import unicodedata


SAFE_SSH_OPTIONS = ["-o", "BatchMode=yes", "-o", "StrictHostKeyChecking=yes",
                    "-o", "ConnectTimeout=15", "-o", "ConnectionAttempts=1",
                    "-o", "ClearAllForwardings=yes", "-o", "PermitLocalCommand=no",
                    "-o", "ServerAliveInterval=10", "-o", "ServerAliveCountMax=2"]
JSON_MARKER = "__TERSH_SSH_QA_JSON__"


class Screen:
    """Small UTF-8/cell-width decoder for QA, not a complete terminal emulator."""
    def __init__(self, width=120, height=30):
        self.decoder = codecs.getincrementaldecoder("utf-8")("replace")
        self.pending = ""
        self.saved_cursor = (0, 0)
        self.main_buffer = None
        self.alternate = False
        self.resize(width, height)

    def resize(self, width, height):
        self.width, self.height = width, height
        self.cells = [[" "] * width for _ in range(height)]
        self.x = self.y = 0
        if self.main_buffer is not None:
            cells, x, y = self.main_buffer
            resized = [[" "] * width for _ in range(height)]
            for row in range(min(height, len(cells))):
                resized[row][:min(width, len(cells[row]))] = cells[row][:width]
            self.main_buffer = (resized, min(x, width - 1), min(y, height - 1))

    @property
    def text(self):
        return "\n".join("".join(row) for row in self.cells)

    def newline(self):
        self.y += 1
        if self.y >= self.height:
            self.cells.pop(0)
            self.cells.append([" "] * self.width)
            self.y = self.height - 1

    def feed(self, data):
        text = self.pending + self.decoder.decode(data)
        self.pending = ""
        index = 0
        while index < len(text):
            ch = text[index]
            if ch == "\x1b":
                if index + 1 >= len(text):
                    self.pending = text[index:]
                    break
                second = text[index + 1]
                if second == "]":
                    ending = re.search(r"\x07|\x1b\\", text[index + 2:])
                    if ending is None:
                        self.pending = text[index:]
                        break
                    index += 2 + ending.end()
                    continue
                if second == "[":
                    match = re.match(r"\x1b\[([0-?]*)([ -/]*)([@-~])", text[index:])
                    if match is None:
                        self.pending = text[index:]
                        break
                    params, _, code = match.groups()
                    nums = [int(part) if part.isdigit() else 0 for part in params.lstrip("?").split(";")]
                    amount = nums[0] or 1
                    if code in "Hf":
                        self.y = min(self.height - 1, max(0, (nums[0] or 1) - 1))
                        self.x = min(self.width - 1, max(0, ((nums[1] if len(nums) > 1 else 1) or 1) - 1))
                    elif code == "A": self.y = max(0, self.y - amount)
                    elif code == "B": self.y = min(self.height - 1, self.y + amount)
                    elif code == "C": self.x = min(self.width - 1, self.x + amount)
                    elif code == "D": self.x = max(0, self.x - amount)
                    elif code == "G": self.x = min(self.width - 1, amount - 1)
                    elif code == "d": self.y = min(self.height - 1, amount - 1)
                    elif code == "J":
                        if nums[0] in (2, 3): self.cells = [[" "] * self.width for _ in range(self.height)]
                        elif nums[0] == 0:
                            self.cells[self.y][self.x:] = [" "] * (self.width - self.x)
                            for row in range(self.y + 1, self.height): self.cells[row] = [" "] * self.width
                    elif code == "K":
                        start = 0 if nums[0] in (1, 2) else self.x
                        end = self.width if nums[0] in (0, 2) else self.x + 1
                        self.cells[self.y][start:end] = [" "] * (end - start)
                    elif code == "s": self.saved_cursor = (self.x, self.y)
                    elif code == "u": self.x, self.y = self.saved_cursor
                    elif code == "h" and params == "?1049":
                        if not self.alternate:
                            self.main_buffer = ([row[:] for row in self.cells], self.x, self.y)
                        self.alternate = True
                        self.cells = [[" "] * self.width for _ in range(self.height)]
                        self.x = self.y = 0
                    elif code == "l" and params == "?1049":
                        if self.alternate and self.main_buffer is not None:
                            self.cells, self.x, self.y = self.main_buffer
                            self.main_buffer = None
                        self.alternate = False
                    index += len(match.group(0))
                    continue
                if second in "()":
                    if index + 2 >= len(text): self.pending = text[index:]; break
                    index += 3
                    continue
                if second == "7": self.saved_cursor = (self.x, self.y)
                if second == "8": self.x, self.y = self.saved_cursor
                index += 2
                continue
            if ch == "\r": self.x = 0
            elif ch == "\n": self.newline()
            elif ch == "\b": self.x = max(0, self.x - 1)
            elif ch == "\t": self.x = min(self.width - 1, (self.x // 8 + 1) * 8)
            elif ord(ch) >= 32:
                if unicodedata.combining(ch):
                    previous = min(self.x - 1, self.width - 1)
                    while previous >= 0 and self.cells[self.y][previous] == "": previous -= 1
                    if previous >= 0: self.cells[self.y][previous] += ch
                else:
                    width = 2 if unicodedata.east_asian_width(ch) in ("W", "F") else 1
                    if self.x + width > self.width: self.x = 0; self.newline()
                    self.cells[self.y][self.x] = ch
                    if width == 2: self.cells[self.y][self.x + 1] = ""
                    self.x += width
            index += 1


REMOTE_RUNNER = r'''
import argparse,base64,json,os,signal,subprocess,sys,termios,time
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--kind',required=True);p.add_argument('--binary');p.add_argument('--path');p.add_argument('--host');p.add_argument('--command-base64');a=p.parse_args()
root=Path(__file__).resolve().parent
if a.kind not in ('direct','cluster-workbench','cluster-shell'):raise SystemExit('invalid session kind')
state=root/('session-'+a.kind+'.json')
def attrs():
    values=termios.tcgetattr(0)
    return [values[:6],[(item.hex() if isinstance(item,bytes) else item) for item in values[6]]]
record={'kind':a.kind,'tty':os.ttyname(0),'before':attrs(),'sizes':[],'started':time.time()}
def save():
    temporary=state.with_suffix('.json.tmp');temporary.write_text(json.dumps(record),encoding='utf-8');os.replace(temporary,state)
def resized(*unused):
    size=os.get_terminal_size(0);record['sizes'].append([size.columns,size.lines]);save()
signal.signal(signal.SIGWINCH,resized);resized()
env=os.environ.copy()
for key in list(env):
    if key.startswith('TERSH_'):env.pop(key,None)
env.update(TERM='xterm-ghostty',LC_ALL='C.UTF-8',HISTFILE='/dev/null',XDG_CONFIG_HOME=str(root/'config'),XDG_STATE_HOME=str(root/'state'),TERSH_PLACES_FILE=str(root/'state'/'places.json'),TERSH_CLIPBOARD='off',TERSH_MOTION='off',INPUTRC=str(root/'inputrc'))
if a.kind=='direct':
    env.update(TERSH_HOST_ALIAS=a.host,TERSH_HOST_ID='ssh-qa:'+a.host)
    command=[a.binary,'--ui-profile','ssh',a.path]
elif a.kind=='cluster-shell':
    env.update(PS1='SSH_QA> ',PROMPT_COMMAND='')
    command=['/bin/bash','--noprofile','--norc','-i']
else:command=['sh','-c',base64.b64decode(a.command_base64).decode('utf-8')]
try:
    record['returncode']=subprocess.call(command,env=env,cwd=root)
finally:
    record['after']=attrs();record['restored']=record['after']==record['before'];record['finished']=time.time();save()
print('__TERSH_SSH_QA_END__'+a.kind,flush=True)
raise SystemExit(record.get('returncode',1))
'''


SETUP = r'''
import base64,hashlib,json,os,subprocess,sys,tempfile
from pathlib import Path
a=json.loads(base64.b64decode(sys.argv[1]));root=Path(a['root']);binary=Path(a['binary'])
if not root.is_absolute() or not root.is_dir() or root.is_symlink() or root.resolve()!=root:raise SystemExit('remote-root must already be an absolute canonical directory, not a symlink')
if root in (Path('/'),Path.home().resolve()):raise SystemExit('remote-root must be an isolated test directory, not the filesystem or home root')
if not binary.is_absolute() or not binary.is_file():raise SystemExit('remote-binary must be an existing absolute file')
if not Path('/bin/bash').is_file():raise SystemExit('controlled /bin/bash is unavailable')
terminfo=subprocess.run(['infocmp','xterm-ghostty'],stdout=subprocess.DEVNULL,stderr=subprocess.PIPE)
if terminfo.returncode:raise SystemExit('xterm-ghostty terminfo is unavailable; no installation attempted')
resolved=subprocess.check_output(['sh','-c','PATH="$HOME/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:/usr/local/cuda/bin:$PATH"; command -v tersh'],text=True).strip()
if not resolved or hashlib.sha256(Path(resolved).read_bytes()).hexdigest()!=hashlib.sha256(binary.read_bytes()).hexdigest():raise SystemExit('cluster command-v tersh does not match remote-binary; install/resolve the intended version before testing')
run=Path(tempfile.mkdtemp(prefix='ssh-pty-',dir=root))
for name in ('config','state'): (run/name).mkdir()
(run/'readme.txt').write_text('ssh-fixture-marker\n',encoding='utf-8')
(run/'stream.log').write_text('ssh-log-marker\n',encoding='utf-8')
(run/'中文目录').mkdir();(run/'中文目录'/'unicode-target-marker.txt').write_text('unicode path verified\n',encoding='utf-8')
(run/'inputrc').write_text('set editing-mode emacs\nset enable-bracketed-paste off\n',encoding='utf-8')
(run/'remote_runner.py').write_text(a['runner'],encoding='utf-8')
print('__TERSH_SSH_QA_JSON__'+json.dumps({'run':str(run),'platform':os.uname().sysname,'architecture':os.uname().machine,'python_version':sys.version.split()[0],'binary':str(binary),'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'cluster_resolved_binary':resolved,'term':'xterm-ghostty'},ensure_ascii=False))
'''


READ_FILES = r'''
import base64,json,sys
from pathlib import Path
a=json.loads(base64.b64decode(sys.argv[1]));root=Path(a['root']).resolve();out={}
for name in a['files']:
    path=root/name
    if path.resolve().is_relative_to(root) is False or path.is_symlink():raise SystemExit('refusing read outside fixture root')
    if path.exists():out[name]=path.read_text(encoding='utf-8')
print('__TERSH_SSH_QA_JSON__'+json.dumps(out,ensure_ascii=False))
'''


def encoded(value):
    return base64.b64encode(json.dumps(value, ensure_ascii=False).encode()).decode()


def wrapper_command(config, kind, command=None):
    args = ["python3", config["remote_runner"], "--kind", kind]
    if command is not None:
        args += ["--command-base64", base64.b64encode(command.encode()).decode()]
    return shlex.join(args)


def cluster_ssh_argv(argv, config):
    """Restrict the shim to the explicit target; preserve Tersh's remote command."""
    index = 0
    while index < len(argv) and argv[index].startswith("-"):
        flag = argv[index]
        if flag in ("-o", "-S", "-p", "-l", "-i"):
            index += 2
        elif flag in ("-n", "-t", "-tt", "-T", "-q"):
            index += 1
        else:
            raise ValueError("Unexpected SSH option in isolated cluster test: " + flag)
    if index >= len(argv) or argv[index] != config["host"]:
        raise ValueError("Refusing SSH target outside the explicitly authorized alias")
    tail = argv[index + 1:]
    if not tail:
        return SAFE_SSH_OPTIONS + argv[:index] + ["-tt", argv[index], wrapper_command(config, "cluster-shell")]
    if len(tail) != 1:
        raise ValueError("Unexpected remote argument topology")
    if "-t" in argv[:index] or "-tt" in argv[:index]:
        return SAFE_SSH_OPTIONS + argv[:index + 1] + [wrapper_command(config, "cluster-workbench", tail[0])]
    return SAFE_SSH_OPTIONS + argv  # Read-only dashboard probe is unchanged.


class Remote:
    def __init__(self, host, program):
        self.host, self.program = host, program

    def command(self, command, tty=False):
        return [self.program, *SAFE_SSH_OPTIONS, *( ["-tt"] if tty else []), self.host, command]

    def python(self, source, payload):
        command = "python3 - " + shlex.quote(encoded(payload))
        result = subprocess.run(self.command(command), input=source, text=True, encoding="utf-8",
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=60)
        if result.returncode:
            raise RuntimeError("SSH fixture command failed: " + result.stderr[-3000:] + result.stdout[-1000:])
        for line in reversed(result.stdout.splitlines()):
            if line.startswith(JSON_MARKER): return json.loads(line[len(JSON_MARKER):])
        raise RuntimeError("SSH fixture command did not return its JSON receipt: " + result.stdout[-1000:])

    def read(self, root, *names):
        return self.python(READ_FILES, {"root": root, "files": list(names)})


def terminal_configuration(attributes):
    """Compare settings, not Darwin's transient pending-input state.

    A read-only OpenSSH control also leaves PENDIN set on a macOS PTY.
    Keep every other flag, speed and control character in the comparison.
    """
    flags = list(attributes[:6])
    if sys.platform == "darwin":
        flags[3] &= ~termios.PENDIN
    return flags, list(attributes[6])


class Session:
    def __init__(self, run, command, env, label):
        self.run, self.label = run, label
        self.screen = Screen()
        self.master, self.slave = pty.openpty()
        fcntl.ioctl(self.slave, termios.TIOCSWINSZ, struct.pack("HHHH", 30, 120, 0, 0))
        self.original = termios.tcgetattr(self.slave)
        self.process = subprocess.Popen(command, stdin=self.slave, stdout=self.slave, stderr=self.slave,
                                        env=env, start_new_session=True)
        self.closed = False
        run.sessions.append(self)

    def read(self, duration=.02):
        end = time.monotonic() + duration
        while time.monotonic() < end:
            if not select.select([self.master], [], [], max(0, end - time.monotonic()))[0]: break
            try: data = os.read(self.master, 262144)
            except OSError: break
            if not data: break
            self.screen.feed(data)

    def send(self, data):
        os.write(self.master, data.encode() if isinstance(data, str) else data)

    def wait(self, label, predicate, timeout=45):
        self.run.current = label
        end = time.monotonic() + timeout
        while not predicate():
            if self.process.poll() is not None or time.monotonic() >= end:
                raise AssertionError(label + "\n" + self.screen.text)
            self.read()

    def expect(self, label, *parts, absent=()):
        self.wait(label, lambda: all(part in self.screen.text for part in parts)
                  and all(part not in self.screen.text for part in absent))

    def resize(self, width):
        self.screen.resize(width, 30)
        fcntl.ioctl(self.slave, termios.TIOCSWINSZ, struct.pack("HHHH", 30, width, 0, 0))
        # The session owns this process group. Include a nested SSH child while
        # the local dashboard is suspended, matching a foreground tty resize.
        os.killpg(self.process.pid, signal.SIGWINCH)

    def capture(self, label):
        self.run.screens[label] = self.screen.text

    def finish(self):
        self.run.current = "finish " + self.label
        end = time.monotonic() + 15
        while self.process.poll() is None and time.monotonic() < end: self.read()
        if self.process.poll() is None: raise AssertionError("SSH/Tersh did not exit\n" + self.screen.text)
        self.close()

    def close(self):
        if self.closed: return
        try:
            if self.process.poll() is None:
                self.process.terminate()
                end = time.monotonic() + 5
                while self.process.poll() is None and time.monotonic() < end: self.read()
            if self.process.poll() is None: self.process.kill(); self.process.wait()
            after = termios.tcgetattr(self.slave)
            restored = terminal_configuration(after) == terminal_configuration(self.original)
            self.run.local_restorations.append({"session": self.label, "exit_code": self.process.returncode,
                "restored": restored, "exact_termios_match": after == self.original,
                "transient_pendin_changed": bool((after[3] ^ self.original[3]) & getattr(termios, "PENDIN", 0))})
            if self.process.returncode != 0 or not restored: raise AssertionError("local terminal/exit not restored for " + self.label)
        finally:
            os.close(self.master); os.close(self.slave); self.closed = True


class Run:
    def __init__(self, args, local_root):
        self.args, self.local_root = args, local_root
        self.remote = Remote(args.host, shutil.which("ssh"))
        if self.remote.program is None: raise RuntimeError("OpenSSH client is unavailable")
        self.sessions, self.screens, self.local_restorations = [], {}, []
        self.current = "remote fixture preparation"
        self.info = self.remote.python(SETUP, {"root": args.remote_root, "binary": args.remote_binary, "runner": REMOTE_RUNNER})
        self.root = self.info["run"]
        self.runner = str(PurePosixPath(self.root) / "remote_runner.py")
        self.checks, self.remote_receipts = {}, {}
        self.env = os.environ.copy()
        for key in list(self.env):
            if key.startswith("TERSH_") or key == "NO_COLOR": self.env.pop(key, None)
        self.env.update(TERM="xterm-ghostty", HISTFILE="/dev/null", XDG_CONFIG_HOME=str(local_root / "config"),
                        XDG_STATE_HOME=str(local_root / "state"), TERSH_CLIPBOARD="off")

    def saved(self, name):
        files = self.remote.read(self.root, name)
        return json.loads(files[name]) if name in files else None

    def expect_remote_size(self, kind, width, session):
        deadline = time.monotonic() + 15
        while True:
            state = self.saved("session-" + kind + ".json")
            if state is not None and [width, 30] in state["sizes"]: return
            if time.monotonic() >= deadline: raise AssertionError("SSH window-change did not reach remote PTY")
            session.read(.1)

    def direct(self):
        command = shlex.join(["python3", self.runner, "--kind", "direct", "--binary", self.args.remote_binary,
                              "--path", self.root + "/readme.txt", "--host", self.args.host])
        session = Session(self, self.remote.command(command, tty=True), self.env, "direct_ssh")
        session.expect("real SSH preview", "Preview | raw", "ssh-fixture-marker", self.args.host)
        session.send("o"); session.expect("real SSH actions", "Actions", "Enter run"); session.capture("ssh_actions")
        session.send(b"\x1b"); session.expect("leave SSH actions", "Preview | raw", absent=("Enter run",))
        session.send("q"); session.expect("remote file view", "Files", absent=("Preview | raw",))
        session.send(":"); session.expect("remote goto input", "Go to directory")
        session.send(self.root + "/中文目录x"); session.send(b"\x7f\r")
        session.expect("UTF-8 path plus backspace navigates on remote", "unicode-target-marker.txt", absent=("Go to directory",))
        places = self.saved("state/places.json")
        assert any(place["path"] == self.root + "/中文目录" and place["host"] == self.args.host for place in places)
        session.capture("ssh_unicode_path")
        for width in (40, 80, 120):
            session.resize(width)
            session.expect("SSH resize to " + str(width), "Files", "unicode-target-marker.txt")
            self.expect_remote_size("direct", width, session)
            session.capture("ssh_" + str(width) + "_columns")
        session.send("q"); session.finish()
        receipt = self.saved("session-direct.json")
        assert receipt["returncode"] == 0 and receipt["restored"]
        self.remote_receipts["direct"] = receipt
        self.checks["direct_ssh"] = {"actions": True, "unicode_path_verified_in_remote_places": True,
                                     "backspace": True, "resize_widths": [40, 80, 120], "remote_termios_restored": True}

    def cluster(self):
        if self.args.cluster_config:
            inventory = json.loads(self.args.cluster_config.read_text(encoding="utf-8"))
            assert not inventory.get("main_machine") and not inventory.get("jump_host"), "cluster fixture must contain only the authorized target"
            servers = inventory.get("servers", [])
            assert len(servers) == 1, "cluster fixture must have exactly one server"
            server = servers[0]
            assert (server.get("campus_ip") or server.get("alias")) == self.args.host
            assert not server.get("ssh_user") and not server.get("proxy_jump"), "use the verified SSH alias, not another explicit target/jump"
        else:
            server = {"alias": self.args.host, "campus_ip": self.args.host, "role": "Authorized SSH test"}
            inventory = {"servers": [server]}
        server["workdir"] = self.root
        display_host = server.get("alias") or self.args.host
        config_path = self.local_root / "single-host.json"
        config_path.write_text(json.dumps(inventory), encoding="utf-8")
        shim_dir = self.local_root / "bin"; shim_dir.mkdir()
        shim_config = {"host": self.args.host, "remote_runner": self.runner}
        shim = shim_dir / "ssh"
        shim.write_text("#!" + sys.executable + "\nimport importlib.util,os,sys\ns=importlib.util.spec_from_file_location('sshqa'," + repr(str(Path(__file__).resolve())) + ");m=importlib.util.module_from_spec(s);s.loader.exec_module(m)\na=m.cluster_ssh_argv(sys.argv[1:]," + repr(shim_config) + ")\nos.execv(" + repr(self.remote.program) + ",[" + repr(self.remote.program) + "]+a)\n", encoding="utf-8")
        shim.chmod(0o700)
        env = dict(self.env, PATH=str(shim_dir) + os.pathsep + self.env.get("PATH", ""))
        session = Session(self, [str(self.args.local_binary), "--ui-profile", "ssh", "--c", "--cluster-config", str(config_path)], env, "cluster_ssh")
        session.expect("single authorized host probe succeeds", display_host, "OK 1", "CHK 0/1")
        session.send("P"); session.expect("pause cluster while testing nested sessions", "PAUSED")
        session.send("t"); session.expect("cluster t opens remote workbench", "Files", "readme.txt", display_host, absent=("Cluster Status",))
        session.capture("cluster_remote_workbench")
        for width in (40, 80, 120):
            session.resize(width)
            session.expect("nested SSH resize to " + str(width), "Files", "readme.txt")
            self.expect_remote_size("cluster-workbench", width, session)
            session.capture("cluster_nested_" + str(width) + "_columns")
        session.send("o"); session.expect("nested workbench actions", "Actions", "Enter run")
        session.send(b"\x1b"); session.expect("nested actions close", "Files", absent=("Enter run",))
        session.send("q"); session.expect("q returns from remote workbench to local dashboard", "Cluster Status", "PAUSED", display_host)
        receipt = self.saved("session-cluster-workbench.json")
        assert receipt["returncode"] == 0 and receipt["restored"]
        self.remote_receipts["cluster_workbench"] = receipt
        places = self.saved("state/places.json")
        assert any(place["host"] == display_host and place["path"] == self.root and place["identity"] != "local" for place in places)
        session.capture("cluster_return_after_t")
        session.send("s"); session.expect("cluster s opens isolated real remote bash", "SSH_QA> ")
        # Bash/readline receives actual UTF-8, DEL and arrow key bytes. The remote
        # output file is the authoritative result, independent of screen width.
        output = self.root + "/shell-edited.txt"
        session.send("printf '%s\\n' '中文X")
        session.send(b"\x7f\x1b[DZ\x1b[C")
        session.send("' > " + shlex.quote(output) + "\r")
        session.expect("edited remote shell command returned", "SSH_QA> ")
        deadline = time.monotonic() + 15
        while True:
            files = self.remote.read(self.root, "shell-edited.txt")
            if "shell-edited.txt" in files: break
            if time.monotonic() >= deadline: raise AssertionError("remote edited command did not produce its fixture file")
            session.read(.1)
        assert files["shell-edited.txt"] == "中Z文\n", repr(files["shell-edited.txt"])
        session.capture("cluster_remote_shell_edit")
        session.send("exit\r"); session.expect("exit returns from remote shell to dashboard", "Cluster Status", "PAUSED", display_host)
        receipt = self.saved("session-cluster-shell.json")
        assert receipt["returncode"] == 0 and receipt["restored"]
        self.remote_receipts["cluster_shell"] = receipt
        session.capture("cluster_return_after_s")
        session.send("q"); session.finish()
        self.checks["cluster_transport"] = {"only_inventory_target": self.args.host, "t_roundtrip": True,
            "remote_host_path_scope": True, "s_roundtrip": True, "shell_unicode_backspace_arrows_verified_in_file": True,
            "remote_termios_restored": True, "nested_resize_widths": [40, 80, 120]}


def self_test():
    screen = Screen(12, 2)
    data = "A中文B".encode()
    screen.feed(data[:4]); screen.feed(data[4:])
    assert screen.cells[0][:6] == ["A", "中", "", "文", "", "B"]
    screen.feed(b"\x1b[1;6HZ")
    assert screen.cells[0][5] == "Z"
    screen.feed("\r\ne\u0301!".encode()); assert screen.cells[1][:2] == ["e\u0301", "!"]
    main = Screen(12, 2)
    main.feed(b"main\x1b[?1049hOTHER\x1b[?1049l")
    assert main.text.startswith("main") and main.x == 4 and not main.alternate
    main.feed(b"\x1b[?1049h"); main.resize(8, 2); main.feed(b"alt\x1b[?1049l")
    assert main.text.startswith("main") and main.x == 4
    config = {"host": "explicit-test-alias", "remote_runner": "/isolated/runner.py"}
    command = "sh -c 'exec tersh'"
    wrapped = cluster_ssh_argv(["-t", config["host"], command], config)
    assert base64.b64encode(command.encode()).decode() in wrapped[-1]
    probe = ["-n", "-o", "BatchMode=yes", config["host"], "read-only probe"]
    assert cluster_ssh_argv(probe, config)[-len(probe):] == probe
    try: cluster_ssh_argv(["different-target"], config)
    except ValueError: pass
    else: raise AssertionError("shim permitted an unauthorized alias")
    compile(REMOTE_RUNNER, "remote_runner.py", "exec"); compile(SETUP, "setup.py", "exec"); compile(READ_FILES, "read_files.py", "exec")
    local = type("LocalFixture", (), {"sessions": [], "local_restorations": []})()
    session = Session(local, [sys.executable, "-c", "pass"], os.environ.copy(), "local-constructor-fixture")
    session.finish()
    assert len(local.local_restorations) == 1
    assert local.local_restorations[0]["exit_code"] == 0 and local.local_restorations[0]["restored"]
    master, slave = pty.openpty()
    try:
        attributes = termios.tcgetattr(slave)
        changed = list(attributes); changed[3] ^= termios.ICANON
        assert terminal_configuration(changed) != terminal_configuration(attributes)
        if sys.platform == "darwin":
            changed[3] = attributes[3] ^ termios.PENDIN
            assert terminal_configuration(changed) == terminal_configuration(attributes)
    finally:
        os.close(master); os.close(slave)
    print("SSH QA decoder, alias restriction and embedded Python self-tests passed; no network used.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--host")
    parser.add_argument("--remote-binary")
    parser.add_argument("--remote-root")
    parser.add_argument("--local-binary", type=Path)
    parser.add_argument("--cluster-config", type=Path)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test: self_test(); return
    for name in ("host", "remote_binary", "remote_root", "local_binary", "output"):
        if getattr(args, name) is None: parser.error("--" + name.replace("_", "-") + " is required")
    if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._-]*", args.host): parser.error("--host must be one explicit SSH alias")
    for value in (args.remote_root, args.remote_binary):
        if not PurePosixPath(value).is_absolute() or any(ord(ch) < 32 for ch in value): parser.error("remote paths must be absolute and contain no control characters")
    args.local_binary = args.local_binary.resolve()
    if not args.local_binary.is_file(): parser.error("local binary is missing")
    receipt = {"status": "running", "host": args.host, "local_platform": os.uname().sysname,
               "local_binary": str(args.local_binary), "local_sha256": hashlib.sha256(args.local_binary.read_bytes()).hexdigest(),
               "harness": "Real OpenSSH transport; temporary local ssh shim only restricts the target and wraps interactive children with isolated remote state/termios capture. Tersh-generated workbench command is unchanged; shell uses bash --noprofile --norc and HISTFILE=/dev/null.",
               "limits": "Local terminal configuration comparison reports but excludes Darwin's transient PENDIN state (verified with a standalone SSH control). UTF-8 QA cell decoder is not a complete emulator. Unicode editing is verified from remote files/places; no GUI/mobile appearance, other host, disconnect recovery or load-test claim."}
    run = None
    with tempfile.TemporaryDirectory(prefix="tersh-ssh-pty-") as directory:
        try:
            run = Run(args, Path(directory))
            receipt["remote_fixture"] = run.info
            print(json.dumps({"phase": "direct_ssh", "remote_fixture": run.root}), flush=True)
            run.direct()
            print(json.dumps({"phase": "cluster_t_and_s"}), flush=True)
            run.cluster(); receipt["status"] = "passed"
        except Exception as error:
            receipt.update(status="failed", error=str(error), failed_check=run.current if run else "remote setup")
            if run and run.sessions: run.screens["failure"] = run.sessions[-1].screen.text
            raise
        finally:
            if run:
                for session in run.sessions:
                    if not session.closed:
                        try: session.close()
                        except Exception as error: receipt.setdefault("cleanup_errors", []).append(str(error))
                receipt.update(checks=run.checks, screens=run.screens, local_terminals=run.local_restorations, remote_terminals=run.remote_receipts)
            args.output.parent.mkdir(parents=True, exist_ok=True)
            args.output.write_text(json.dumps(receipt, ensure_ascii=False, indent=2), encoding="utf-8")
            print(json.dumps({"status": receipt["status"], "receipt": str(args.output.resolve()), "failed_check": receipt.get("failed_check")}))


if __name__ == "__main__":
    main()

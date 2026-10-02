"""Local POSIX PTY performance receipts. Synthetic files only; no SSH.

Usage: python3 scripts/verify-product-performance.py BINARY OUTPUT [--stability-seconds 600]
Process-cold launches use OS-warm directory data; this does not flush disk caches.
"""
import argparse
import ast
import codecs
import fcntl
import json
import os
from pathlib import Path
import pty
import re
import select
import signal
import shutil
import sys
import statistics
import struct
import subprocess
import tempfile
import termios
import time

# Reuse the checked-in ASCII screen decoder without executing that script's tests.
source = Path(__file__).with_name('verify-workflow-pty.py').read_text()
tree = ast.parse(source)
node = next(n for n in tree.body if isinstance(n, ast.ClassDef) and n.name == 'Screen')
exec(compile(ast.Module(body=[node], type_ignores=[]), str(Path(__file__).with_name('verify-workflow-pty.py')), 'exec'))

parser = argparse.ArgumentParser()
parser.add_argument('binary')
parser.add_argument('output')
parser.add_argument('--stability-seconds', type=int, default=600)
parser.add_argument('--startup-runs', type=int, default=30)
parser.add_argument('--skip-large', action='store_true')
args = parser.parse_args()
binary = str(Path(args.binary).resolve())
ps_program = shutil.which("ps")
if ps_program is None:
    raise RuntimeError("ps is required for RSS/CPU measurements; install procps or provide ps on PATH")


def p95(values):
    return sorted(values)[max(0, int(len(values) * .95 + .999) - 1)]


class Session:
    def __init__(self, path, state):
        self.screen = Screen(100, 30)
        self.master, self.slave = pty.openpty()
        fcntl.ioctl(self.slave, termios.TIOCSWINSZ, struct.pack('HHHH', 30, 100, 0, 0))
        self.original = termios.tcgetattr(self.slave)
        env = os.environ.copy()
        for name in ['TERSH_KEYMAP', 'TERSH_HOST_ALIAS', 'TERSH_HOST_ID', 'NO_COLOR']:
            env.pop(name, None)
        env.update(XDG_CONFIG_HOME=str(state/'config'), XDG_STATE_HOME=str(state/'state'), TERSH_PLACES='off', TERSH_CLIPBOARD='off')
        self.started = time.perf_counter()
        self.process = subprocess.Popen([binary, '--ui-profile', 'ssh', str(path)], stdin=self.slave, stdout=self.slave, stderr=self.slave, env=env, start_new_session=True)

    def read(self, timeout=.01):
        if select.select([self.master], [], [], timeout)[0]:
            try:
                data = os.read(self.master, 262144)
            except OSError:
                data = b''
            self.screen.feed(data)
            return len(data)
        return 0

    def until(self, predicate, timeout=30):
        deadline = time.monotonic()+timeout
        while not predicate(self.screen.text):
            if self.process.poll() is not None:
                raise AssertionError('TUI exited early: '+self.screen.text)
            if time.monotonic() > deadline:
                raise AssertionError('Timed out: '+self.screen.text)
            self.read(.01)

    def rss(self):
        return int(subprocess.check_output([ps_program, '-o', 'rss=', '-p', str(self.process.pid)]).strip())

    def cpu_seconds(self):
        value = subprocess.check_output([ps_program,'-o','time=','-p',str(self.process.pid)],text=True).strip()
        days = 0
        if '-' in value:
            prefix,value=value.split('-',1);days=int(prefix)
        parts=list(map(float,value.split(':')))
        return days*86400+sum(v*60**i for i,v in enumerate(reversed(parts)))

    def close(self):
        try:
            if self.process.poll() is None:
                os.write(self.master,b'\x03')
            # Keep draining while exiting: a PTY's finite output buffer can block
            # a renderer/terminal restoration if the harness only waits on PID.
            deadline = time.monotonic()+10
            while self.process.poll() is None and time.monotonic()<deadline:
                self.read(.02)
            if self.process.poll() is None:
                raise AssertionError('TUI did not exit while output was drained: '+self.screen.text)
            assert self.process.returncode == 0
            assert termios.tcgetattr(self.slave) == self.original
        finally:
            if self.process.poll() is None:
                self.process.kill()
                self.process.wait()
            os.close(self.master)
            os.close(self.slave)


def ready(text):
    return 'a000000.txt' in text and 'Loading directory' not in text and 'q quit' in '\n'.join(text.splitlines()[-3:])


receipt = {'binary':binary,'binary_bytes':Path(binary).stat().st_size,'platform':os.uname().sysname,'architecture':os.uname().machine,'python_version':sys.version.split()[0],'startup_definition':'File list visible and action menu responded with its footer; process-cold, OS file cache not flushed.','remote_hosts_contacted':False,'scenarios':{}}
with tempfile.TemporaryDirectory(prefix='tersh-product-performance-') as temp:
    root = Path(temp)
    small=root/'1000';small.mkdir()
    for n in range(1000):
        (small/f'a{n:06}.txt').write_text('bounded preview\n')
    startup=[];menus=[];rss=[]
    for iteration in range(args.startup_runs):
        s=Session(small,root)
        try:
            s.until(ready)
            began=time.perf_counter();os.write(s.master,b'o');s.until(lambda t:'Actions ' in t and 'Enter run' in '\n'.join(t.splitlines()[-3:]))
            menus.append((time.perf_counter()-began)*1000)
            startup.append((time.perf_counter()-s.started)*1000)
            rss.append(s.rss())
        finally:s.close()
    if startup:
        receipt['scenarios']['1000_files']={'runs':args.startup_runs,'startup_ms_p50':statistics.median(startup),'startup_ms_p95':p95(startup),'menu_response_ms_p95':p95(menus),'rss_kib_median':statistics.median(rss),'rss_kib_max_sample':max(rss)}
        print(json.dumps({'phase':'startup','result':receipt['scenarios']['1000_files']}),flush=True)
    for count in ([] if args.skip_large else [10000,100000]):
        directory=root/str(count);directory.mkdir()
        for n in range(count):(directory/f'a{n:06}.txt').touch()
        s=Session(directory,root)
        try:
            s.until(ready,timeout=60)
            elapsed=(time.perf_counter()-s.started)*1000
            began=time.perf_counter();os.write(s.master,b'o');s.until(lambda t:'Actions ' in t and 'Enter run' in '\n'.join(t.splitlines()[-3:]))
            receipt['scenarios'][f'{count}_files']={'startup_ms':elapsed,'menu_response_ms':(time.perf_counter()-began)*1000,'rss_kib_sample':s.rss()}
        finally:s.close()
        print(json.dumps({'phase':'large_directory','count':count,'result':receipt['scenarios'][f'{count}_files']}),flush=True)
    s=Session(small,root)
    try:
        s.until(ready)
        # Allow a pending initial preview to finish before measuring idle output.
        settle=time.monotonic()+.5
        while time.monotonic()<settle:s.read(.02)
        idle_bytes=0;samples=[s.rss()];cpu_before=s.cpu_seconds();start=time.monotonic();sample_at=start+60
        while time.monotonic()-start<args.stability_seconds:
            idle_bytes+=s.read(min(.5,max(0,args.stability_seconds-(time.monotonic()-start))))
            if time.monotonic()>=sample_at:
                samples.append(s.rss());sample_at+=60
                print(json.dumps({'phase':'stability','seconds':round(time.monotonic()-start),'rss_kib':samples[-1],'output_bytes':idle_bytes}),flush=True)
        samples.append(s.rss())
        receipt['scenarios']['idle_stability']={'seconds':args.stability_seconds,'output_bytes':idle_bytes,'rss_kib_samples':samples,'rss_delta_kib':samples[-1]-samples[0], 'cpu_seconds_delta':round(s.cpu_seconds()-cpu_before,4), 'cpu_time_source':'ps time; display precision limits apply'}
    finally:s.close()
receipt['terminal_modes_restored']=True
receipt['limitations']=f'{os.uname().sysname} native subprocess PTY only; memory figures are sampled RSS, not process peak. No forced cold disk cache, SSH-transport latency, other-operating-system or human usability claim.'
Path(args.output).parent.mkdir(parents=True,exist_ok=True)
Path(args.output).write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps(receipt,indent=2),flush=True)

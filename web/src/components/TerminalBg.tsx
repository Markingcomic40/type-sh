import { useEffect, useRef, useState } from 'react';

// The terminal running behind everything. Types a command, hits enter, dribbles
// out the output, picks another one, forever. Client only obviously.

// Fake values that need to at least look plausible
const randHex = (n: number): string =>
  Array.from(
    { length: n },
    () => '0123456789abcdef'[Math.floor(Math.random() * 16)],
  ).join('');
const randPick = <T,>(arr: T[]): T =>
  arr[Math.floor(Math.random() * arr.length)];

const PKGS = [
  'lodash',
  'react-query',
  'zod',
  'drizzle-orm',
  'vitest',
  'tsx',
  'framer-motion',
  'kysely',
  'hono',
  'tanstack-router',
];
const VERSIONS = [
  '1.4.2',
  '0.18.0',
  '3.22.4',
  '2.0.1',
  '1.6.0',
  '0.45.0',
  '11.2.6',
];
const SIZES = ['1.2', '3.4', '5.7', '8.1', '12.4', '0.9', '2.3'];

// Plain string if you dont care about the color, otherwise pick a cls:
// info | warn | err | ok | muted | prompt | accent
type OutLine = string | { txt: string; cls: string };
type Command = { cmd: () => string; lines: () => OutLine[] };

const COMMANDS: Command[] = [
  // --
  {
    cmd: () =>
      `brew install ${randPick(['fzf', 'ripgrep', 'neovim', 'tmux', 'bat', 'exa'])}`,
    lines: () => [
      {
        txt: '==> Downloading https://ghcr.io/v2/homebrew/core/...',
        cls: 'info',
      },
      '######################################################## 100.0%',
      {
        txt:
          '==> Pouring ' +
          randPick(PKGS) +
          '--' +
          randPick(VERSIONS) +
          '.arm64_sequoia.bottle.tar.gz',
        cls: 'info',
      },
      '🍺  /opt/homebrew/Cellar/' +
        randPick(PKGS) +
        '/' +
        randPick(VERSIONS) +
        ': 18 files, ' +
        randPick(SIZES) +
        'MB',
    ],
  },
  {
    cmd: () => `pnpm add ${randPick(PKGS)} ${randPick(PKGS)}`,
    lines: () => {
      const a = randPick(PKGS),
        b = randPick(PKGS);
      return [
        { txt: ' WARN  deprecated subdep@1.0.0', cls: 'warn' },
        'Packages: +' + (12 + Math.floor(Math.random() * 80)),
        '++++++++++++++++++++++++++++++++++++++++++++',
        'Progress: resolved 412, reused 388, downloaded 24, added 24',
        { txt: 'dependencies:', cls: 'ok' },
        `+ ${a} ${randPick(VERSIONS)}`,
        `+ ${b} ${randPick(VERSIONS)}`,
        {
          txt: 'Done in ' + (1.2 + Math.random() * 3).toFixed(1) + 's',
          cls: 'ok',
        },
      ];
    },
  },
  {
    cmd: () =>
      `sudo apt install -y ${randPick(['build-essential', 'postgresql', 'redis-server', 'docker.io'])}`,
    lines: () => [
      'Reading package lists... Done',
      'Building dependency tree... Done',
      'The following NEW packages will be installed:',
      '  ' + randPick(PKGS) + ' ' + randPick(PKGS) + ' ' + randPick(PKGS),
      'Need to get ' + randPick(SIZES) + ' MB of archives.',
      'Setting up ' + randPick(PKGS) + ' (' + randPick(VERSIONS) + ') ...',
      { txt: 'Processing triggers for man-db (2.11.2-2) ...', cls: 'muted' },
    ],
  },

  // --
  {
    cmd: () =>
      `git clone git@github.com:${randPick(['acme', 'vercel', 'fly', 'openai'])}/${randPick(['edge-fn', 'kit', 'infra', 'cli'])}.git`,
    lines: () => [
      "Cloning into '" + randPick(['edge-fn', 'kit', 'infra']) + "'...",
      { txt: 'remote: Enumerating objects: 4821, done.', cls: 'info' },
      'remote: Counting objects: 100% (4821/4821), done.',
      'Receiving objects: 100% (4821/4821), 8.42 MiB | 12.30 MiB/s, done.',
      'Resolving deltas: 100% (3104/3104), done.',
    ],
  },
  {
    cmd: () =>
      `git push origin ${randPick(['main', 'feat/typing-sim', 'fix/wpm-jitter', 'chore/deps'])}`,
    lines: () => {
      const a = randHex(7),
        b = randHex(7);
      return [
        'Enumerating objects: 14, done.',
        'Counting objects: 100% (14/14), done.',
        'Delta compression using up to 10 threads',
        'Writing objects: 100% (8/8), 1.21 KiB | 1.21 MiB/s, done.',
        { txt: 'To github.com:typesh/typesh.git', cls: 'info' },
        `   ${a}..${b}  main -> main`,
      ];
    },
  },
  {
    cmd: () => `git rebase -i HEAD~${2 + Math.floor(Math.random() * 5)}`,
    lines: () => [
      { txt: 'Successfully rebased and updated refs/heads/main.', cls: 'ok' },
    ],
  },

  // --
  {
    cmd: () => `cargo build --release`,
    lines: () => [
      {
        txt: '   Compiling ' + randPick(PKGS) + ' v' + randPick(VERSIONS),
        cls: 'info',
      },
      '   Compiling typesh v0.1.0',
      {
        txt:
          '    Finished release [optimized] target(s) in ' +
          (8 + Math.random() * 24).toFixed(2) +
          's',
        cls: 'ok',
      },
    ],
  },
  {
    cmd: () => `go test ./...`,
    lines: () => [
      'ok  \tgithub.com/typesh/core\t' + (0.2 + Math.random()).toFixed(3) + 's',
      'ok  \tgithub.com/typesh/render\t' +
        (0.1 + Math.random()).toFixed(3) +
        's',
      'ok  \tgithub.com/typesh/tui\t' + (0.4 + Math.random()).toFixed(3) + 's',
      { txt: 'PASS', cls: 'ok' },
    ],
  },
  {
    cmd: () => `pnpm test`,
    lines: () => [
      { txt: ' ✓ src/wpm.test.ts (12)', cls: 'ok' },
      { txt: ' ✓ src/typing.test.ts (28)', cls: 'ok' },
      { txt: ' ✓ src/render.test.ts (9)', cls: 'ok' },
      'Test Files  3 passed (3)',
      '     Tests  49 passed (49)',
      '  Duration  ' + (0.4 + Math.random() * 1.2).toFixed(2) + 's',
    ],
  },

  // --
  {
    cmd: () => 'whoami',
    lines: () => [randPick(['dev', 'ada', 'linus', 'grace', 'alan', '10x'])],
  },
  {
    cmd: () => 'pwd',
    lines: () => [
      randPick([
        '/home/dev/code/typesh',
        '/Users/ada/src/typesh',
        '/srv/typesh',
      ]),
    ],
  },
  {
    cmd: () => 'uname -a',
    lines: () => [
      'Darwin typesh.local 24.0.0 Darwin Kernel Version 24.0.0 arm64',
    ],
  },
  {
    cmd: () => 'neofetch',
    lines: () => [
      { txt: '       _._     ', cls: 'warn' },
      { txt: "    .-'`   `'-.   user@typesh", cls: 'warn' },
      { txt: "  .'  __    __ '. OS: Arch Linux x86_64", cls: 'warn' },
      { txt: ' /   /  \\  /  \\  \\ Kernel: 6.7.4-arch1', cls: 'warn' },
      { txt: ' |  |    ||    | | Shell: zsh 5.9', cls: 'warn' },
      { txt: " '. '-..--..'-' .' CPU: M3 Max (14)", cls: 'warn' },
      { txt: "   '-.________.-'  Memory: 18.2GB / 64GB", cls: 'warn' },
    ],
  },

  // --
  {
    cmd: () => `docker compose up -d`,
    lines: () => [
      '[+] Running 4/4',
      { txt: ' ✔ Container typesh-db-1     Started', cls: 'ok' },
      { txt: ' ✔ Container typesh-redis-1  Started', cls: 'ok' },
      { txt: ' ✔ Container typesh-api-1    Started', cls: 'ok' },
      { txt: ' ✔ Container typesh-web-1    Started', cls: 'ok' },
    ],
  },
  {
    cmd: () => `kubectl get pods -n production`,
    lines: () => [
      'NAME                       READY   STATUS    RESTARTS   AGE',
      'typesh-api-7d9c4b-x9k2j    1/1     Running   0          14h',
      'typesh-api-7d9c4b-pl4mz    1/1     Running   0          14h',
      'typesh-worker-58fb-qv2zt   1/1     Running   1          3d',
    ],
  },

  // Silly ones
  {
    cmd: () => 'sudo make me a sandwich',
    lines: () => [
      { txt: '[sudo] password for dev: ', cls: 'muted' },
      '🥪  okay.',
      { txt: "(it's mid)", cls: 'muted' },
    ],
  },
  {
    cmd: () => 'rm -rf node_modules',
    lines: () => [
      {
        txt:
          'freed ' +
          (180 + Math.floor(Math.random() * 600)) +
          'MB. you feel lighter.',
        cls: 'ok',
      },
    ],
  },
  {
    cmd: () => 'fortune | cowsay',
    lines: () => [
      ' _______________________________________ ',
      '< the cli is mightier than the gui      >',
      ' --------------------------------------- ',
      '        \\   ^__^',
      '         \\  (oo)\\_______',
      '            (__)\\       )\\/\\',
      '                ||----w |',
      '                ||     ||',
    ],
  },
  {
    cmd: () => 'echo $SHELL',
    lines: () => ['/bin/zsh'],
  },

  {
    cmd: () => 'cd ~/L-special-folder && ls -la',
    lines: () => [
      'total 24',
      'drwxr-xr-x   8 dev  staff   256 Sep 17 04:13 .',
      'drwxr-x---+ 42 dev  staff  1344 Sep 17 03:59 ..',
      'drwxr-xr-x  12 dev  staff   384 Sep 17 04:01 horse-tinder',
      'drwxr-xr-x   7 dev  staff   224 Sep 17 04:02 super-secret-folder',
      'drwxr-xr-x   5 dev  staff   160 Sep 17 04:03 definitely-not-production',
      '-rw-r--r--   1 dev  staff    42 Sep 17 04:13 todo-final-final-v2.md',
    ],
  },
  {
    cmd: () => 'uv init super-secret-folder',
    lines: () => [
      { txt: 'Initialized project `super-secret-folder`', cls: 'ok' },
      { txt: '$ cd super-secret-folder', cls: 'muted' },
      { txt: '$ uv add fastapi uvicorn', cls: 'muted' },
      'Resolved 14 packages in 184ms',
      'Installed 12 packages in 91ms',
      { txt: '+ fastapi==0.116.1', cls: 'ok' },
      { txt: '+ uvicorn==0.35.0', cls: 'ok' },
    ],
  },

  {
    cmd: () => 'claude',
    lines: () => {
      return [
        { txt: '  Claude Code', cls: 'accent' },
        { txt: '  ~/code/horse-tinder', cls: 'muted' },
        '',
        { txt: '› build tinder for horses', cls: 'prompt' },
        '',
        { txt: '  • Creating an equine-first dating experience', cls: 'info' },
        { txt: '  ✎ wrote src/components/HorseCard.tsx', cls: 'warn' },
        { txt: '  ✎ wrote src/lib/blockchain.ts', cls: 'warn' },
        '',
        { txt: '› why is there blockchain. delete it', cls: 'prompt' },
        '',
        { txt: '  ✎ wrote src/lib/blockchain-v2.ts', cls: 'warn' },
        '',
        { txt: '› you made a SECOND blockchain', cls: 'err' },
        { txt: '› /exit', cls: 'prompt' },
        { txt: '  Session ended', cls: 'muted' },
        '',
        { txt: 'dev@typesh:~/code$ codex', cls: 'prompt' },
        { txt: '  OpenAI Codex', cls: 'accent' },
        { txt: '  ~/code/horse-tinder', cls: 'muted' },
        '',
        { txt: '› fix the null pointer exception', cls: 'prompt' },
        '',
        { txt: '  • Read 7 files', cls: 'info' },
        { txt: '  ✎ catch (Exception ignored) {}', cls: 'warn' },
        '',
        { txt: '› THAT IS NOT A FIX', cls: 'err' },
        { txt: '› /quit', cls: 'prompt' },
        { txt: '  Goodbye. The exception remains.', cls: 'muted' },
        '',
        { txt: 'dev@typesh:~/code$ cd super-secret-folder', cls: 'prompt' },
        { txt: 'dev@typesh:~/code/super-secret-folder$ uv run main.py', cls: 'prompt' },
        { txt: 'it works on my machine', cls: 'ok' },
      ];
    },
  },

  // --
  {
    cmd: () => 'pnpm dev',
    lines: () => [
      '',
      {
        txt:
          '  VITE v5.4.2  ready in ' +
          (180 + Math.floor(Math.random() * 400)) +
          ' ms',
        cls: 'ok',
      },
      '',
      { txt: '  ➜  Local:   http://localhost:5173/', cls: 'info' },
      {
        txt:
          '  ➜  Network: http://192.168.1.' +
          (2 + Math.floor(Math.random() * 250)) +
          ':5173/',
        cls: 'muted',
      },
      { txt: '  ➜  press h + enter to show help', cls: 'muted' },
    ],
  },
  {
    cmd: () =>
      `ssh deploy@${randPick(['edge-1', 'db-2', 'worker-7'])}.typesh.dev`,
    lines: () => [
      {
        txt: 'Welcome to Ubuntu 24.04.1 LTS (GNU/Linux 6.8.0 aarch64)',
        cls: 'muted',
      },
      '',
      ' System load:  0.' +
        Math.floor(Math.random() * 90) +
        '    Processes:        ' +
        (90 + Math.floor(Math.random() * 60)),
      ' Memory usage: ' +
        (12 + Math.floor(Math.random() * 40)) +
        '%    Users logged in:  1',
      '',
      {
        txt:
          'Last login: ' +
          randPick(['Mon', 'Tue', 'Wed', 'Thu', 'Fri']) +
          ' from 10.0.0.' +
          Math.floor(Math.random() * 250),
        cls: 'muted',
      },
    ],
  },
  {
    cmd: () => `curl -s https://api.typesh.dev/v1/stats | jq`,
    lines: () => [
      '{',
      '  "users": ' + (12000 + Math.floor(Math.random() * 9000)) + ',',
      '  "avg_wpm": ' + (60 + Math.floor(Math.random() * 50)) + ',',
      '  "keystrokes_today": ' +
        (1 + Math.floor(Math.random() * 9)) +
        '.' +
        Math.floor(Math.random() * 9) +
        'M,',
      '  "status": "ok"',
      '}',
    ],
  },
  {
    cmd: () => 'make lint',
    lines: () => [
      { txt: 'eslint . --max-warnings 0', cls: 'muted' },
      { txt: '✓ 142 files, 0 problems', cls: 'ok' },
      { txt: 'prettier --check .', cls: 'muted' },
      { txt: 'All matched files use Prettier code style!', cls: 'ok' },
    ],
  },
];

type BufferLine = { kind: 'cmd' | 'out'; content: string; cls?: string | null };

export default function TerminalBg() {
  const [lines, setLines] = useState<BufferLine[]>([]);
  const [draft, setDraft] = useState(''); // the command mid typing, before enter
  const cmdQueueRef = useRef<Command[]>([]);
  const stoppedRef = useRef(false);

  // Shuffle instead of picking at random so you dont get the same command twice in a row
  const refill = () => {
    const next = [...COMMANDS];
    for (let i = next.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [next[i], next[j]] = [next[j], next[i]];
    }
    cmdQueueRef.current = next;
  };

  useEffect(() => {
    refill();
    const timeouts: ReturnType<typeof setTimeout>[] = [];
    const sched = (fn: () => void, ms: number) => {
      const t = setTimeout(fn, ms);
      timeouts.push(t);
      return t;
    };

    // Capped or the DOM just keeps growing forever
    const push = (line: BufferLine) => {
      setLines((prev) => {
        const next = [...prev, line];
        if (next.length > 80) next.splice(0, next.length - 80);
        return next;
      });
    };

    const runCommand = (cmdEntry: Command) => {
      if (stoppedRef.current) return;
      const cmdStr = cmdEntry.cmd();
      let i = 0;
      const typeNext = () => {
        if (stoppedRef.current) return;
        if (i >= cmdStr.length) {
          // enter: draft becomes a real line and the output starts
          sched(
            () => {
              push({ kind: 'cmd', content: cmdStr });
              setDraft('');
              const outLines = cmdEntry.lines();
              // Staggered, all at once looks wrong
              let delay = 60;
              outLines.forEach((ln) => {
                const txt = typeof ln === 'object' ? ln.txt : ln;
                const cls = typeof ln === 'object' ? ln.cls : null;
                sched(() => push({ kind: 'out', content: txt, cls }), delay);
                delay += 90 + Math.random() * 220;
              });
              sched(nextCommand, delay + 800 + Math.random() * 1200);
            },
            250 + Math.random() * 400,
          );
          return;
        }
        setDraft(cmdStr.slice(0, i + 1));
        i++;
        sched(typeNext, 35 + Math.random() * 75);
      };
      // Little pause before it starts typing
      sched(typeNext, 200 + Math.random() * 500);
    };

    const nextCommand = () => {
      if (stoppedRef.current) return;
      if (cmdQueueRef.current.length === 0) refill();
      const c = cmdQueueRef.current.shift();
      if (c) runCommand(c);
    };

    // So it isnt empty on the first frame
    push({
      kind: 'out',
      content: 'Last login: Thu Jan  1 00:00:42 on ttys042',
      cls: 'muted',
    });
    sched(nextCommand, 400);

    return () => {
      stoppedRef.current = true;
      timeouts.forEach(clearTimeout);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const scrollerRef = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const el = scrollerRef.current;
    if (el) el.scrollTop = el.scrollHeight;
  }, [lines, draft]);

  return (
    <div className="term-root">
      <div className="term-scroller" ref={scrollerRef}>
        {lines.map((ln, i) => {
          if (ln.kind === 'cmd') {
            return (
              <div className="term-line term-cmd" key={i}>
                <span className="term-prompt">dev@typesh</span>
                <span className="term-sep">:</span>
                <span className="term-path">~/code</span>
                <span className="term-sep">$ </span>
                <span className="term-cmd-text">{ln.content}</span>
              </div>
            );
          }
          return (
            <div
              className={`term-line term-out ${ln.cls ? 'term-' + ln.cls : ''}`}
              key={i}>
              {ln.content}
            </div>
          );
        })}
        <div className="term-line term-active">
          <span className="term-prompt">dev@typesh</span>
          <span className="term-sep">:</span>
          <span className="term-path">~/code</span>
          <span className="term-sep">$ </span>
          <span className="term-cmd-text">{draft}</span>
          <span className="term-cursor" />
        </div>
      </div>
    </div>
  );
}

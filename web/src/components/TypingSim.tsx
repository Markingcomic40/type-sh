import { useEffect, useRef, useState, type ReactNode } from 'react';

// Fake typing on one line: [typed words] [current word] [upcoming words]
// Current word stays near the middle and everything slides left as we go.
// We keep how each past word was actually typed so the red stays red while it scrolls off.
// Client only, timers + Math.random everywhere so it cant be SSRd

// Lowercase only, looks better in mono
const CORPUS: string[] = (
  'the quick brown fox jumps over the lazy dog while the compiler chews ' +
  'through another pile of unfinished thoughts and half remembered keywords ' +
  'lorem ipsum dolor sit amet consectetur adipiscing elit sed do eiusmod ' +
  'tempor incididunt ut labore et dolore magna aliqua duis aute irure dolor ' +
  'in reprehenderit voluptate velit esse cillum eu fugiat nulla pariatur ' +
  'excepteur sint occaecat cupidatat non proident sunt in culpa qui officia ' +
  'deserunt mollit anim id est laborum at vero eos et accusamus et iusto ' +
  'odio dignissimos ducimus qui blanditiis praesentium voluptatum deleniti ' +
  'atque corrupti quos dolores et quas molestias excepturi sint occaecati ' +
  'cupiditate non provident similique sunt in culpa qui officia deserunt'
).split(/\s+/);

const HISTORY_WIN = 20; // how many past words we remember char by char

// Typos should land on a key thats actually next to the right one, otherwise it reads as random garbage
const NEIGHBORS: Record<string, string> = {
  a: 'sqwz',
  b: 'vghn',
  c: 'xdfv',
  d: 'serfcx',
  e: 'wsdr',
  f: 'drtgvc',
  g: 'ftyhbv',
  h: 'gyujnb',
  i: 'ujko',
  j: 'huikmn',
  k: 'jiolm',
  l: 'kop',
  m: 'njk',
  n: 'bhjm',
  o: 'iklp',
  p: 'ol',
  q: 'wa',
  r: 'edft',
  s: 'awedxz',
  t: 'rfgy',
  u: 'yhji',
  v: 'cfgb',
  w: 'qase',
  x: 'zsdc',
  y: 'tghu',
  z: 'asx',
};

const wrongChar = (c: string): string => {
  const opts = NEIGHBORS[c.toLowerCase()];
  if (!opts) return 'x';
  return opts[Math.floor(Math.random() * opts.length)];
};

type KeyEvent = { type: 'add'; char: string } | { type: 'back' };

// Plan out every keystroke for a word upfront, the loop just replays them
function planWord(word: string, mistakeRate: number): KeyEvent[] {
  const events: KeyEvent[] = [];
  let i = 0;
  let safety = 0;
  while (i < word.length && safety++ < 60) {
    const correctChar = word[i];
    const willMistake = Math.random() < mistakeRate;
    if (willMistake) {
      const bad = wrongChar(correctChar);
      events.push({ type: 'add', char: bad });
      // Sometimes you notice, sometimes you dont
      if (Math.random() < 0.7) {
        events.push({ type: 'back' });
        events.push({ type: 'add', char: correctChar });
        i++;
      } else {
        i++;
      }
    } else {
      events.push({ type: 'add', char: correctChar });
      i++;
    }
  }
  // Every now and then overshoot the word and walk it back
  if (Math.random() < 0.08) {
    const extra = wrongChar(word[word.length - 1] || 'a');
    events.push({ type: 'add', char: extra });
    events.push({ type: 'back' });
  }
  return events;
}

type CharState = 'pending' | 'correct' | 'wrong' | 'overflow';
type CharCell = { ch: string; state: CharState };

function charsFor(
  target: string,
  typed: string,
  includePending: boolean,
): CharCell[] {
  const out: CharCell[] = [];
  const len = Math.max(target.length, typed.length);
  for (let i = 0; i < len; i++) {
    const t = typed[i];
    const want = target[i];
    if (t == null) {
      if (includePending) out.push({ ch: want, state: 'pending' });
    } else if (i >= target.length) {
      out.push({ ch: t, state: 'overflow' });
    } else if (t === want) {
      out.push({ ch: t, state: 'correct' });
    } else {
      out.push({ ch: t, state: 'wrong' });
    }
  }
  return out;
}

type TypingSimProps = {
  targetWpm?: number;
  mistakeRate?: number; // 0..1
  oscAmp?: number;
  oscPeriod?: number;
};

type HistoryEntry = { word: string; typed: string };

export default function TypingSim({
  targetWpm = 140,
  mistakeRate = 0.08,
  oscAmp = 22,
  oscPeriod = 18,
}: TypingSimProps) {
  const [wordIdx, setWordIdx] = useState(0);
  const [typed, setTyped] = useState('');
  const [wpm, setWpm] = useState(targetWpm);
  const [history, setHistory] = useState<HistoryEntry[]>([]);

  const queueRef = useRef<KeyEvent[]>([]);
  const tickRef = useRef(0);
  // Mirror of typed, otherwise the closure in the loop hands us a stale value
  // right when we need the final one to push into history
  const typedRef = useRef('');
  typedRef.current = typed;

  // New word, new plan
  useEffect(() => {
    queueRef.current = planWord(CORPUS[wordIdx % CORPUS.length], mistakeRate);
    tickRef.current = 0;
  }, [wordIdx, mistakeRate]);

  // Nobody types at a constant speed so the wpm rides a sine
  useEffect(() => {
    let cancelled = false;
    const t0 = performance.now();

    const step = () => {
      if (cancelled) return;
      const now = performance.now();
      const elapsed = (now - t0) / 1000;
      // Slow wave for the general pace, fast one on top for nervous fingers
      const omegaSlow = (Math.PI * 2) / Math.max(2, oscPeriod);
      const oscillated =
        targetWpm +
        Math.sin(elapsed * omegaSlow) * oscAmp +
        Math.sin(elapsed * omegaSlow * 4.8) * (oscAmp * 0.27);
      const liveWpm = Math.max(40, oscillated);
      setWpm(liveWpm);

      const cps = (liveWpm * 5) / 60;
      const baseMs = 1000 / cps;
      const jitter = baseMs * (0.55 + Math.random() * 0.9);

      const q = queueRef.current;
      if (tickRef.current >= q.length) {
        // Word done, save how it went before moving on
        setTimeout(() => {
          if (cancelled) return;
          const finishedWord = CORPUS[wordIdx % CORPUS.length];
          const finishedTyped = typedRef.current;
          setHistory((h) => {
            const next = [...h, { word: finishedWord, typed: finishedTyped }];
            if (next.length > HISTORY_WIN) {
              next.splice(0, next.length - HISTORY_WIN);
            }
            return next;
          });
          setTyped('');
          setWordIdx((w) => w + 1);
        }, baseMs * 1.4);
        return;
      }

      const ev = q[tickRef.current++];
      if (ev.type === 'add') {
        setTyped((s) => s + ev.char);
      } else {
        setTyped((s) => s.slice(0, -1));
      }
      setTimeout(step, jitter);
    };

    const initial = setTimeout(step, 80);
    return () => {
      cancelled = true;
      clearTimeout(initial);
    };
  }, [wordIdx, targetWpm, oscAmp, oscPeriod]);

  // --
  const target = CORPUS[wordIdx % CORPUS.length];

  const beforeNodes: ReactNode[] = [];
  history.forEach((h, hi) => {
    const states = charsFor(h.word, h.typed, false);
    states.forEach((c, ci) => {
      beforeNodes.push(
        <span
          key={`b-${hi}-${ci}`}
          className={`ts-c ts-c-${c.state} ts-c-done`}>
          {c.ch}
        </span>,
      );
    });
    beforeNodes.push(
      <span key={`bs-${hi}`} className="ts-c ts-c-correct ts-c-done ts-c-space">
        {' '}
      </span>,
    );
  });

  // Spaces have to be their own spans or they collapse
  const afterNodes: ReactNode[] = [];
  for (let i = wordIdx + 1; i < wordIdx + 24; i++) {
    afterNodes.push(
      <span key={`as-${i}`} className="ts-c ts-c-pending ts-c-space">
        {' '}
      </span>,
    );
    const w = CORPUS[i % CORPUS.length];
    for (let ci = 0; ci < w.length; ci++) {
      afterNodes.push(
        <span key={`a-${i}-${ci}`} className="ts-c ts-c-pending">
          {w[ci]}
        </span>,
      );
    }
  }

  const currentChars = charsFor(target, typed, true);

  return (
    <div className="ts-wrap">
      <div className="ts-line">
        <span className="ts-before">{beforeNodes}</span>
        <span className="ts-current">
          {currentChars.map((c, i) => (
            <span key={i} className={`ts-c ts-c-${c.state}`}>
              {c.ch}
            </span>
          ))}
          <span className="ts-caret" />
        </span>
        <span className="ts-after">{afterNodes}</span>
      </div>
      <div className="ts-meta">
        <span className="ts-wpm-num">{Math.round(wpm)}</span>
        <span className="ts-wpm-lbl">wpm</span>
      </div>
    </div>
  );
}

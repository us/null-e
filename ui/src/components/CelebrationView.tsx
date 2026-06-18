import { useEffect, useRef, useCallback, useMemo, useState } from 'react';
import { motion, useReducedMotion } from 'framer-motion';
import {
  PartyPopper,
  RotateCcw,
  AlertTriangle,
  ChevronDown,
  ChevronRight,
  ExternalLink,
  Trash2,
  ShieldAlert,
  Lock,
  Copy,
  Check,
} from 'lucide-react';
import { commands, type CleanFailureDto } from '@/lib/tauri';
import { useCleanStore } from '@/stores/clean-store';
import { useScanStore } from '@/stores/scan-store';
import { useSystemStore } from '@/stores/system-store';
import { useUiStore } from '@/stores/ui-store';
import { AnimatedCounter } from '@/components/shared/AnimatedCounter';
import { formatSize } from '@/lib/format';

/** Human-readable, actionable copy per failure category (matches backend DeleteFailureClass). */
const CATEGORY_META: Record<
  string,
  { title: string; description: string; action?: 'fda' }
> = {
  fda: {
    title: 'Need Full Disk Access',
    description:
      'macOS privacy protection blocked these. Grant Full Disk Access to null-e, relaunch, then retry.',
    action: 'fda',
  },
  needs_admin: {
    title: 'Need administrator rights',
    description:
      'These are owned by the system. null-e does not auto-elevate — remove them manually with admin rights if needed.',
  },
  sip_protected: {
    title: 'Protected by macOS',
    description: 'These are SIP-protected and cannot be removed by any app.',
  },
  read_only: {
    title: 'On a read-only volume',
    description: 'These live on the sealed system volume and cannot be deleted.',
  },
  immutable: {
    title: 'Locked files',
    description: 'These have an immutable flag that could not be cleared.',
  },
  busy: {
    title: 'In use',
    description: 'These were busy or being rewritten. Close related apps and try again.',
  },
  refused: {
    title: 'Protected locations (skipped)',
    description: 'These are system aggregate locations null-e refuses to bulk-delete for safety.',
  },
  other: {
    title: 'Could not be removed',
    description: 'These failed for an unexpected reason. See details below.',
  },
};

function categoryOf(f: CleanFailureDto): string {
  if (f.category && CATEGORY_META[f.category]) return f.category;
  return f.is_tcc ? 'fda' : 'other';
}

export function CelebrationView() {
  const summary = useCleanStore((s) => s.summary);
  const reduceMotion = useReducedMotion();
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [showFailures, setShowFailures] = useState(true);
  const [trashEmptied, setTrashEmptied] = useState(false);
  const [trashFreed, setTrashFreed] = useState(0);
  const [emptyingTrash, setEmptyingTrash] = useState(false);
  const [copiedLog, setCopiedLog] = useState(false);

  const usedTrash = summary?.used_trash ?? false;
  const pending = summary?.bytes_pending ?? 0;
  const hasFailed = summary != null && summary.failed > 0;

  // Bytes actually reclaimed NOW. `summary.bytes_freed` already includes permanent deletes AND
  // side-channel reclaim (permanently-deleted caches + snapshot-thinning actions) merged in — those
  // free space immediately even in Trash mode. Trashed bytes only count once the Trash is emptied.
  const reclaimedNow = useMemo(() => {
    if (!summary) return 0;
    const immediate = summary.bytes_freed;
    const fromTrash = usedTrash && trashEmptied ? trashFreed : 0;
    return immediate + fromTrash;
  }, [summary, usedTrash, trashEmptied, trashFreed]);

  const celebrate = !hasFailed && reclaimedNow > 0;

  const visibleFailures = useMemo(
    () => summary?.failures.slice(0, 10) ?? [],
    [summary]
  );
  const hiddenFailureCount = Math.max(0, (summary?.failures.length ?? 0) - visibleFailures.length);

  // Group failures by actionable category.
  const failureGroups = useMemo(() => {
    const map = new Map<string, number>();
    for (const f of summary?.failures ?? []) {
      const cat = categoryOf(f);
      map.set(cat, (map.get(cat) ?? 0) + 1);
    }
    return Array.from(map.entries());
  }, [summary]);

  // Confetti effect — only when something was actually reclaimed, and never when the user has asked
  // for reduced motion (the 80-particle animation is a vestibular trigger).
  useEffect(() => {
    if (!celebrate || reduceMotion) return;

    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const resize = () => {
      canvas.width = window.innerWidth;
      canvas.height = window.innerHeight;
    };
    resize();
    window.addEventListener('resize', resize);

    const colors = ['#9DA7EA', '#F4B6C3', '#D7FFE9', '#EEBE55', '#EC5E52', '#DEDDFC'];
    const particles: Array<{
      x: number; y: number; vx: number; vy: number;
      size: number; color: string; rotation: number; rotSpeed: number; life: number;
    }> = [];

    for (let i = 0; i < 80; i++) {
      particles.push({
        x: canvas.width / 2 + (Math.random() - 0.5) * 200,
        y: canvas.height / 2 - 50,
        vx: (Math.random() - 0.5) * 12,
        vy: Math.random() * -12 - 2,
        size: Math.random() * 8 + 4,
        color: colors[Math.floor(Math.random() * colors.length)],
        rotation: Math.random() * 360,
        rotSpeed: (Math.random() - 0.5) * 10,
        life: 1,
      });
    }

    let animFrame: number;
    const animate = () => {
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      let alive = false;
      for (const p of particles) {
        if (p.life <= 0) continue;
        alive = true;
        p.x += p.vx; p.y += p.vy; p.vy += 0.3;
        p.rotation += p.rotSpeed; p.life -= 0.008; p.vx *= 0.99;
        ctx.save();
        ctx.translate(p.x, p.y);
        ctx.rotate((p.rotation * Math.PI) / 180);
        ctx.globalAlpha = Math.max(0, p.life);
        ctx.fillStyle = p.color;
        ctx.fillRect(-p.size / 2, -p.size / 2, p.size, p.size * 0.6);
        ctx.restore();
      }
      if (alive) animFrame = requestAnimationFrame(animate);
    };
    animate();

    return () => {
      cancelAnimationFrame(animFrame);
      window.removeEventListener('resize', resize);
    };
  }, [celebrate, reduceMotion]);

  const handleNewScan = useCallback(() => {
    useCleanStore.getState().reset();
    useScanStore.getState().reset();
    useUiStore.getState().setAppState('welcome');
  }, []);

  const handleBackToResults = useCallback(() => {
    useCleanStore.getState().reset();
    useUiStore.getState().setAppState('results');
  }, []);

  const handleEmptyTrash = useCallback(async () => {
    setEmptyingTrash(true);
    try {
      const freed = await commands.emptyTrash();
      setTrashFreed(freed);
      setTrashEmptied(true);
      // Refresh disk/system figures now that space is actually back.
      await useSystemStore.getState().detectSystem();
    } catch (err) {
      console.error('Failed to empty Trash:', err);
    } finally {
      setEmptyingTrash(false);
    }
  }, []);

  const handleCopyLog = useCallback(async () => {
    if (!summary) return;
    const lines = summary.failures.map(
      (f) => `[${categoryOf(f)}] ${f.path}\n    ${f.reason}`
    );
    const text = `null-e failure log (${summary.failures.length} item(s))\n\n${lines.join('\n')}`;
    try {
      await navigator.clipboard.writeText(text);
      setCopiedLog(true);
      setTimeout(() => setCopiedLog(false), 2000);
    } catch (err) {
      console.error('Failed to copy failure log:', err);
    }
  }, [summary]);

  if (!summary) return null;

  const openFdaSettings = async () => {
    try {
      await commands.openPrivacySettings();
    } catch (err) {
      console.error('Failed to open Full Disk Access settings:', err);
    }
  };

  // Headline framing.
  const showTrashPending = usedTrash && !trashEmptied && pending > 0;
  const headlineTitle = hasFailed
    ? 'Completed with errors'
    : showTrashPending
      ? 'Moved to Trash'
      : 'Space reclaimed!';
  const headlineValue = showTrashPending ? pending : reclaimedNow;

  return (
    <div className="relative flex flex-col items-center justify-center h-full">
      {celebrate && !reduceMotion && <canvas ref={canvasRef} className="confetti-container" />}

      <motion.div
        initial={{ scale: 0.8, opacity: 0 }}
        animate={{ scale: 1, opacity: 1 }}
        transition={{ duration: 0.5, type: 'spring', bounce: 0.4 }}
        className="flex flex-col items-center gap-6 z-10"
      >
        {/* Icon — success, trash-pending, or warning */}
        <div className={`flex items-center justify-center w-16 h-16 rounded-full ${
          hasFailed ? 'bg-[var(--color-warning)]/10'
            : showTrashPending ? 'bg-[var(--color-primary)]/10'
            : 'bg-[var(--color-safe)]/10'
        }`}>
          {hasFailed ? (
            <AlertTriangle size={32} className="text-[var(--color-warning)]" />
          ) : showTrashPending ? (
            <Trash2 size={32} className="text-[var(--color-primary)]" />
          ) : (
            <PartyPopper size={32} className="text-[var(--color-safe)]" />
          )}
        </div>

        <div className="text-center">
          <h2 className="text-lg font-semibold text-[var(--color-text)] mb-2">
            {headlineTitle}
          </h2>
          <p className="display text-5xl text-[var(--color-text)] tabular-nums">
            <AnimatedCounter
              value={headlineValue}
              format={formatSize}
              duration={1.5}
            />
          </p>
          {showTrashPending && (
            <p className="mt-2 text-sm text-[var(--color-text-secondary)]">
              Empty the Trash to reclaim this space — trashed items aren’t freed until you do.
            </p>
          )}
          {showTrashPending && summary.bytes_freed > 0 && (
            <p className="mt-1 text-sm text-[var(--color-safe)]">
              {formatSize(summary.bytes_freed)} already reclaimed now (caches / snapshots).
            </p>
          )}
          {usedTrash && trashEmptied && (
            <p className="mt-2 text-sm text-[var(--color-safe)]">
              Trash emptied — {formatSize(trashFreed)} reclaimed.
            </p>
          )}
        </div>

        {/* Empty Trash CTA */}
        {showTrashPending && (
          <button
            onClick={() => { void handleEmptyTrash(); }}
            disabled={emptyingTrash}
            className="inline-flex items-center gap-2 rounded-full bg-[var(--color-primary)] px-5 py-2.5 text-sm font-medium text-white transition-colors hover:bg-[var(--color-primary-hover)] disabled:opacity-60"
          >
            <Trash2 size={15} />
            {emptyingTrash ? 'Emptying…' : 'Empty Trash now'}
          </button>
        )}

        {/* Stats — bento tint cards */}
        <div className="flex gap-3">
          <div className="tint-sky rounded-[18px] px-5 py-3 text-center min-w-[88px]">
            <p className="display text-xl tabular-nums">{summary.succeeded}</p>
            <p className="text-[11px] opacity-70 lowercase">
              {usedTrash ? 'trashed' : 'cleaned'}
            </p>
          </div>
          {summary.failed > 0 && (
            <div className="tint-peach rounded-[18px] px-5 py-3 text-center min-w-[88px]">
              <p className="display text-xl tabular-nums">{summary.failed}</p>
              <p className="text-[11px] opacity-70 lowercase">failed</p>
            </div>
          )}
          <div className="tint-lav rounded-[18px] px-5 py-3 text-center min-w-[88px]">
            <p className="display text-base">{summary.method_label}</p>
            <p className="text-[11px] opacity-70 lowercase">method</p>
          </div>
        </div>

        {/* Failure summary grouped by actionable cause */}
        {failureGroups.length > 0 && (
          <div className="w-full max-w-2xl space-y-3">
            {failureGroups.map(([cat, count]) => {
              const meta = CATEGORY_META[cat] ?? CATEGORY_META.other;
              const Icon = cat === 'fda' ? AlertTriangle : cat === 'needs_admin' ? ShieldAlert : Lock;
              const accent = cat === 'fda' ? 'amber' : 'slate';
              return (
                <div
                  key={cat}
                  className={`rounded-2xl border px-5 py-4 ${
                    accent === 'amber'
                      ? 'border-[var(--color-warning-border)] bg-[var(--color-warning-surface)]'
                      : 'border-[var(--color-border)] bg-[var(--color-surface)]/80'
                  }`}
                >
                  <div className="flex items-start gap-3">
                    <Icon size={18} className={`mt-0.5 ${accent === 'amber' ? 'text-[var(--color-warning-text)]' : 'text-[var(--color-text-muted)]'}`} />
                    <div className="flex-1">
                      <h3 className="text-sm font-semibold text-[var(--color-text)]">
                        {meta.title} · {count} item{count === 1 ? '' : 's'}
                      </h3>
                      <p className="mt-1 text-sm text-[var(--color-text-secondary)]">
                        {meta.description}
                      </p>
                    </div>
                    {meta.action === 'fda' && (
                      <button
                        onClick={() => { void openFdaSettings(); }}
                        className="inline-flex items-center gap-2 rounded-xl bg-[var(--color-warning)] px-3 py-2 text-sm font-medium text-[var(--color-warning-strong-text)] transition-colors hover:bg-[var(--color-warning-hover)]"
                      >
                        <ExternalLink size={14} />
                        Open Settings
                      </button>
                    )}
                  </div>
                </div>
              );
            })}
          </div>
        )}

        {/* Detailed failure list */}
        {summary.failures.length > 0 && (
          <div className="w-full max-w-2xl rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)]/80">
            <div className="flex w-full items-center justify-between px-5 py-4">
              <button
                onClick={() => setShowFailures((value) => !value)}
                aria-expanded={showFailures}
                className="flex flex-1 items-center justify-between text-left rounded-lg focus-visible:ring-2 focus-visible:ring-[var(--color-primary)] outline-none"
              >
                <div>
                  <p className="text-sm font-semibold text-[var(--color-text)]">
                    Failure details
                  </p>
                  <p className="mt-1 text-xs text-[var(--color-text-muted)]">
                    {summary.failed} item{summary.failed === 1 ? '' : 's'} could not be cleaned
                  </p>
                </div>
              </button>
              <div className="flex items-center gap-2 pl-3">
                <button
                  onClick={() => { void handleCopyLog(); }}
                  className="inline-flex items-center gap-1.5 rounded-lg border border-[var(--color-border)] px-2.5 py-1.5 text-xs text-[var(--color-text-secondary)] transition-colors hover:bg-[var(--color-surface-hover)] focus-visible:ring-2 focus-visible:ring-[var(--color-primary)] outline-none"
                  title="Copy the failure log to share or diagnose"
                >
                  {copiedLog ? <Check size={13} /> : <Copy size={13} />}
                  {copiedLog ? 'Copied' : 'Copy log'}
                </button>
                <button
                  onClick={() => setShowFailures((value) => !value)}
                  aria-expanded={showFailures}
                  aria-label={showFailures ? 'Collapse failure details' : 'Expand failure details'}
                  className="rounded-lg p-1 focus-visible:ring-2 focus-visible:ring-[var(--color-primary)] outline-none"
                >
                  {showFailures ? (
                    <ChevronDown size={18} className="text-[var(--color-text-muted)]" />
                  ) : (
                    <ChevronRight size={18} className="text-[var(--color-text-muted)]" />
                  )}
                </button>
              </div>
            </div>

            {showFailures && (
              <div className="border-t border-[var(--color-border)] px-5 py-4">
                <div className="space-y-3">
                  {visibleFailures.map((failure) => (
                    <div
                      key={`${failure.path}:${failure.reason}`}
                      className="rounded-xl bg-[var(--color-bg)] px-4 py-3"
                    >
                      <p className="break-all text-sm font-medium text-[var(--color-text)]">
                        {failure.path}
                      </p>
                      <p className="mt-1 text-sm text-[var(--color-text-secondary)]">
                        {failure.reason}
                      </p>
                    </div>
                  ))}
                </div>
                {hiddenFailureCount > 0 && (
                  <p className="mt-3 text-xs text-[var(--color-text-muted)]">
                    and {hiddenFailureCount} more
                  </p>
                )}
              </div>
            )}
          </div>
        )}

        {/* Actions */}
        <div className="flex gap-3 mt-4">
          <button
            onClick={handleBackToResults}
            className="flex items-center gap-2 px-5 py-2.5 rounded-full text-sm text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-hover)] transition-colors border border-[var(--color-border)] focus-visible:ring-2 focus-visible:ring-[var(--color-primary)] outline-none"
          >
            Back to results
          </button>
          <button
            onClick={handleNewScan}
            className="flex items-center gap-2 px-5 py-2.5 rounded-full text-sm font-medium text-white bg-[var(--color-primary)] hover:bg-[var(--color-primary-hover)] transition-colors focus-visible:ring-2 focus-visible:ring-white outline-none"
          >
            <RotateCcw size={14} />
            New scan
          </button>
        </div>
      </motion.div>
    </div>
  );
}

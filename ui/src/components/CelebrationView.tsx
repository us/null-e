import { useEffect, useRef, useCallback } from 'react';
import { motion } from 'framer-motion';
import { PartyPopper, RotateCcw, AlertTriangle } from 'lucide-react';
import { useCleanStore } from '@/stores/clean-store';
import { useScanStore } from '@/stores/scan-store';
import { useUiStore } from '@/stores/ui-store';
import { AnimatedCounter } from '@/components/shared/AnimatedCounter';
import { formatSize } from '@/lib/format';

export function CelebrationView() {
  const summary = useCleanStore((s) => s.summary);
  const canvasRef = useRef<HTMLCanvasElement>(null);

  const hasFailed = summary != null && summary.failed > 0;
  const isFullSuccess = summary != null && summary.failed === 0;

  // Confetti effect — only on full success
  useEffect(() => {
    if (!isFullSuccess) return;

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

    const colors = ['#6366F1', '#10B981', '#F59E0B', '#EF4444', '#8B5CF6', '#EC4899'];
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
  }, [isFullSuccess]);

  const handleNewScan = useCallback(() => {
    useCleanStore.getState().reset();
    useScanStore.getState().reset();
    useUiStore.getState().setAppState('welcome');
  }, []);

  const handleBackToResults = useCallback(() => {
    useCleanStore.getState().reset();
    useUiStore.getState().setAppState('results');
  }, []);

  if (!summary) return null;

  return (
    <div className="relative flex flex-col items-center justify-center h-full">
      {isFullSuccess && <canvas ref={canvasRef} className="confetti-container" />}

      <motion.div
        initial={{ scale: 0.8, opacity: 0 }}
        animate={{ scale: 1, opacity: 1 }}
        transition={{ duration: 0.5, type: 'spring', bounce: 0.4 }}
        className="flex flex-col items-center gap-6 z-10"
      >
        {/* Icon — success or warning */}
        <div className={`flex items-center justify-center w-16 h-16 rounded-full ${
          hasFailed ? 'bg-[var(--color-warning)]/10' : 'bg-[var(--color-safe)]/10'
        }`}>
          {hasFailed ? (
            <AlertTriangle size={32} className="text-[var(--color-warning)]" />
          ) : (
            <PartyPopper size={32} className="text-[var(--color-safe)]" />
          )}
        </div>

        <div className="text-center">
          <h2 className="text-lg font-semibold text-[var(--color-text)] mb-2">
            {hasFailed ? 'Completed with errors' : 'Space reclaimed!'}
          </h2>
          <p className="text-4xl font-bold text-[var(--color-text)] tabular-nums">
            <AnimatedCounter
              value={summary.bytes_freed}
              format={formatSize}
              duration={1.5}
            />
          </p>
        </div>

        {/* Stats */}
        <div className="flex gap-8 text-center">
          <div>
            <p className="text-xl font-bold text-[var(--color-text)] tabular-nums">
              {summary.succeeded}
            </p>
            <p className="text-xs text-[var(--color-text-muted)]">Cleaned</p>
          </div>
          {summary.failed > 0 && (
            <div>
              <p className="text-xl font-bold text-[var(--color-danger)] tabular-nums">
                {summary.failed}
              </p>
              <p className="text-xs text-[var(--color-text-muted)]">Failed</p>
            </div>
          )}
          <div>
            <p className="text-xl font-bold text-[var(--color-text)]">
              {summary.used_trash ? 'Trash' : 'Deleted'}
            </p>
            <p className="text-xs text-[var(--color-text-muted)]">Method</p>
          </div>
        </div>

        {/* Actions */}
        <div className="flex gap-3 mt-4">
          <button
            onClick={handleBackToResults}
            className="flex items-center gap-2 px-5 py-2.5 rounded-xl text-sm text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-hover)] transition-colors border border-[var(--color-border)] focus-visible:ring-2 focus-visible:ring-[var(--color-primary)] outline-none"
          >
            Back to results
          </button>
          <button
            onClick={handleNewScan}
            className="flex items-center gap-2 px-5 py-2.5 rounded-xl text-sm font-medium text-white bg-[var(--color-primary)] hover:bg-[var(--color-primary-hover)] transition-colors focus-visible:ring-2 focus-visible:ring-white outline-none"
          >
            <RotateCcw size={14} />
            New scan
          </button>
        </div>
      </motion.div>
    </div>
  );
}

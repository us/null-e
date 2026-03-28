import { useState, useEffect, useRef } from 'react';
import { motion } from 'framer-motion';
import { FolderSearch, Package, HardDrive, Clock, X } from 'lucide-react';
import { useScanStore } from '@/stores/scan-store';
import { useUiStore } from '@/stores/ui-store';
import { formatSize, formatNumber } from '@/lib/format';

export function ScanningView() {
  const { progress, cancelScan } = useScanStore();
  const [elapsed, setElapsed] = useState(0);
  const startRef = useRef(Date.now());

  useEffect(() => {
    startRef.current = Date.now();
    const timer = setInterval(() => {
      setElapsed(Math.floor((Date.now() - startRef.current) / 1000));
    }, 1000);
    return () => clearInterval(timer);
  }, []);

  const handleCancel = () => {
    cancelScan();
    useUiStore.getState().setAppState('welcome');
  };

  const formatElapsed = (secs: number) => {
    if (secs < 60) return `${secs}s`;
    const m = Math.floor(secs / 60);
    const s = secs % 60;
    return `${m}m ${s}s`;
  };

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      transition={{ duration: 0.3 }}
      className="flex flex-col items-center justify-center h-full gap-8 px-6"
    >
      {/* Title */}
      <div className="flex flex-col items-center gap-2">
        <motion.img
          src="/logo.png"
          alt="null-e"
          width={96}
          height={96}
          className="rounded-2xl"
          animate={{
            rotate: [0, -15, 15, -10, 10, -5, 360],
            scale: [1, 1.08, 0.95, 1.05, 0.98, 1.02, 1],
          }}
          transition={{
            duration: 2.5,
            repeat: Infinity,
            ease: [0.4, 0, 0.2, 1],
          }}
        />
        <span className="text-lg font-semibold text-[var(--color-text)]">
          Scanning...
        </span>
      </div>

      {/* Indeterminate progress bar */}
      <div className="w-80">
        <div className="w-full h-2.5 rounded-full bg-[var(--color-bg-tertiary)] overflow-hidden relative">
          <motion.div
            className="absolute h-full w-1/3 rounded-full bg-gradient-to-r from-transparent via-[var(--color-primary)] to-transparent"
            animate={{ x: ['-100%', '400%'] }}
            transition={{
              duration: 1.5,
              repeat: Infinity,
              ease: 'easeInOut',
            }}
          />
        </div>
      </div>

      {/* Live stats */}
      {progress && (
        <div className="grid grid-cols-2 gap-x-10 gap-y-3">
          <StatItem
            icon={<FolderSearch size={14} />}
            label="Directories"
            value={formatNumber(progress.directories_scanned)}
          />
          <StatItem
            icon={<Package size={14} />}
            label="Projects"
            value={formatNumber(progress.projects_found)}
          />
          <StatItem
            icon={<HardDrive size={14} />}
            label="Found"
            value={formatSize(progress.total_size_found)}
          />
          <StatItem
            icon={<Clock size={14} />}
            label="Elapsed"
            value={formatElapsed(elapsed)}
          />
        </div>
      )}

      {/* Current path */}
      {progress?.current_path && (
        <p className="text-xs text-[var(--color-text-muted)] max-w-md truncate text-center">
          {progress.current_path}
        </p>
      )}

      {/* Cancel */}
      <button
        onClick={handleCancel}
        className="flex items-center gap-2 px-5 py-2 rounded-xl text-sm text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-hover)] transition-colors border border-[var(--color-border)]"
      >
        <X size={14} />
        Cancel
      </button>
    </motion.div>
  );
}

function StatItem({
  icon,
  label,
  value,
}: {
  icon: React.ReactNode;
  label: string;
  value: string;
}) {
  return (
    <div className="flex items-center gap-2">
      <span className="text-[var(--color-primary)]">{icon}</span>
      <div>
        <p className="text-xs text-[var(--color-text-muted)]">{label}</p>
        <p className="text-sm font-semibold text-[var(--color-text)] tabular-nums">
          {value}
        </p>
      </div>
    </div>
  );
}

import { motion } from 'framer-motion';
import { useScanStore } from '@/stores/scan-store';
import { formatSize, formatNumber } from '@/lib/format';

export function ScanProgress() {
  const { isScanning, progress } = useScanStore();

  if (!isScanning || !progress) return null;

  const progressPercent = progress.is_complete ? 100 : undefined;

  return (
    <div className="p-4 rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] space-y-3">
      <div className="flex items-center justify-between text-sm">
        <span className="text-[var(--color-text)] font-medium">Scanning...</span>
        <div className="flex items-center gap-4 text-[var(--color-text-secondary)]">
          <span>{formatNumber(progress.directories_scanned)} dirs</span>
          <span>{formatNumber(progress.projects_found)} projects</span>
          <span>{formatSize(progress.total_size_found)}</span>
        </div>
      </div>

      {/* Progress bar */}
      <div className="h-1.5 rounded-full bg-[var(--color-bg-tertiary)] overflow-hidden">
        {progressPercent !== undefined ? (
          <motion.div
            className="h-full rounded-full bg-[var(--color-primary)]"
            initial={{ width: 0 }}
            animate={{ width: `${progressPercent}%` }}
            transition={{ duration: 0.3 }}
          />
        ) : (
          <motion.div
            className="h-full w-1/3 rounded-full bg-[var(--color-primary)]"
            animate={{ x: ['-100%', '400%'] }}
            transition={{ repeat: Infinity, duration: 1.5, ease: 'easeInOut' }}
          />
        )}
      </div>

      {/* Current path */}
      <p className="text-xs text-[var(--color-text-muted)] truncate">
        {progress.current_path}
      </p>
    </div>
  );
}

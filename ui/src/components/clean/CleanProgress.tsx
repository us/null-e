import { motion } from 'framer-motion';
import { useCleanStore } from '@/stores/clean-store';
import { formatSize } from '@/lib/format';

export function CleanProgress() {
  const { isCleaning, progress } = useCleanStore();

  if (!isCleaning || !progress) return null;

  const percent =
    progress.total_items > 0
      ? (progress.completed_items / progress.total_items) * 100
      : 0;

  return (
    <div className="p-4 rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] space-y-3">
      <div className="flex items-center justify-between text-sm">
        <span className="text-[var(--color-text)] font-medium">Cleaning...</span>
        <span className="text-[var(--color-text-secondary)]">
          {progress.completed_items} / {progress.total_items} items
        </span>
      </div>

      {/* Progress bar */}
      <div className="h-2 rounded-full bg-[var(--color-bg-tertiary)] overflow-hidden">
        <motion.div
          className="h-full rounded-full bg-[var(--color-danger)]"
          initial={{ width: 0 }}
          animate={{ width: `${percent}%` }}
          transition={{ duration: 0.3 }}
        />
      </div>

      <div className="flex items-center justify-between text-xs text-[var(--color-text-muted)]">
        <span className="truncate">{progress.current_item}</span>
        <span className="shrink-0">{formatSize(progress.bytes_cleaned)} freed</span>
      </div>
    </div>
  );
}

import { motion, useReducedMotion } from 'framer-motion';
import { Loader2, X } from 'lucide-react';
import type { CleanProgressDto } from '@/lib/tauri';
import { useCleanStore } from '@/stores/clean-store';
import { useFocusTrap } from '@/hooks/useFocusTrap';
import { formatSize } from '@/lib/format';

const SYSTEM_PREFIX = 'system:';

interface CleaningOverlayProps {
  progress: CleanProgressDto;
}

export function CleaningOverlay({ progress }: CleaningOverlayProps) {
  const canceling = useCleanStore((s) => s.canceling);
  const usedTrash = useCleanStore((s) => s.usedTrash);
  const reduceMotion = useReducedMotion();
  const trapRef = useFocusTrap<HTMLDivElement>(true);

  const isIndeterminate = progress.total_items === 0;
  const percent = isIndeterminate
    ? 0
    : Math.round((progress.completed_items / progress.total_items) * 100);

  const displayPath = progress.current_item?.startsWith(SYSTEM_PREFIX)
    ? progress.current_item.slice(SYSTEM_PREFIX.length)
    : progress.current_item;

  // Mode-aware accounting label: in Trash mode nothing is actually freed yet — items are only moved
  // to the Trash — so saying "freed" mid-clean would contradict the (correct) celebration screen.
  const movedLabel = usedTrash ? 'moved to Trash' : 'freed';

  const handleCancel = () => {
    useCleanStore.getState().cancelClean();
  };

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      className="fixed inset-0 z-30 flex items-center justify-center bg-black/50"
      role="dialog"
      aria-modal="true"
      aria-label="Cleaning in progress"
    >
      <div
        ref={trapRef}
        tabIndex={-1}
        className="glass-modal p-8 max-w-sm w-full mx-4 flex flex-col items-center gap-5 outline-none"
      >
        <Loader2
          size={32}
          className="animate-spin text-[var(--color-primary)]"
        />
        <div className="text-center">
          <p className="text-base font-semibold text-[var(--color-text)]">
            {canceling ? 'Stopping…' : 'Cleaning…'}
          </p>
          <p className="text-sm text-[var(--color-text-secondary)] mt-1 tabular-nums">
            {canceling
              ? 'Finishing the current item safely…'
              : isIndeterminate
                ? 'Preparing…'
                : `${progress.completed_items} / ${progress.total_items} items`}
          </p>
        </div>

        {/* Progress bar */}
        <div className="w-full h-2 rounded-full bg-[var(--color-bg-tertiary)] overflow-hidden relative">
          {isIndeterminate ? (
            reduceMotion ? (
              <div className="absolute h-full w-1/3 rounded-full bg-[var(--color-primary)] opacity-60" />
            ) : (
              <motion.div
                className="absolute h-full w-1/3 rounded-full bg-gradient-to-r from-transparent via-[var(--color-primary)] to-transparent"
                animate={{ x: ['-100%', '400%'] }}
                transition={{ duration: 1.5, repeat: Infinity, ease: 'easeInOut' }}
              />
            )
          ) : (
            <motion.div
              className="h-full rounded-full bg-gradient-to-r from-[var(--color-primary)] to-[var(--color-safe)]"
              initial={{ width: 0 }}
              animate={{ width: `${percent}%` }}
              transition={{ duration: reduceMotion ? 0 : 0.3 }}
            />
          )}
        </div>

        <p className="text-xs text-[var(--color-text-muted)] tabular-nums">
          {formatSize(progress.bytes_cleaned)} {movedLabel}
        </p>

        {displayPath && (
          <p className="text-[11px] text-[var(--color-text-muted)] truncate max-w-full">
            {displayPath}
          </p>
        )}

        {/* Cancel button */}
        <button
          onClick={handleCancel}
          disabled={canceling}
          className="pill flex items-center gap-2 px-4 py-2 text-xs text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-hover)] transition-colors focus-visible:ring-2 focus-visible:ring-[var(--color-primary)] outline-none disabled:opacity-50 disabled:cursor-not-allowed"
          aria-label="Cancel cleaning"
        >
          <X size={12} />
          {canceling ? 'Stopping…' : 'Cancel'}
        </button>
      </div>
    </motion.div>
  );
}

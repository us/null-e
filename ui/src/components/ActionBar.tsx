import { Trash2 } from 'lucide-react';
import { motion } from 'framer-motion';
import { formatSize } from '@/lib/format';

interface ActionBarProps {
  selectedSize: number;
  selectedCount: number;
  onClean: () => void;
}

export function ActionBar({
  selectedSize,
  selectedCount,
  onClean,
}: ActionBarProps) {
  return (
    <motion.div
      initial={{ y: 20, opacity: 0 }}
      animate={{ y: 0, opacity: 1 }}
      exit={{ y: 20, opacity: 0 }}
      className="shrink-0 border-t border-[var(--color-border)] glass px-5 py-3"
    >
      <button
        onClick={onClean}
        className="w-full flex items-center justify-center gap-3 py-3.5 rounded-full text-white font-semibold bg-[var(--color-primary)] hover:bg-[var(--color-primary-hover)] transition-all active:scale-[0.98] focus-visible:ring-2 focus-visible:ring-[var(--color-primary)] outline-none"
      >
        <Trash2 size={18} />
        <span className="tabular-nums">
          Clean {formatSize(selectedSize)} ({selectedCount} item
          {selectedCount !== 1 ? 's' : ''})
        </span>
      </button>
    </motion.div>
  );
}

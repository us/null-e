import { Trash2 } from 'lucide-react';
import { formatSize } from '@/lib/format';

interface CleanButtonProps {
  selectedSize: number;
  selectedCount: number;
  onClick: () => void;
  className?: string;
}

export function CleanButton({ selectedSize, selectedCount, onClick, className = '' }: CleanButtonProps) {
  const disabled = selectedCount === 0;

  return (
    <button
      onClick={onClick}
      disabled={disabled}
      className={`inline-flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium text-white transition-all ${
        disabled
          ? 'bg-[var(--color-text-muted)] opacity-50 cursor-not-allowed'
          : 'bg-[var(--color-danger)] hover:bg-[var(--color-danger-hover)] shadow-lg shadow-red-500/20'
      } ${className}`}
    >
      <Trash2 size={16} />
      <span>
        Clean{selectedCount > 0 ? ` (${selectedCount})` : ''}
        {selectedSize > 0 && ` - ${formatSize(selectedSize)}`}
      </span>
    </button>
  );
}

import { formatSize } from '@/lib/format';
import { getSizeHeatColor } from '@/lib/size-color';
import { SafetyBadge } from '@/components/shared/SafetyBadge';

export interface SystemEntry {
  path: string;
  name: string;
  size: number;
  icon: string;
  description: string;
  category: string;
  safetyLevel?: string;
  cacheId?: string;
}

interface SystemItemRowProps {
  entry: SystemEntry;
  selected: boolean;
  onToggle: (path: string) => void;
  maxSize: number;
}

/** Maps backend snake_case safety levels to PascalCase for SafetyBadge */
function normalizeSafetyLevel(level: string): string {
  const map: Record<string, string> = {
    safe: 'Safe',
    safe_with_cost: 'SafeWithCost',
    caution: 'Caution',
    dangerous: 'Dangerous',
  };
  return map[level] ?? level;
}

export function SystemItemRow({
  entry,
  selected,
  onToggle,
  maxSize,
}: SystemItemRowProps) {
  const barPct = maxSize > 0 ? (entry.size / maxSize) * 100 : 0;
  const isHuge = entry.size >= 1024 * 1024 * 1024;
  const heatColor = getSizeHeatColor(entry.size);

  return (
    <label
      className={`flex items-center gap-3 px-4 py-2.5 cursor-pointer transition-all border-b border-[var(--color-border)] group ${
        selected
          ? 'bg-[var(--color-primary-soft)]'
          : 'hover:bg-[var(--color-surface-hover)]'
      }`}
    >
      {/* Checkbox */}
      <input
        type="checkbox"
        checked={selected}
        onChange={() => onToggle(entry.path)}
        className="shrink-0 w-4 h-4 rounded cursor-pointer"
        aria-label={`Select ${entry.name}`}
      />

      {/* Icon */}
      <div className="shrink-0 w-9 h-9 flex items-center justify-center rounded-lg bg-[var(--color-bg-tertiary)] text-lg">
        {entry.icon}
      </div>

      {/* Content */}
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span className="font-semibold text-[var(--color-text)] truncate">
            {entry.name}
          </span>
          {entry.safetyLevel && (
            <SafetyBadge level={normalizeSafetyLevel(entry.safetyLevel)} />
          )}
        </div>
        <div className="flex items-center gap-2 mt-1">
          <div className="flex-1 h-1 rounded-full bg-[var(--color-bg-tertiary)] overflow-hidden max-w-[200px]">
            <div
              className="h-full rounded-full transition-all duration-300"
              style={{ width: `${barPct}%`, backgroundColor: heatColor }}
            />
          </div>
          <span className="text-[11px] text-[var(--color-text-muted)] truncate" title={entry.description}>
            {entry.description}
          </span>
        </div>
      </div>

      {/* Size */}
      <span
        className={`tabular-nums font-bold shrink-0 ${isHuge ? 'text-base' : 'text-sm'}`}
        style={{ color: heatColor }}
      >
        {formatSize(entry.size)}
      </span>
    </label>
  );
}

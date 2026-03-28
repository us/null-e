import { formatSize } from '@/lib/format';
import { getSizeHeatColor } from '@/lib/size-color';

export interface ImpactItem {
  path: string;
  name: string;
  detail: string;
  size: number;
  icon: React.ReactNode;
}

interface ImpactCardsProps {
  items: ImpactItem[];
  selectedPaths: Set<string>;
  onToggle: (path: string) => void;
}

export function ImpactCards({ items, selectedPaths, onToggle }: ImpactCardsProps) {
  if (items.length === 0) return null;

  return (
    <div className="shrink-0 px-5 py-3 border-b border-[var(--color-border)]">
      <div className="text-[11px] uppercase tracking-wider text-[var(--color-text-muted)] mb-2">
        Biggest items
      </div>
      <div className="grid grid-cols-3 gap-2">
        {items.map((item) => {
          const selected = selectedPaths.has(item.path);
          const color = getSizeHeatColor(item.size);

          return (
            <button
              key={item.path}
              onClick={() => onToggle(item.path)}
              className={`flex items-center gap-2.5 px-3 py-2.5 rounded-xl border transition-all focus-visible:ring-2 focus-visible:ring-[var(--color-primary)] outline-none ${
                selected
                  ? 'border-[var(--color-primary)] bg-[var(--color-primary-soft)]'
                  : 'border-[var(--color-border)] bg-[var(--color-bg-secondary)] hover:border-[var(--color-text-muted)]'
              }`}
              aria-label={`${item.name} — ${formatSize(item.size)}`}
              aria-pressed={selected}
            >
              <div className="shrink-0 text-lg">{item.icon}</div>
              <div className="flex-1 min-w-0 text-left">
                <div className="text-xs font-semibold text-[var(--color-text)] truncate">
                  {item.name}
                </div>
                <div className="text-[11px] text-[var(--color-text-muted)] truncate">
                  {item.detail}
                </div>
              </div>
              <div
                className="shrink-0 text-sm font-bold tabular-nums"
                style={{ color }}
              >
                {formatSize(item.size)}
              </div>
            </button>
          );
        })}
      </div>
    </div>
  );
}

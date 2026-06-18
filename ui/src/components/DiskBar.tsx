import { formatSize } from '@/lib/format';

interface DiskBarProps {
  totalCleanable: number;
  selectedSize: number;
  selectedCount: number;
  artifactCount: number;
  projectCount: number;
  systemItemCount?: number;
  diskUsed?: number;
  diskTotal?: number;
}

/**
 * Pastel-bento summary bar: a hero "selected to clean" card alongside coloured tint stat cards.
 * Replaces the old donut ring with the bento card language used across the app.
 */
export function DiskBar({
  totalCleanable,
  selectedSize,
  selectedCount,
  artifactCount,
  projectCount,
  systemItemCount = 0,
  diskUsed,
  diskTotal,
}: DiskBarProps) {
  const totalItems = artifactCount + systemItemCount;
  const hasDiskInfo = diskUsed != null && diskTotal != null && diskTotal > 0;
  const diskFree = hasDiskInfo ? diskTotal! - diskUsed! : 0;
  const afterCleanFree = diskFree + selectedSize;
  const selectedPct = totalCleanable > 0 ? Math.min(selectedSize / totalCleanable, 1) : 0;

  return (
    <div className="shrink-0 px-4 py-3 border-b border-[var(--color-border)]">
      <div className="grid grid-cols-[1.7fr_1fr_1fr] gap-2.5 sm:grid-cols-[1.7fr_1fr_1fr_1fr]">
        {/* Hero — selected to clean */}
        <div className="bento p-3.5 flex flex-col justify-center">
          <div className="text-[11px] uppercase tracking-wider text-[var(--color-text-muted)]">
            Selected to clean
          </div>
          <div className="flex items-baseline gap-2 mt-0.5">
            <span className="display text-3xl text-[var(--color-primary)] tabular-nums leading-none">
              {formatSize(selectedSize)}
            </span>
            <span className="text-sm text-[var(--color-text-muted)]">
              of {formatSize(totalCleanable)}
            </span>
          </div>
          {/* progress */}
          <div className="mt-2 h-1.5 rounded-full bg-[var(--color-bg-tertiary)] overflow-hidden">
            <div
              className="h-full rounded-full bg-[var(--color-primary)] transition-all duration-500"
              style={{ width: `${selectedPct * 100}%` }}
            />
          </div>
          {hasDiskInfo && (
            <div
              className="text-[11px] text-[var(--color-text-muted)] mt-1.5"
              title="In Trash mode the space is reclaimed once you empty the Trash."
            >
              {formatSize(diskFree)} free
              {selectedSize > 0 && (
                <> → up to <span className="text-[var(--color-safe)] font-medium">{formatSize(afterCleanFree)}</span> after cleanup</>
              )}
            </div>
          )}
        </div>

        {/* Disk free */}
        <div className="tint-peach rounded-[18px] p-3.5 flex flex-col justify-center">
          <div className="text-[11px] opacity-70 lowercase">disk free</div>
          <div className="display text-xl tabular-nums mt-0.5">
            {hasDiskInfo ? formatSize(diskFree) : '—'}
          </div>
        </div>

        {/* Projects */}
        <div className="tint-sky rounded-[18px] p-3.5 flex flex-col justify-center">
          <div className="text-[11px] opacity-70 lowercase">projects</div>
          <div className="display text-xl tabular-nums mt-0.5">{projectCount}</div>
        </div>

        {/* Items (hidden on smallest width via grid template) */}
        <div className="tint-lav rounded-[18px] p-3.5 flex-col justify-center hidden sm:flex">
          <div className="text-[11px] opacity-70 lowercase">items</div>
          <div className="display text-xl tabular-nums mt-0.5">
            {selectedCount}<span className="text-sm opacity-60"> / {totalItems}</span>
          </div>
        </div>
      </div>
    </div>
  );
}

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

  const ringSize = 84;
  const strokeWidth = 6;
  const radius = (ringSize - strokeWidth) / 2;
  const circumference = 2 * Math.PI * radius;

  const usedPct = hasDiskInfo ? Math.min(diskUsed! / diskTotal!, 1) : 0;
  const cleanablePct = hasDiskInfo
    ? Math.min(totalCleanable / diskTotal!, usedPct)
    : 0;
  const selectedPct = hasDiskInfo
    ? Math.min(selectedSize / diskTotal!, cleanablePct)
    : totalCleanable > 0
      ? selectedSize / totalCleanable
      : 0;

  const diskFree = hasDiskInfo ? diskTotal! - diskUsed! : 0;
  const afterCleanFree = diskFree + selectedSize;

  return (
    <div className="shrink-0 px-5 py-4 border-b border-[var(--color-border)]">
      <div className="flex items-center justify-between">
        {/* Left: ring + text */}
        <div className="flex items-center gap-4">
          {/* Donut ring */}
          <div className="relative shrink-0" style={{ width: ringSize, height: ringSize }}>
            {hasDiskInfo ? (
              <svg width={ringSize} height={ringSize} className="-rotate-90">
                <circle cx={ringSize / 2} cy={ringSize / 2} r={radius} fill="none"
                  stroke="var(--color-bg-tertiary)" strokeWidth={strokeWidth} />
                <circle cx={ringSize / 2} cy={ringSize / 2} r={radius} fill="none"
                  stroke="var(--color-text-muted)" strokeWidth={strokeWidth}
                  strokeDasharray={circumference}
                  strokeDashoffset={circumference * (1 - usedPct)}
                  strokeLinecap="round" className="opacity-20" />
                {cleanablePct > 0 && (
                  <circle cx={ringSize / 2} cy={ringSize / 2} r={radius} fill="none"
                    stroke="#f59e0b" strokeWidth={strokeWidth}
                    strokeDasharray={circumference}
                    strokeDashoffset={circumference * (1 - cleanablePct)}
                    strokeLinecap="round" className="opacity-40 transition-all duration-500" />
                )}
                <circle cx={ringSize / 2} cy={ringSize / 2} r={radius} fill="none"
                  stroke="var(--color-primary)" strokeWidth={strokeWidth}
                  strokeDasharray={circumference}
                  strokeDashoffset={circumference * (1 - selectedPct)}
                  strokeLinecap="round" className="transition-all duration-500 ease-out" />
              </svg>
            ) : (
              /* Skeleton loading ring */
              <svg width={ringSize} height={ringSize} className="-rotate-90">
                <circle cx={ringSize / 2} cy={ringSize / 2} r={radius} fill="none"
                  stroke="var(--color-bg-tertiary)" strokeWidth={strokeWidth} />
                <circle cx={ringSize / 2} cy={ringSize / 2} r={radius} fill="none"
                  stroke="var(--color-bg-tertiary)" strokeWidth={strokeWidth}
                  strokeDasharray={circumference}
                  strokeDashoffset={circumference * 0.7}
                  strokeLinecap="round" className="animate-pulse opacity-30" />
              </svg>
            )}
            {/* Center: logo inside ring */}
            <div className="absolute inset-0 flex items-center justify-center">
              <img
                src="/logo.png"
                alt="null-e"
                width={ringSize - strokeWidth * 2 - 8}
                height={ringSize - strokeWidth * 2 - 8}
                className="rounded-xl"
              />
            </div>
          </div>

          <div>
            <div className="text-xs uppercase tracking-wider text-[var(--color-text-muted)] mb-1">
              Selected to clean
            </div>
            <div className="flex items-baseline gap-2">
              <span className="text-2xl font-bold tabular-nums text-[var(--color-primary)]">
                {formatSize(selectedSize)}
              </span>
              <span className="text-sm text-[var(--color-text-muted)]">
                of {formatSize(totalCleanable)}
              </span>
            </div>
            {hasDiskInfo && (
              <div className="text-[11px] text-[var(--color-text-muted)] mt-0.5">
                {formatSize(diskTotal!)} disk · {formatSize(diskFree)} free
                {selectedSize > 0 && (
                  <> → <span className="text-[var(--color-safe)] font-medium">{formatSize(afterCleanFree)} free</span></>
                )}
              </div>
            )}
            {/* Ring legend */}
            {hasDiskInfo && (
              <div className="flex items-center gap-3 mt-1.5">
                <LegendDot color="var(--color-text-muted)" opacity={0.2} label="Used" />
                <LegendDot color="#f59e0b" opacity={0.4} label="Cleanable" />
                <LegendDot color="var(--color-primary)" opacity={1} label="Selected" />
              </div>
            )}
          </div>
        </div>

        {/* Right: stats */}
        <div className="flex gap-4 text-right">
          <Stat label="Items" value={`${selectedCount} / ${totalItems}`} />
          <Stat label="Projects" value={String(projectCount)} />
          {systemItemCount > 0 && (
            <Stat label="System" value={String(systemItemCount)} />
          )}
          {hasDiskInfo && (
            <Stat label="Disk" value={`${formatSize(diskUsed!)} / ${formatSize(diskTotal!)}`} />
          )}
        </div>
      </div>
    </div>
  );
}

function Stat({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <div className="text-xs text-[var(--color-text-muted)]">{label}</div>
      <div className="text-sm font-semibold tabular-nums text-[var(--color-text-secondary)]">
        {value}
      </div>
    </div>
  );
}

function LegendDot({ color, opacity, label }: { color: string; opacity: number; label: string }) {
  return (
    <div className="flex items-center gap-1">
      <span
        className="w-2 h-2 rounded-full"
        style={{ backgroundColor: color, opacity }}
      />
      <span className="text-[9px] text-[var(--color-text-muted)]">{label}</span>
    </div>
  );
}

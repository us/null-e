interface SafetyBadgeProps {
  level: string;
}

const levelConfig: Record<string, { label: string; color: string; bg: string }> = {
  Safe: { label: 'Safe', color: 'var(--color-safe)', bg: 'rgba(34,197,94,0.15)' },
  SafeWithCost: { label: 'Safe*', color: 'var(--color-warning)', bg: 'rgba(245,158,11,0.15)' },
  Caution: { label: 'Caution', color: '#F97316', bg: 'rgba(249,115,22,0.15)' },
  Dangerous: { label: 'Danger', color: 'var(--color-danger)', bg: 'rgba(239,68,68,0.15)' },
};

export function SafetyBadge({ level }: SafetyBadgeProps) {
  const config = levelConfig[level] ?? {
    label: level,
    color: 'var(--color-text-muted)',
    bg: 'rgba(156,163,175,0.15)',
  };

  return (
    <span
      className="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium"
      style={{ color: config.color, backgroundColor: config.bg }}
    >
      {config.label}
    </span>
  );
}

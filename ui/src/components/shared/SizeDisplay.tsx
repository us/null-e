import { formatSize } from '@/lib/format';

interface SizeDisplayProps {
  bytes: number;
  className?: string;
}

export function SizeDisplay({ bytes, className = '' }: SizeDisplayProps) {
  const isLarge = bytes >= 1024 * 1024 * 100; // 100MB+

  return (
    <span
      className={`font-semibold ${
        isLarge
          ? 'bg-gradient-to-r from-[var(--color-primary)] to-[var(--color-safe)] bg-clip-text text-transparent'
          : 'text-[var(--color-text)]'
      } ${className}`}
    >
      {formatSize(bytes)}
    </span>
  );
}

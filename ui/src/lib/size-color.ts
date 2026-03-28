const SIZE_50MB = 50 * 1024 * 1024;
const SIZE_500MB = 500 * 1024 * 1024;

/**
 * Returns a heat color based on size:
 * - Green (#22c55e) for < 50 MB
 * - Amber (#f59e0b) for 50–500 MB
 * - Red (#ef4444) for > 500 MB
 */
export function getSizeHeatColor(size: number): string {
  if (size >= SIZE_500MB) return '#ef4444';
  if (size >= SIZE_50MB) return '#f59e0b';
  return '#22c55e';
}

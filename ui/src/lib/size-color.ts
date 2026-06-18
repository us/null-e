const SIZE_50MB = 50 * 1024 * 1024;
const SIZE_500MB = 500 * 1024 * 1024;

/**
 * Returns a heat color based on size, using the normal-is-dead brand palette:
 * - Teal/sage (#96AA9F) for < 50 MB
 * - Mustard (#EEBE55) for 50–500 MB
 * - Brick (#EC5E52) for > 500 MB
 */
export function getSizeHeatColor(size: number): string {
  if (size >= SIZE_500MB) return '#EC5E52';
  if (size >= SIZE_50MB) return '#EEBE55';
  return '#96AA9F';
}

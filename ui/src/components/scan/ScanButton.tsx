import { Search, Loader2, X } from 'lucide-react';
import { useScanStore } from '@/stores/scan-store';

interface ScanButtonProps {
  className?: string;
  compact?: boolean;
}

export function ScanButton({ className = '', compact = false }: ScanButtonProps) {
  const { isScanning, startScan, cancelScan } = useScanStore();

  const handleClick = () => {
    if (isScanning) {
      cancelScan();
    } else {
      startScan([]).catch(console.error);
    }
  };

  return (
    <button
      onClick={handleClick}
      className={`inline-flex items-center justify-center gap-2 font-medium text-white rounded-lg transition-all ${
        isScanning
          ? 'bg-[var(--color-danger)] hover:bg-[var(--color-danger-hover)]'
          : 'bg-gradient-to-r from-[var(--color-primary)] to-blue-600 hover:from-blue-600 hover:to-blue-700 shadow-lg shadow-blue-500/25'
      } ${compact ? 'px-4 py-2 text-sm' : 'px-6 py-3 text-base'} ${className}`}
    >
      {isScanning ? (
        <>
          <Loader2 size={compact ? 14 : 18} className="animate-spin" />
          <X size={compact ? 14 : 18} />
          {!compact && <span>Cancel Scan</span>}
        </>
      ) : (
        <>
          <Search size={compact ? 14 : 18} />
          {!compact && <span>Start Scan</span>}
        </>
      )}
    </button>
  );
}

import { File, Folder } from 'lucide-react';
import type { ArtifactDto } from '@/lib/tauri';
import { SizeDisplay } from '@/components/shared/SizeDisplay';
import { SafetyBadge } from '@/components/shared/SafetyBadge';
import { formatNumber } from '@/lib/format';

interface ArtifactRowProps {
  artifact: ArtifactDto;
  selected: boolean;
  onToggle: () => void;
}

export function ArtifactRow({ artifact, selected, onToggle }: ArtifactRowProps) {
  const Icon = artifact.file_count > 1 ? Folder : File;

  return (
    <label className="flex items-center gap-3 px-3 py-2 rounded-lg hover:bg-[var(--color-bg-tertiary)] cursor-pointer transition-colors">
      <input
        type="checkbox"
        checked={selected}
        onChange={onToggle}
        className="w-4 h-4 rounded accent-[var(--color-primary)]"
      />
      <Icon size={14} className="text-[var(--color-text-muted)] shrink-0" />
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span className="text-sm text-[var(--color-text)] truncate">{artifact.name}</span>
          <span className="text-xs text-[var(--color-text-muted)]">{artifact.kind}</span>
        </div>
        <div className="flex items-center gap-3 text-xs text-[var(--color-text-muted)]">
          <span>{formatNumber(artifact.file_count)} files</span>
          <SafetyBadge level="Safe" />
        </div>
      </div>
      <SizeDisplay bytes={artifact.size} className="text-sm" />
    </label>
  );
}

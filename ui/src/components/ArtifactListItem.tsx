import { TechIcon, getTechColor, getArtifactIcon } from '@/components/shared/TechIcon';
import { formatSize } from '@/lib/format';
import { getSizeHeatColor } from '@/lib/size-color';
import type { ArtifactDto } from '@/lib/tauri';

interface ArtifactListItemProps {
  artifact: ArtifactDto;
  projectName: string;
  projectRoot: string;
  kind: string;
  selected: boolean;
  onToggle: (path: string) => void;
  maxSize: number;
}

export function ArtifactListItem({
  artifact,
  projectName,
  projectRoot,
  kind,
  selected,
  onToggle,
  maxSize,
}: ArtifactListItemProps) {
  const techColor = getTechColor(kind);
  const heatColor = getSizeHeatColor(artifact.size);
  const barPct = maxSize > 0 ? (artifact.size / maxSize) * 100 : 0;
  const isHuge = artifact.size >= 1024 * 1024 * 1024;
  const artifactEmoji = getArtifactIcon(artifact.kind);

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
        onChange={() => onToggle(artifact.path)}
        className="shrink-0 w-4 h-4 rounded cursor-pointer focus-visible:ring-2 focus-visible:ring-[var(--color-primary)] outline-none"
        aria-label={`Select ${projectName} ${artifact.name}`}
      />

      {/* Tech icon in colored box */}
      <div
        className="shrink-0 w-9 h-9 flex items-center justify-center rounded-xl"
        style={{ backgroundColor: `${techColor}15` }}
      >
        <TechIcon kind={kind} size={22} />
      </div>

      {/* Content */}
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          {/* Project name = primary */}
          <span className="font-semibold text-[var(--color-text)] truncate">
            {projectName}
          </span>
          {/* Artifact badge */}
          <span className="shrink-0 text-[11px] flex items-center gap-1 px-2 py-0.5 rounded-full bg-[var(--color-bg-tertiary)] text-[var(--color-text-secondary)]">
            <span>{artifactEmoji}</span>
            {artifact.name}
          </span>
        </div>
        {/* Size bar + path */}
        <div className="flex items-center gap-2 mt-1">
          <div className="flex-1 h-1 rounded-full bg-[var(--color-bg-tertiary)] overflow-hidden max-w-[200px]">
            <div
              className="h-full rounded-full transition-all duration-300"
              style={{ width: `${barPct}%`, backgroundColor: heatColor }}
            />
          </div>
          <span className="text-[11px] text-[var(--color-text-muted)] truncate" title={projectRoot}>
            {shortenPath(projectRoot)}
          </span>
        </div>
      </div>

      {/* Size */}
      <span
        className={`tabular-nums font-bold shrink-0 ${isHuge ? 'text-base' : 'text-sm'}`}
        style={{ color: heatColor }}
      >
        {formatSize(artifact.size)}
      </span>
    </label>
  );
}

function shortenPath(path: string): string {
  return path
    .replace(/^\/(?:Users|home)\/[^/]+/, '~')
    .replace(/^[A-Za-z]:\\Users\\[^\\]+/, '~');
}

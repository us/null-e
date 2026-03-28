import { useState } from 'react';
import { ChevronDown, ChevronRight, FolderOpen } from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';
import type { ProjectDto } from '@/lib/tauri';
import { SizeDisplay } from '@/components/shared/SizeDisplay';
import { ArtifactRow } from './ArtifactRow';

interface ProjectCardProps {
  project: ProjectDto;
  selectedArtifacts: Set<string>;
  onToggleArtifact: (path: string) => void;
  onToggleProject: (project: ProjectDto) => void;
  allSelected: boolean;
}

const kindColors: Record<string, string> = {
  Node: '#68A063',
  Rust: '#DEA584',
  Python: '#3776AB',
  Go: '#00ADD8',
  Java: '#ED8B00',
  DotNet: '#512BD4',
  Ruby: '#CC342D',
  Swift: '#F05138',
  Dart: '#00B4AB',
};

function getKindColor(kind: string): string {
  return kindColors[kind] ?? 'var(--color-text-secondary)';
}

export function ProjectCard({
  project,
  selectedArtifacts,
  onToggleArtifact,
  onToggleProject,
  allSelected,
}: ProjectCardProps) {
  const [expanded, setExpanded] = useState(false);
  const kindColor = getKindColor(project.kind);

  return (
    <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] overflow-hidden">
      {/* Header */}
      <div className="flex items-center gap-3 px-4 py-3">
        <input
          type="checkbox"
          checked={allSelected}
          onChange={() => onToggleProject(project)}
          className="w-4 h-4 rounded accent-[var(--color-primary)]"
        />
        <button
          onClick={() => setExpanded(!expanded)}
          className="text-[var(--color-text-muted)] hover:text-[var(--color-text)]"
        >
          {expanded ? <ChevronDown size={16} /> : <ChevronRight size={16} />}
        </button>
        <FolderOpen size={18} style={{ color: kindColor }} className="shrink-0" />
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <span className="text-sm font-medium text-[var(--color-text)] truncate">
              {project.name}
            </span>
            <span
              className="inline-flex items-center px-1.5 py-0.5 rounded text-[10px] font-medium"
              style={{ color: kindColor, backgroundColor: `${kindColor}20` }}
            >
              {project.kind}
            </span>
          </div>
          <p className="text-xs text-[var(--color-text-muted)] truncate">{project.root}</p>
        </div>
        <div className="text-right shrink-0">
          <SizeDisplay bytes={project.total_size} className="text-sm" />
          {project.cleanable_size > 0 && (
            <p className="text-xs text-[var(--color-safe)]">
              {((project.cleanable_size / project.total_size) * 100).toFixed(0)}% cleanable
            </p>
          )}
        </div>
      </div>

      {/* Artifacts */}
      <AnimatePresence>
        {expanded && (
          <motion.div
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: 'auto', opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            transition={{ duration: 0.2 }}
            className="overflow-hidden"
          >
            <div className="border-t border-[var(--color-border)] px-2 py-1 space-y-0.5">
              {project.artifacts.map((artifact) => (
                <ArtifactRow
                  key={artifact.path}
                  artifact={artifact}
                  selected={selectedArtifacts.has(artifact.path)}
                  onToggle={() => onToggleArtifact(artifact.path)}
                />
              ))}
              {project.artifacts.length === 0 && (
                <p className="text-xs text-[var(--color-text-muted)] px-3 py-2">
                  No cleanable artifacts found
                </p>
              )}
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}

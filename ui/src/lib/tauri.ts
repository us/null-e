import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

// Scan types
export interface ScanConfigDto {
  roots: string[];
  max_depth?: number;
  min_size?: number;
}

export interface ScanProgressDto {
  directories_scanned: number;
  projects_found: number;
  total_size_found: number;
  current_path: string;
  is_complete: boolean;
}

export interface ProjectDto {
  id: number;
  kind: string;
  root: string;
  name: string;
  artifacts: ArtifactDto[];
  total_size: number;
  cleanable_size: number;
}

export interface ArtifactDto {
  path: string;
  kind: string;
  size: number;
  file_count: number;
  name: string;
}

export interface ScanResultDto {
  projects: ProjectDto[];
  total_size: number;
  total_cleanable: number;
  duration_ms: number;
  directories_scanned: number;
}

// Clean types
export interface CleanConfigDto {
  use_trash: boolean;
  dry_run: boolean;
  force: boolean;
}

export interface CleanProgressDto {
  total_items: number;
  completed_items: number;
  bytes_cleaned: number;
  current_item: string;
  is_complete: boolean;
}

export interface CleanFailureDto {
  path: string;
  reason: string;
  is_tcc: boolean;
  /** Failure class for grouping: fda | needs_admin | sip_protected | read_only | immutable | busy | refused | other. */
  category: string;
}

export interface CleanSummaryDto {
  total_items: number;
  succeeded: number;
  failed: number;
  /** Bytes returned to free space now (0 for Trash mode — see bytes_pending). */
  bytes_freed: number;
  /** Bytes pending reclamation (Trash mode: emptying the Trash frees these). */
  bytes_pending: number;
  used_trash: boolean;
  method_label: string;
  failures: CleanFailureDto[];
}

// Cache types
export interface GlobalCacheDto {
  name: string;
  id: string;
  icon: string;
  path: string;
  size: number;
  file_count: number;
  clean_command?: string;
  description: string;
}

// Cleaner types
export type Reclaimability =
  | 'user_reclaimable'
  | 'needs_admin'
  | 'os_managed_purgeable'
  | 'sip_protected';

export interface CleanableItemDto {
  name: string;
  category: string;
  subcategory: string;
  icon: string;
  path: string;
  /** On-disk allocated size (bytes). */
  size: number;
  /** Bytes honestly reclaimable (0 for OS-managed/purgeable & SIP-protected). */
  reclaimable_bytes: number;
  reclaimability: Reclaimability;
  description: string;
  safety_level: string;
  clean_command?: string;
}

export interface DetectCleanersResultDto {
  items: CleanableItemDto[];
  /** Cleaners whose detection errored (surfaced so scan gaps aren't hidden). */
  skipped: string[];
}

// Disk info
export interface DiskInfoDto {
  total: number;
  used: number;
  available: number;
  mount_point: string;
}

export interface FdaStatusDto {
  status: 'granted' | 'not_granted' | 'unknown';
  platform: string;
}

export interface SystemActionResultDto {
  success: boolean;
  bytes_freed: number;
  message: string;
}

// Commands
export const commands = {
  startScan: (config: ScanConfigDto) =>
    invoke<string>('start_scan', { config }),

  cancelScan: () => invoke<void>('cancel_scan'),

  getScanResult: () => invoke<ScanResultDto | null>('get_scan_result'),

  startClean: (targets: string[], config: CleanConfigDto) =>
    invoke<string>('start_clean', { targets, config }),

  cancelClean: () => invoke<void>('cancel_clean'),

  detectCaches: () => invoke<GlobalCacheDto[]>('detect_caches'),

  cleanCache: (id: string) => invoke<number>('clean_cache', { id }),

  detectCleaners: () => invoke<DetectCleanersResultDto>('detect_cleaners'),

  /** Run a typed system action (official command or guarded delete), resolved server-side. */
  runSystemAction: (path: string, name: string) =>
    invoke<SystemActionResultDto>('run_system_action', { path, name }),

  getConfig: () => invoke<Record<string, unknown>>('get_config'),

  saveConfig: (config: Record<string, unknown>) =>
    invoke<void>('save_config', { config }),

  getDiskInfo: () => invoke<DiskInfoDto>('get_disk_info'),

  getAppVersion: () => invoke<string>('get_app_version'),

  checkFdaStatus: () => invoke<FdaStatusDto>('check_fda_status'),

  /** Open the macOS Full Disk Access settings pane (with the Privacy_AllFiles anchor). */
  openPrivacySettings: () => invoke<void>('open_privacy_settings'),

  /** Empty the user's Trash; returns bytes actually freed. */
  emptyTrash: () => invoke<number>('empty_trash'),
};

// Events
export const events = {
  onScanProgress: (handler: (payload: ScanProgressDto) => void) =>
    listen<ScanProgressDto>('scan:progress', (e) => handler(e.payload)),

  onScanComplete: (handler: (payload: ScanResultDto) => void) =>
    listen<ScanResultDto>('scan:complete', (e) => handler(e.payload)),

  onScanError: (handler: (payload: string) => void) =>
    listen<string>('scan:error', (e) => handler(e.payload)),

  onCleanProgress: (handler: (payload: CleanProgressDto) => void) =>
    listen<CleanProgressDto>('clean:progress', (e) => handler(e.payload)),

  onCleanComplete: (handler: (payload: CleanSummaryDto) => void) =>
    listen<CleanSummaryDto>('clean:complete', (e) => handler(e.payload)),
};

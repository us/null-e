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

export interface CleanSummaryDto {
  total_items: number;
  succeeded: number;
  failed: number;
  bytes_freed: number;
  used_trash: boolean;
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
export interface CleanableItemDto {
  name: string;
  category: string;
  subcategory: string;
  icon: string;
  path: string;
  size: number;
  description: string;
  safety_level: string;
  clean_command?: string;
}

// Disk info
export interface DiskInfoDto {
  total: number;
  used: number;
  available: number;
  mount_point: string;
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

  cleanCache: (id: string) => invoke<void>('clean_cache', { id }),

  detectCleaners: () => invoke<CleanableItemDto[]>('detect_cleaners'),

  getConfig: () => invoke<Record<string, unknown>>('get_config'),

  saveConfig: (config: Record<string, unknown>) =>
    invoke<void>('save_config', { config }),

  getDiskInfo: () => invoke<DiskInfoDto>('get_disk_info'),

  getAppVersion: () => invoke<string>('get_app_version'),
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

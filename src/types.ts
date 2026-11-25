export type Language = "tr" | "en";

export interface GameProfile {
  id: string;
  name: string;
  exeName: string;
  enabled: boolean;
  highPriority: boolean;
  affinityCores: number[] | null;
}

export interface BackgroundApp {
  exeName: string;
  label: string;
  enabled: boolean;
}

export interface AppConfig {
  language: Language;
  autoDetect: boolean;
  pollIntervalMs: number;
  powerPlanBoost: boolean;
  highResTimer: boolean;
  ecoQosEnabled: boolean;
  gameProfiles: GameProfile[];
  backgroundApps: BackgroundApp[];
}

export interface LogEntry {
  time: string;
  level: "info" | "success" | "warn";
  messageTr: string;
  messageEn: string;
}

export interface DashboardStatus {
  boostActive: boolean;
  manualMode: boolean;
  activeGameName: string | null;
  activeGameExe: string | null;
  cpuUsage: number;
  ramUsedGb: number;
  ramTotalGb: number;
  ramPercent: number;
  logicalCores: number;
  highPriorityApplied: boolean;
  affinityApplied: boolean;
  ecoQosCount: number;
  ecoQosApps: string[];
  powerPlanApplied: boolean;
  timerApplied: boolean;
  logs: LogEntry[];
}

export interface RunningProcessInfo {
  exeName: string;
  instanceCount: number;
  memoryMb: number;
  isTrackedGame: boolean;
  isBackgroundApp: boolean;
}

import React, { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Activity,
  Cpu,
  Gamepad2,
  Gauge,
  Globe,
  Layers,
  Leaf,
  MemoryStick,
  Plus,
  Power,
  Settings,
  ShieldCheck,
  Timer,
  Trash2,
  Zap,
  X,
} from "lucide-react";
import {
  AppConfig,
  DashboardStatus,
  GameProfile,
  Language,
  RunningProcessInfo,
} from "./types";
import { t } from "./i18n";
import { ProcessPickerModal } from "./components/ProcessPickerModal";
import { AffinityModal } from "./components/AffinityModal";

type TabKey = "games" | "background" | "settings";

const DEFAULT_STATUS: DashboardStatus = {
  boostActive: false,
  manualMode: false,
  activeGameName: null,
  activeGameExe: null,
  cpuUsage: 0,
  ramUsedGb: 0,
  ramTotalGb: 0,
  ramPercent: 0,
  logicalCores: 8,
  highPriorityApplied: false,
  affinityApplied: false,
  ecoQosCount: 0,
  ecoQosApps: [],
  powerPlanApplied: false,
  timerApplied: false,
  logs: [],
};

export function App() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [status, setStatus] = useState<DashboardStatus>(DEFAULT_STATUS);
  const [activeTab, setActiveTab] = useState<TabKey>("games");
  const [busyBoost, setBusyBoost] = useState(false);
  const [errorToast, setErrorToast] = useState<string | null>(null);

  // Modals
  const [pickerMode, setPickerMode] = useState<"game" | "background" | null>(
    null
  );
  const [runningProcesses, setRunningProcesses] = useState<
    RunningProcessInfo[]
  >([]);
  const [loadingProcesses, setLoadingProcesses] = useState(false);

  const [manualMode, setManualMode] = useState<"game" | "background" | null>(
    null
  );
  const [manualName, setManualName] = useState("");
  const [manualExe, setManualExe] = useState("");

  const [affinityProfile, setAffinityProfile] = useState<GameProfile | null>(
    null
  );

  const lang: Language = config?.language === "en" ? "en" : "tr";
  const dict = t(lang);

  const showError = useCallback((msg: unknown) => {
    const text = typeof msg === "string" ? msg : String(msg);
    setErrorToast(text);
    setTimeout(() => setErrorToast(null), 4500);
  }, []);

  const loadConfig = useCallback(async () => {
    try {
      const cfg = await invoke<AppConfig>("get_config");
      setConfig(cfg);
    } catch (err) {
      showError(err);
    }
  }, [showError]);

  const pollStatus = useCallback(async () => {
    try {
      const st = await invoke<DashboardStatus>("get_status");
      setStatus(st);
    } catch {
      // ignore transient poll errors
    }
  }, []);

  useEffect(() => {
    loadConfig();
    pollStatus();
    const timer = setInterval(pollStatus, 2000);
    return () => clearInterval(timer);
  }, [loadConfig, pollStatus]);

  const persistConfig = async (updated: AppConfig) => {
    setConfig(updated);
    try {
      const saved = await invoke<AppConfig>("save_config", {
        newConfig: updated,
      });
      setConfig(saved);
    } catch (err) {
      showError(err);
    }
  };

  const handleToggleLanguage = () => {
    if (!config) return;
    const nextLang: Language = lang === "tr" ? "en" : "tr";
    persistConfig({ ...config, language: nextLang });
  };

  const handleToggleManualBoost = async () => {
    setBusyBoost(true);
    try {
      const next = await invoke<DashboardStatus>("toggle_manual_boost");
      setStatus(next);
    } catch (err) {
      showError(err);
    } finally {
      setBusyBoost(false);
    }
  };

  const openProcessPicker = async (mode: "game" | "background") => {
    setPickerMode(mode);
    setLoadingProcesses(true);
    try {
      const list = await invoke<RunningProcessInfo[]>("list_running_processes");
      setRunningProcesses(list);
    } catch (err) {
      showError(err);
    } finally {
      setLoadingProcesses(false);
    }
  };

  const refreshProcesses = async () => {
    setLoadingProcesses(true);
    try {
      const list = await invoke<RunningProcessInfo[]>("list_running_processes");
      setRunningProcesses(list);
    } catch (err) {
      showError(err);
    } finally {
      setLoadingProcesses(false);
    }
  };

  const handleAddFromPicker = async (exeName: string) => {
    try {
      if (pickerMode === "game") {
        const updated = await invoke<AppConfig>("add_game_profile", {
          name: "",
          exeName,
        });
        setConfig(updated);
      } else if (pickerMode === "background") {
        const updated = await invoke<AppConfig>("add_background_app", {
          exeName,
          label: "",
        });
        setConfig(updated);
      }
      await refreshProcesses();
    } catch (err) {
      showError(err);
    }
  };

  const handleManualSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!manualExe.trim()) return;
    try {
      if (manualMode === "game") {
        const updated = await invoke<AppConfig>("add_game_profile", {
          name: manualName,
          exeName: manualExe,
        });
        setConfig(updated);
      } else if (manualMode === "background") {
        const updated = await invoke<AppConfig>("add_background_app", {
          exeName: manualExe,
          label: manualName,
        });
        setConfig(updated);
      }
      setManualMode(null);
      setManualName("");
      setManualExe("");
    } catch (err) {
      showError(err);
    }
  };

  const handleUpdateGame = async (profile: GameProfile) => {
    try {
      const updated = await invoke<AppConfig>("update_game_profile", {
        profile,
      });
      setConfig(updated);
    } catch (err) {
      showError(err);
    }
  };

  const handleRemoveGame = async (id: string) => {
    try {
      const updated = await invoke<AppConfig>("remove_game_profile", { id });
      setConfig(updated);
    } catch (err) {
      showError(err);
    }
  };

  const handleToggleBgApp = (exeName: string) => {
    if (!config) return;
    const nextApps = config.backgroundApps.map((app) =>
      app.exeName === exeName ? { ...app, enabled: !app.enabled } : app
    );
    persistConfig({ ...config, backgroundApps: nextApps });
  };

  const handleRemoveBgApp = async (exeName: string) => {
    try {
      const updated = await invoke<AppConfig>("remove_background_app", {
        exeName,
      });
      setConfig(updated);
    } catch (err) {
      showError(err);
    }
  };

  return (
    <div className="min-h-screen bg-[#070b14] text-slate-100 flex flex-col">
      {/* Subtle Ambient Top Glow */}
      <div
        className={`pointer-events-none fixed inset-x-0 top-0 h-64 transition-opacity duration-700 ${
          status.boostActive
            ? "bg-[radial-gradient(ellipse_at_top,_rgba(16,185,129,0.18),_transparent_70%)] opacity-100"
            : "bg-[radial-gradient(ellipse_at_top,_rgba(6,182,212,0.12),_transparent_70%)] opacity-100"
        }`}
      />

      {/* Error Toast */}
      {errorToast && (
        <div className="fixed bottom-5 right-5 z-50 max-w-sm rounded-xl bg-rose-950/95 border border-rose-500/40 px-4 py-3 text-xs text-rose-100 shadow-2xl flex items-center gap-2.5">
          <span className="flex-1">{errorToast}</span>
          <button onClick={() => setErrorToast(null)}>
            <X className="w-3.5 h-3.5 text-rose-300" />
          </button>
        </div>
      )}

      {/* Header */}
      <header className="relative z-10 flex items-center justify-between px-6 py-4 border-b border-slate-800/70 bg-[#070b14]/80 backdrop-blur-md">
        <div className="flex items-center gap-3">
          <div
            className={`w-10 h-10 rounded-xl flex items-center justify-center border transition-all ${
              status.boostActive
                ? "bg-emerald-500/15 border-emerald-400/40 text-emerald-400 shadow-lg shadow-emerald-500/20"
                : "bg-cyan-500/15 border-cyan-400/30 text-cyan-400 shadow-lg shadow-cyan-500/10"
            }`}
          >
            <Zap className="w-5 h-5" />
          </div>
          <div>
            <div className="flex items-center gap-2">
              <h1 className="text-lg font-bold tracking-tight text-white">
                Stream<span className="text-cyan-400">Boost</span>
              </h1>
              <span className="text-[10px] font-semibold uppercase tracking-wider px-2 py-0.5 rounded-full bg-slate-800/90 text-slate-400 border border-slate-700/70">
                v0.1.0
              </span>
            </div>
            <p className="text-xs text-slate-400">{dict.subtitle}</p>
          </div>
        </div>

        <div className="flex items-center gap-3">
          <span className="hidden md:inline-flex items-center gap-1.5 text-[11px] text-slate-400 px-2.5 py-1 rounded-lg bg-slate-900/70 border border-slate-800/80">
            <ShieldCheck className="w-3.5 h-3.5 text-emerald-400" />
            {dict.trayHint}
          </span>

          <button
            onClick={handleToggleLanguage}
            className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-slate-900 hover:bg-slate-800 border border-slate-700/80 text-xs font-semibold text-slate-200 transition-colors"
            title="TR / EN"
          >
            <Globe className="w-3.5 h-3.5 text-cyan-400" />
            {lang === "tr" ? "TR" : "EN"}
          </button>
        </div>
      </header>

      {/* Main Content */}
      <main className="relative z-10 flex-1 max-w-6xl w-full mx-auto px-6 py-5 space-y-5">
        {/* Top Live Status Card */}
        <section
          className={`rounded-2xl border p-5 transition-all duration-300 backdrop-blur-md ${
            status.boostActive
              ? "bg-gradient-to-br from-emerald-950/35 via-[#0d1424]/95 to-[#0b1120]/95 border-emerald-500/40 shadow-xl shadow-emerald-950/30"
              : "bg-[#0d1424]/90 border-slate-800/80 shadow-xl shadow-black/20"
          }`}
        >
          <div className="flex flex-col lg:flex-row lg:items-center justify-between gap-5">
            {/* Left: Status Indicator & Active Game */}
            <div className="flex items-start gap-4">
              <div className="relative mt-0.5">
                <div
                  className={`w-12 h-12 rounded-2xl flex items-center justify-center border ${
                    status.boostActive
                      ? "bg-emerald-500/20 border-emerald-400/50 text-emerald-300"
                      : "bg-slate-800/80 border-slate-700/80 text-slate-400"
                  }`}
                >
                  <Gauge className="w-6 h-6" />
                </div>
                {status.boostActive && (
                  <span className="absolute -top-1 -right-1 flex h-3.5 w-3.5">
                    <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75" />
                    <span className="relative inline-flex rounded-full h-3.5 w-3.5 bg-emerald-500" />
                  </span>
                )}
              </div>

              <div>
                <div className="flex items-center gap-2.5 flex-wrap">
                  <span
                    className={`text-xs font-bold tracking-wider px-2.5 py-0.5 rounded-full border ${
                      status.boostActive
                        ? "bg-emerald-500/20 text-emerald-300 border-emerald-500/40"
                        : config?.autoDetect
                        ? "bg-cyan-500/15 text-cyan-300 border-cyan-500/30"
                        : "bg-slate-800 text-slate-400 border-slate-700"
                    }`}
                  >
                    {status.boostActive
                      ? dict.statusBoostActive
                      : config?.autoDetect
                      ? dict.statusMonitoring
                      : dict.statusIdle}
                  </span>

                  {status.boostActive && (
                    <span className="text-sm font-semibold text-white">
                      {status.activeGameName
                        ? `${status.activeGameName} (${status.activeGameExe})`
                        : dict.manualModeLabel}
                    </span>
                  )}
                </div>

                <p className="text-xs text-slate-400 mt-1.5">
                  {status.boostActive
                    ? status.ecoQosApps.length > 0
                      ? `EcoQoS: ${status.ecoQosApps.slice(0, 4).join(", ")}${
                          status.ecoQosApps.length > 4
                            ? ` +${status.ecoQosApps.length - 4}`
                            : ""
                        }`
                      : dict.subtitle
                    : config?.autoDetect
                    ? dict.waitingForGame
                    : dict.autoDetectOff}
                </p>
              </div>
            </div>

            {/* Right: Telemetry + Manual Boost Button */}
            <div className="flex flex-wrap items-center gap-4">
              {/* CPU Gauge */}
              <div className="w-36 rounded-xl bg-slate-900/80 border border-slate-800/90 px-3.5 py-2.5">
                <div className="flex items-center justify-between text-[11px] text-slate-400 mb-1.5">
                  <span className="inline-flex items-center gap-1">
                    <Cpu className="w-3.5 h-3.5 text-cyan-400" />
                    CPU
                  </span>
                  <span className="font-semibold text-slate-100">
                    {status.cpuUsage.toFixed(1)}%
                  </span>
                </div>
                <div className="h-1.5 w-full rounded-full bg-slate-800 overflow-hidden">
                  <div
                    className="h-full rounded-full bg-gradient-to-r from-cyan-500 to-sky-400 transition-all duration-500"
                    style={{ width: `${Math.min(100, status.cpuUsage)}%` }}
                  />
                </div>
              </div>

              {/* RAM Gauge */}
              <div className="w-40 rounded-xl bg-slate-900/80 border border-slate-800/90 px-3.5 py-2.5">
                <div className="flex items-center justify-between text-[11px] text-slate-400 mb-1.5">
                  <span className="inline-flex items-center gap-1">
                    <MemoryStick className="w-3.5 h-3.5 text-violet-400" />
                    RAM
                  </span>
                  <span className="font-semibold text-slate-100">
                    {status.ramUsedGb.toFixed(1)}/{status.ramTotalGb.toFixed(0)}{" "}
                    GB
                  </span>
                </div>
                <div className="h-1.5 w-full rounded-full bg-slate-800 overflow-hidden">
                  <div
                    className="h-full rounded-full bg-gradient-to-r from-violet-500 to-fuchsia-400 transition-all duration-500"
                    style={{ width: `${Math.min(100, status.ramPercent)}%` }}
                  />
                </div>
              </div>

              {/* Manual Boost Toggle Button */}
              <button
                onClick={handleToggleManualBoost}
                disabled={busyBoost}
                className={`inline-flex items-center gap-2 px-4 py-3 rounded-xl text-xs font-bold transition-all cursor-pointer disabled:opacity-60 ${
                  status.boostActive
                    ? "bg-rose-500/20 hover:bg-rose-500/30 text-rose-200 border border-rose-500/40"
                    : "bg-gradient-to-r from-cyan-500 to-emerald-500 hover:from-cyan-400 hover:to-emerald-400 text-slate-950 shadow-lg shadow-cyan-500/20"
                }`}
              >
                <Power className="w-4 h-4" />
                {status.boostActive ? dict.btnBoostOff : dict.btnBoostOn}
              </button>
            </div>
          </div>

          {/* Active Optimization Badges */}
          <div className="mt-4 pt-4 border-t border-slate-800/70 flex flex-wrap items-center gap-2">
            <OptimizationBadge
              active={status.highPriorityApplied}
              icon={<Zap className="w-3.5 h-3.5" />}
              label={dict.badgeHighPriority}
            />
            <OptimizationBadge
              active={status.affinityApplied}
              icon={<Cpu className="w-3.5 h-3.5" />}
              label={`${dict.badgeAffinity} (${status.logicalCores} ${dict.coresLabel})`}
            />
            <OptimizationBadge
              active={status.ecoQosCount > 0}
              icon={<Leaf className="w-3.5 h-3.5" />}
              label={`${dict.badgeEcoQos} (${status.ecoQosCount})`}
            />
            <OptimizationBadge
              active={status.powerPlanApplied}
              icon={<Gauge className="w-3.5 h-3.5" />}
              label={dict.badgePowerPlan}
            />
            <OptimizationBadge
              active={status.timerApplied}
              icon={<Timer className="w-3.5 h-3.5" />}
              label={dict.badgeTimer}
            />
          </div>
        </section>

        {/* Navigation Tabs */}
        <div className="flex items-center gap-2 border-b border-slate-800/80 pb-1">
          <TabButton
            active={activeTab === "games"}
            onClick={() => setActiveTab("games")}
            icon={<Gamepad2 className="w-4 h-4" />}
            label={dict.tabGames}
            count={config?.gameProfiles.length}
          />
          <TabButton
            active={activeTab === "background"}
            onClick={() => setActiveTab("background")}
            icon={<Layers className="w-4 h-4" />}
            label={dict.tabBackground}
            count={config?.backgroundApps.length}
          />
          <TabButton
            active={activeTab === "settings"}
            onClick={() => setActiveTab("settings")}
            icon={<Settings className="w-4 h-4" />}
            label={dict.tabSettings}
          />
        </div>

        {/* TAB 1: GAME PROFILES */}
        {activeTab === "games" && config && (
          <section className="space-y-4">
            <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
              <p className="text-xs text-slate-400 max-w-2xl">
                {dict.gamesDesc}
              </p>
              <div className="flex items-center gap-2 shrink-0">
                <button
                  onClick={() => openProcessPicker("game")}
                  className="inline-flex items-center gap-1.5 px-3.5 py-2 rounded-xl bg-cyan-500/15 hover:bg-cyan-500/25 text-cyan-300 border border-cyan-500/30 text-xs font-semibold transition-colors"
                >
                  <Plus className="w-3.5 h-3.5" />
                  {dict.addFromRunning}
                </button>
                <button
                  onClick={() => {
                    setManualMode("game");
                    setManualName("");
                    setManualExe("");
                  }}
                  className="inline-flex items-center gap-1.5 px-3.5 py-2 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-200 border border-slate-700 text-xs font-medium transition-colors"
                >
                  <Plus className="w-3.5 h-3.5" />
                  {dict.addManual}
                </button>
              </div>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
              {config.gameProfiles.map((game) => {
                const isRunningNow =
                  status.boostActive &&
                  status.activeGameExe?.toLowerCase() ===
                    game.exeName.toLowerCase();
                return (
                  <div
                    key={game.id}
                    className={`rounded-2xl border p-4 transition-all ${
                      isRunningNow
                        ? "bg-emerald-950/25 border-emerald-500/40"
                        : "bg-[#0d1424]/80 border-slate-800/80 hover:border-slate-700/80"
                    }`}
                  >
                    <div className="flex items-start justify-between gap-3">
                      <div className="min-w-0">
                        <div className="flex items-center gap-2">
                          <h3 className="text-sm font-semibold text-slate-100 truncate">
                            {game.name}
                          </h3>
                          {isRunningNow && (
                            <span className="text-[10px] font-bold uppercase px-2 py-0.5 rounded-full bg-emerald-500/20 text-emerald-300 border border-emerald-500/30">
                              ACTIVE
                            </span>
                          )}
                        </div>
                        <p className="text-xs font-mono text-slate-400 truncate mt-0.5">
                          {game.exeName}
                        </p>
                      </div>

                      <div className="flex items-center gap-2">
                        <ToggleSwitch
                          checked={game.enabled}
                          onChange={(val) =>
                            handleUpdateGame({ ...game, enabled: val })
                          }
                        />
                        <button
                          onClick={() => handleRemoveGame(game.id)}
                          className="p-1.5 rounded-lg text-slate-500 hover:text-rose-400 hover:bg-rose-500/10 transition-colors"
                        >
                          <Trash2 className="w-4 h-4" />
                        </button>
                      </div>
                    </div>

                    <div className="mt-3.5 pt-3 border-t border-slate-800/70 flex flex-wrap items-center justify-between gap-2">
                      <label className="inline-flex items-center gap-2 text-xs text-slate-300 cursor-pointer">
                        <input
                          type="checkbox"
                          checked={game.highPriority}
                          onChange={(e) =>
                            handleUpdateGame({
                              ...game,
                              highPriority: e.target.checked,
                            })
                          }
                          className="w-3.5 h-3.5 accent-cyan-500 rounded"
                        />
                        <span>{dict.highPriority}</span>
                      </label>

                      <button
                        onClick={() => setAffinityProfile(game)}
                        className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-lg bg-slate-900 hover:bg-slate-800 border border-slate-700/80 text-[11px] font-medium text-slate-300 transition-colors"
                      >
                        <Cpu className="w-3.5 h-3.5 text-cyan-400" />
                        {game.affinityCores
                          ? `${game.affinityCores.length}/${status.logicalCores} ${dict.selectedCores}`
                          : dict.allCores}
                      </button>
                    </div>
                  </div>
                );
              })}
            </div>
          </section>
        )}

        {/* TAB 2: BACKGROUND APPS (EcoQoS) */}
        {activeTab === "background" && config && (
          <section className="space-y-4">
            <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
              <p className="text-xs text-slate-400 max-w-2xl">{dict.bgDesc}</p>
              <div className="flex items-center gap-2 shrink-0">
                <button
                  onClick={() => openProcessPicker("background")}
                  className="inline-flex items-center gap-1.5 px-3.5 py-2 rounded-xl bg-emerald-500/15 hover:bg-emerald-500/25 text-emerald-300 border border-emerald-500/30 text-xs font-semibold transition-colors"
                >
                  <Plus className="w-3.5 h-3.5" />
                  {dict.addFromRunning}
                </button>
                <button
                  onClick={() => {
                    setManualMode("background");
                    setManualName("");
                    setManualExe("");
                  }}
                  className="inline-flex items-center gap-1.5 px-3.5 py-2 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-200 border border-slate-700 text-xs font-medium transition-colors"
                >
                  <Plus className="w-3.5 h-3.5" />
                  {dict.addManual}
                </button>
              </div>
            </div>

            <div className="rounded-xl bg-cyan-950/25 border border-cyan-500/25 px-4 py-3 flex items-start gap-2.5 text-xs text-cyan-200/90">
              <ShieldCheck className="w-4 h-4 text-cyan-400 shrink-0 mt-0.5" />
              <span>{dict.protectedNotice}</span>
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
              {config.backgroundApps.map((app) => {
                const isThrottled = status.ecoQosApps.some(
                  (e) => e.toLowerCase() === app.exeName.toLowerCase()
                );
                return (
                  <div
                    key={app.exeName}
                    className={`rounded-2xl border p-3.5 flex items-center justify-between gap-3 transition-all ${
                      isThrottled
                        ? "bg-emerald-950/20 border-emerald-500/35"
                        : "bg-[#0d1424]/80 border-slate-800/80"
                    }`}
                  >
                    <div className="min-w-0">
                      <div className="flex items-center gap-2">
                        <span className="text-sm font-medium text-slate-100 truncate">
                          {app.label}
                        </span>
                        {isThrottled && (
                          <span className="text-[10px] font-semibold px-1.5 py-0.5 rounded bg-emerald-500/20 text-emerald-300">
                            EcoQoS
                          </span>
                        )}
                      </div>
                      <p className="text-xs font-mono text-slate-400 truncate mt-0.5">
                        {app.exeName}
                      </p>
                    </div>

                    <div className="flex items-center gap-2 shrink-0">
                      <ToggleSwitch
                        checked={app.enabled}
                        onChange={() => handleToggleBgApp(app.exeName)}
                      />
                      <button
                        onClick={() => handleRemoveBgApp(app.exeName)}
                        className="p-1.5 rounded-lg text-slate-500 hover:text-rose-400 hover:bg-rose-500/10 transition-colors"
                      >
                        <Trash2 className="w-3.5 h-3.5" />
                      </button>
                    </div>
                  </div>
                );
              })}
            </div>
          </section>
        )}

        {/* TAB 3: SETTINGS & LIVE LOGS */}
        {activeTab === "settings" && config && (
          <section className="grid grid-cols-1 lg:grid-cols-2 gap-5">
            {/* Left: Optimization Toggles & Safety */}
            <div className="space-y-4">
              <div className="rounded-2xl border border-slate-800/80 bg-[#0d1424]/85 p-5 space-y-4">
                <h2 className="text-sm font-semibold text-slate-100 flex items-center gap-2">
                  <Settings className="w-4 h-4 text-cyan-400" />
                  {dict.settingsTitle}
                </h2>

                <SettingRow
                  title={dict.autoDetectTitle}
                  desc={dict.autoDetectDesc}
                  checked={config.autoDetect}
                  onChange={(v) => persistConfig({ ...config, autoDetect: v })}
                />
                <SettingRow
                  title={dict.ecoQosTitle}
                  desc={dict.ecoQosDesc}
                  checked={config.ecoQosEnabled}
                  onChange={(v) =>
                    persistConfig({ ...config, ecoQosEnabled: v })
                  }
                />
                <SettingRow
                  title={dict.powerPlanTitle}
                  desc={dict.powerPlanDesc}
                  checked={config.powerPlanBoost}
                  onChange={(v) =>
                    persistConfig({ ...config, powerPlanBoost: v })
                  }
                />
                <SettingRow
                  title={dict.timerTitle}
                  desc={dict.timerDesc}
                  checked={config.highResTimer}
                  onChange={(v) =>
                    persistConfig({ ...config, highResTimer: v })
                  }
                />
              </div>

              <div className="rounded-2xl border border-emerald-500/25 bg-emerald-950/15 p-5 space-y-2.5">
                <h3 className="text-sm font-semibold text-emerald-300 flex items-center gap-2">
                  <ShieldCheck className="w-4 h-4" />
                  {dict.safetyTitle}
                </h3>
                <ul className="space-y-1.5 text-xs text-slate-300 list-disc pl-4 leading-relaxed">
                  {dict.safetyPoints.map((pt, i) => (
                    <li key={i}>{pt}</li>
                  ))}
                </ul>
              </div>
            </div>

            {/* Right: Activity Log */}
            <div className="rounded-2xl border border-slate-800/80 bg-[#0d1424]/85 p-5 flex flex-col max-h-[520px]">
              <h2 className="text-sm font-semibold text-slate-100 flex items-center gap-2 mb-3">
                <Activity className="w-4 h-4 text-cyan-400" />
                {dict.activityLog}
              </h2>

              <div className="flex-1 overflow-y-auto space-y-2 pr-1">
                {status.logs.length === 0 ? (
                  <p className="text-xs text-slate-500 py-8 text-center">
                    {dict.noLogs}
                  </p>
                ) : (
                  status.logs.map((entry, idx) => (
                    <div
                      key={`${entry.time}-${idx}`}
                      className="rounded-xl bg-slate-900/80 border border-slate-800/80 px-3 py-2 text-xs flex items-start gap-2.5"
                    >
                      <span className="font-mono text-[11px] text-slate-500 shrink-0 mt-0.5">
                        {entry.time}
                      </span>
                      <span
                        className={
                          entry.level === "success"
                            ? "text-emerald-300"
                            : entry.level === "warn"
                            ? "text-amber-300"
                            : "text-slate-300"
                        }
                      >
                        {lang === "tr" ? entry.messageTr : entry.messageEn}
                      </span>
                    </div>
                  ))
                )}
              </div>
            </div>
          </section>
        )}
      </main>

      {/* Running Process Picker Modal */}
      {pickerMode && (
        <ProcessPickerModal
          mode={pickerMode}
          lang={lang}
          processes={runningProcesses}
          loading={loadingProcesses}
          onRefresh={refreshProcesses}
          onSelect={handleAddFromPicker}
          onClose={() => setPickerMode(null)}
        />
      )}

      {/* Manual Add Modal */}
      {manualMode && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm p-4">
          <form
            onSubmit={handleManualSubmit}
            className="w-full max-w-md rounded-2xl border border-slate-800 bg-[#0d1322] shadow-2xl overflow-hidden"
          >
            <div className="flex items-center justify-between px-5 py-4 border-b border-slate-800/80">
              <h3 className="text-sm font-semibold text-slate-100">
                {manualMode === "game"
                  ? dict.manualModalTitleGame
                  : dict.manualModalTitleBg}
              </h3>
              <button
                type="button"
                onClick={() => setManualMode(null)}
                className="p-1.5 rounded-lg text-slate-400 hover:text-slate-100 hover:bg-slate-800"
              >
                <X className="w-4 h-4" />
              </button>
            </div>
            <div className="p-5 space-y-4">
              <div>
                <label className="block text-xs font-medium text-slate-300 mb-1.5">
                  {dict.displayName}
                </label>
                <input
                  type="text"
                  value={manualName}
                  onChange={(e) => setManualName(e.target.value)}
                  placeholder={
                    manualMode === "game" ? "Counter-Strike 2" : "Discord"
                  }
                  className="w-full px-3 py-2 text-sm rounded-xl bg-slate-900 border border-slate-700 text-slate-100 focus:outline-none focus:border-cyan-500"
                />
              </div>
              <div>
                <label className="block text-xs font-medium text-slate-300 mb-1.5">
                  {dict.exeName} *
                </label>
                <input
                  type="text"
                  required
                  value={manualExe}
                  onChange={(e) => setManualExe(e.target.value)}
                  placeholder={
                    manualMode === "game" ? "cs2.exe" : "Discord.exe"
                  }
                  className="w-full px-3 py-2 text-sm font-mono rounded-xl bg-slate-900 border border-slate-700 text-slate-100 focus:outline-none focus:border-cyan-500"
                />
              </div>
            </div>
            <div className="px-5 py-3.5 border-t border-slate-800/80 flex items-center justify-end gap-2 bg-slate-900/40">
              <button
                type="button"
                onClick={() => setManualMode(null)}
                className="px-4 py-2 rounded-xl text-xs font-medium text-slate-300 hover:bg-slate-800"
              >
                {dict.cancel}
              </button>
              <button
                type="submit"
                className="px-4 py-2 rounded-xl text-xs font-semibold bg-cyan-500 hover:bg-cyan-400 text-slate-950"
              >
                {dict.save}
              </button>
            </div>
          </form>
        </div>
      )}

      {/* Affinity Modal */}
      {affinityProfile && (
        <AffinityModal
          profile={affinityProfile}
          logicalCores={status.logicalCores}
          lang={lang}
          onSave={async (cores) => {
            await handleUpdateGame({
              ...affinityProfile,
              affinityCores: cores,
            });
            setAffinityProfile(null);
          }}
          onClose={() => setAffinityProfile(null)}
        />
      )}
    </div>
  );
}

function OptimizationBadge({
  active,
  icon,
  label,
}: {
  active: boolean;
  icon: React.ReactNode;
  label: string;
}) {
  return (
    <span
      className={`inline-flex items-center gap-1.5 px-2.5 py-1 rounded-lg text-[11px] font-medium border transition-colors ${
        active
          ? "bg-emerald-500/15 text-emerald-300 border-emerald-500/35"
          : "bg-slate-900/70 text-slate-500 border-slate-800"
      }`}
    >
      {icon}
      {label}
    </span>
  );
}

function TabButton({
  active,
  onClick,
  icon,
  label,
  count,
}: {
  active: boolean;
  onClick: () => void;
  icon: React.ReactNode;
  label: string;
  count?: number;
}) {
  return (
    <button
      onClick={onClick}
      className={`inline-flex items-center gap-2 px-4 py-2.5 rounded-t-xl text-xs font-semibold border-b-2 transition-all cursor-pointer ${
        active
          ? "border-cyan-400 text-cyan-300 bg-cyan-500/10"
          : "border-transparent text-slate-400 hover:text-slate-200 hover:bg-slate-900/50"
      }`}
    >
      {icon}
      <span>{label}</span>
      {typeof count === "number" && (
        <span
          className={`px-1.5 py-0.5 rounded-md text-[10px] ${
            active
              ? "bg-cyan-500/20 text-cyan-200"
              : "bg-slate-800 text-slate-400"
          }`}
        >
          {count}
        </span>
      )}
    </button>
  );
}

function ToggleSwitch({
  checked,
  onChange,
}: {
  checked: boolean;
  onChange: (val: boolean) => void;
}) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      onClick={() => onChange(!checked)}
      className={`relative inline-flex h-5 w-9 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ${
        checked ? "bg-cyan-500" : "bg-slate-700"
      }`}
    >
      <span
        className={`pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow transition duration-200 ${
          checked ? "translate-x-4" : "translate-x-0"
        }`}
      />
    </button>
  );
}

function SettingRow({
  title,
  desc,
  checked,
  onChange,
}: {
  title: string;
  desc: string;
  checked: boolean;
  onChange: (val: boolean) => void;
}) {
  return (
    <div className="flex items-start justify-between gap-4 py-2 border-b border-slate-800/60 last:border-b-0">
      <div>
        <div className="text-xs font-semibold text-slate-200">{title}</div>
        <div className="text-[11px] text-slate-400 mt-0.5 leading-relaxed">
          {desc}
        </div>
      </div>
      <div className="pt-0.5">
        <ToggleSwitch checked={checked} onChange={onChange} />
      </div>
    </div>
  );
}

export default App;

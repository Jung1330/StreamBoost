use crate::config::{AppConfig, GameProfile};
use crate::safety::is_protected_process;
use crate::win_api;
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::time::Duration;
use sysinfo::{MemoryRefreshKind, ProcessRefreshKind, ProcessesToUpdate, System};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    pub time: String,
    pub level: String, // "info" | "success" | "warn"
    pub message_tr: String,
    pub message_en: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunningProcessInfo {
    pub exe_name: String,
    pub instance_count: usize,
    pub memory_mb: u64,
    pub is_tracked_game: bool,
    pub is_background_app: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardStatus {
    pub boost_active: bool,
    pub manual_mode: bool,
    pub active_game_name: Option<String>,
    pub active_game_exe: Option<String>,
    pub cpu_usage: f32,
    pub ram_used_gb: f64,
    pub ram_total_gb: f64,
    pub ram_percent: f32,
    pub logical_cores: usize,
    pub high_priority_applied: bool,
    pub affinity_applied: bool,
    pub eco_qos_count: usize,
    pub eco_qos_apps: Vec<String>,
    pub power_plan_applied: bool,
    pub timer_applied: bool,
    pub logs: Vec<LogEntry>,
}

pub struct BoostRuntime {
    pub boost_active: bool,
    pub manual_mode: bool,
    pub active_game_id: Option<String>,
    pub active_game_name: Option<String>,
    pub active_game_exe: Option<String>,
    pub original_priorities: HashMap<u32, u32>,
    pub original_affinities: HashMap<u32, usize>,
    pub throttled_pids: HashMap<u32, String>,
    pub original_power_scheme: Option<String>,
    pub power_plan_applied: bool,
    pub timer_applied: bool,
    pub cpu_usage: f32,
    pub ram_used_gb: f64,
    pub ram_total_gb: f64,
    pub ram_percent: f32,
    pub logical_cores: usize,
    pub logs: Vec<LogEntry>,
}

impl BoostRuntime {
    pub fn new() -> Self {
        let mut rt = Self {
            boost_active: false,
            manual_mode: false,
            active_game_id: None,
            active_game_name: None,
            active_game_exe: None,
            original_priorities: HashMap::new(),
            original_affinities: HashMap::new(),
            throttled_pids: HashMap::new(),
            original_power_scheme: None,
            power_plan_applied: false,
            timer_applied: false,
            cpu_usage: 0.0,
            ram_used_gb: 0.0,
            ram_total_gb: 0.0,
            ram_percent: 0.0,
            logical_cores: win_api::get_logical_cpu_count(),
            logs: Vec::new(),
        };
        rt.push_log(
            "info",
            "StreamBoost izleyici başlatıldı. Oyun bekleniyor.",
            "StreamBoost monitor started. Waiting for games.",
        );
        rt
    }

    pub fn push_log(&mut self, level: &str, msg_tr: impl Into<String>, msg_en: impl Into<String>) {
        let entry = LogEntry {
            time: Local::now().format("%H:%M:%S").to_string(),
            level: level.to_string(),
            message_tr: msg_tr.into(),
            message_en: msg_en.into(),
        };
        self.logs.insert(0, entry);
        if self.logs.len() > 50 {
            self.logs.truncate(50);
        }
    }

    pub fn to_dashboard_status(&self) -> DashboardStatus {
        let mut eco_apps: Vec<String> = self
            .throttled_pids
            .values()
            .cloned()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        eco_apps.sort_by_key(|a| a.to_ascii_lowercase());

        DashboardStatus {
            boost_active: self.boost_active,
            manual_mode: self.manual_mode,
            active_game_name: self.active_game_name.clone(),
            active_game_exe: self.active_game_exe.clone(),
            cpu_usage: self.cpu_usage,
            ram_used_gb: self.ram_used_gb,
            ram_total_gb: self.ram_total_gb,
            ram_percent: self.ram_percent,
            logical_cores: self.logical_cores,
            high_priority_applied: !self.original_priorities.is_empty(),
            affinity_applied: !self.original_affinities.is_empty(),
            eco_qos_count: eco_apps.len(),
            eco_qos_apps: eco_apps,
            power_plan_applied: self.power_plan_applied,
            timer_applied: self.timer_applied,
            logs: self.logs.clone(),
        }
    }
}

pub struct AppState {
    pub config: Mutex<AppConfig>,
    pub runtime: Mutex<BoostRuntime>,
    pub stop_flag: AtomicBool,
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        Self {
            config: Mutex::new(config),
            runtime: Mutex::new(BoostRuntime::new()),
            stop_flag: AtomicBool::new(false),
        }
    }

    /// Starts the low-overhead background process monitor (polls every 2.5s, <0.5% CPU).
    pub fn start_monitor(self: &Arc<Self>) {
        let state = Arc::clone(self);
        std::thread::spawn(move || {
            let mut sys = System::new();
            sys.refresh_cpu_usage();
            sys.refresh_memory_specifics(MemoryRefreshKind::nothing().with_ram());

            while !state.stop_flag.load(Ordering::Relaxed) {
                let poll_ms = {
                    let cfg = state.config.lock().unwrap();
                    cfg.poll_interval_ms.clamp(1500, 10000)
                };

                // Lightweight refresh: only process names/PIDs, CPU %, and RAM.
                sys.refresh_cpu_usage();
                sys.refresh_memory_specifics(MemoryRefreshKind::nothing().with_ram());
                sys.refresh_processes_specifics(
                    ProcessesToUpdate::All,
                    true,
                    ProcessRefreshKind::nothing(),
                );

                state.tick_with_system(&sys);

                // Sleep in short slices so shutdown is responsive.
                let mut slept = 0u64;
                while slept < poll_ms && !state.stop_flag.load(Ordering::Relaxed) {
                    let step = (poll_ms - slept).min(250);
                    std::thread::sleep(Duration::from_millis(step));
                    slept += step;
                }
            }
        });
    }

    fn tick_with_system(&self, sys: &System) {
        let cfg = self.config.lock().unwrap().clone();
        let mut rt = self.runtime.lock().unwrap();

        // Update live telemetry
        let total_mem = sys.total_memory() as f64;
        let used_mem = sys.used_memory() as f64;
        rt.cpu_usage = (sys.global_cpu_usage() * 10.0).round() / 10.0;
        rt.ram_total_gb = ((total_mem / 1_073_741_824.0) * 10.0).round() / 10.0;
        rt.ram_used_gb = ((used_mem / 1_073_741_824.0) * 10.0).round() / 10.0;
        rt.ram_percent = if total_mem > 0.0 {
            (((used_mem / total_mem) * 100.0) as f32 * 10.0).round() / 10.0
        } else {
            0.0
        };

        // Build map of running processes by lowercase exe name -> Vec<pid>
        let mut running_by_exe: HashMap<String, Vec<u32>> = HashMap::new();
        for (pid, proc_) in sys.processes() {
            let name = proc_.name().to_string_lossy().to_string();
            let lower = name.to_ascii_lowercase();
            running_by_exe.entry(lower).or_default().push(pid.as_u32());
        }

        // Find first matching enabled game profile
        let detected_game: Option<(&GameProfile, Vec<u32>)> = cfg
            .game_profiles
            .iter()
            .filter(|g| g.enabled && !is_protected_process(&g.exe_name))
            .find_map(|g| {
                let key = g.exe_name.trim().to_ascii_lowercase();
                running_by_exe.get(&key).and_then(|pids| {
                    if pids.is_empty() {
                        None
                    } else {
                        Some((g, pids.clone()))
                    }
                })
            });

        if let Some((profile, pids)) = detected_game {
            if cfg.auto_detect || rt.manual_mode {
                let is_new_session = !rt.boost_active;
                let is_game_switch = rt.active_game_id.as_deref() != Some(profile.id.as_str());

                if is_new_session {
                    rt.boost_active = true;
                    rt.active_game_id = Some(profile.id.clone());
                    rt.active_game_name = Some(profile.name.clone());
                    rt.active_game_exe = Some(profile.exe_name.clone());
                    rt.push_log(
                        "success",
                        format!("{} algılandı — Boost Aktif edildi.", profile.name),
                        format!("{} detected — Boost Activated.", profile.name),
                    );
                    apply_system_boosts(&cfg, &mut rt);
                } else if is_game_switch {
                    rt.active_game_id = Some(profile.id.clone());
                    rt.active_game_name = Some(profile.name.clone());
                    rt.active_game_exe = Some(profile.exe_name.clone());
                }

                apply_game_optimizations(profile, &pids, &mut rt);
                apply_background_eco_qos(&cfg, &running_by_exe, Some(&profile.exe_name), &mut rt);
            }
        } else if rt.boost_active && !rt.manual_mode {
            // Game closed -> automatically restore everything!
            let closed_name = rt
                .active_game_name
                .clone()
                .unwrap_or_else(|| "Oyun".to_string());
            restore_all_locked(&mut rt);
            rt.push_log(
                "info",
                format!("{closed_name} kapandı — Tüm ayarlar eski haline döndürüldü."),
                format!("{closed_name} closed — All settings restored."),
            );
        } else if rt.boost_active && rt.manual_mode {
            // Keep manual boost fresh (throttle newly spawned background apps)
            rt.active_game_id = None;
            rt.active_game_name = None;
            rt.active_game_exe = None;
            apply_background_eco_qos(&cfg, &running_by_exe, None, &mut rt);
        }
    }

    pub fn toggle_manual_boost(&self) -> DashboardStatus {
        let cfg = self.config.lock().unwrap().clone();
        let mut sys = System::new();
        sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing(),
        );

        let mut rt = self.runtime.lock().unwrap();

        if rt.boost_active {
            restore_all_locked(&mut rt);
            rt.manual_mode = false;
            rt.push_log(
                "info",
                "Boost manuel olarak kapatıldı — Tüm ayarlar geri alındı.",
                "Boost disabled manually — All settings restored.",
            );
        } else {
            rt.boost_active = true;
            rt.manual_mode = true;
            rt.push_log(
                "success",
                "Manuel Boost aktif edildi.",
                "Manual Boost activated.",
            );
            apply_system_boosts(&cfg, &mut rt);

            let mut running_by_exe: HashMap<String, Vec<u32>> = HashMap::new();
            for (pid, proc_) in sys.processes() {
                let lower = proc_.name().to_string_lossy().to_ascii_lowercase();
                running_by_exe.entry(lower).or_default().push(pid.as_u32());
            }

            // If a tracked game is also running, optimize it right away.
            let mut active_game_exe: Option<String> = None;
            for profile in cfg
                .game_profiles
                .iter()
                .filter(|g| g.enabled && !is_protected_process(&g.exe_name))
            {
                let key = profile.exe_name.trim().to_ascii_lowercase();
                if let Some(pids) = running_by_exe.get(&key) {
                    if !pids.is_empty() {
                        rt.active_game_id = Some(profile.id.clone());
                        rt.active_game_name = Some(profile.name.clone());
                        rt.active_game_exe = Some(profile.exe_name.clone());
                        active_game_exe = Some(profile.exe_name.clone());
                        apply_game_optimizations(profile, pids, &mut rt);
                        break;
                    }
                }
            }

            apply_background_eco_qos(&cfg, &running_by_exe, active_game_exe.as_deref(), &mut rt);
        }

        rt.to_dashboard_status()
    }

    pub fn list_running_processes(&self) -> Vec<RunningProcessInfo> {
        let cfg = self.config.lock().unwrap().clone();
        let tracked_games: HashSet<String> = cfg
            .game_profiles
            .iter()
            .map(|g| g.exe_name.trim().to_ascii_lowercase())
            .collect();
        let bg_apps: HashSet<String> = cfg
            .background_apps
            .iter()
            .map(|b| b.exe_name.trim().to_ascii_lowercase())
            .collect();

        let mut sys = System::new();
        sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing().with_memory(),
        );

        let mut grouped: HashMap<String, (String, usize, u64)> = HashMap::new();
        for proc_ in sys.processes().values() {
            let name = proc_.name().to_string_lossy().to_string();
            if is_protected_process(&name) {
                continue;
            }
            let lower = name.to_ascii_lowercase();
            let entry = grouped.entry(lower).or_insert((name, 0, 0));
            entry.1 += 1;
            entry.2 += proc_.memory();
        }

        let mut list: Vec<RunningProcessInfo> = grouped
            .into_iter()
            .map(|(lower, (exe_name, count, mem_bytes))| RunningProcessInfo {
                exe_name,
                instance_count: count,
                memory_mb: mem_bytes / (1024 * 1024),
                is_tracked_game: tracked_games.contains(&lower),
                is_background_app: bg_apps.contains(&lower),
            })
            .collect();

        list.sort_by(|a, b| {
            b.memory_mb
                .cmp(&a.memory_mb)
                .then_with(|| a.exe_name.to_ascii_lowercase().cmp(&b.exe_name.to_ascii_lowercase()))
        });
        list
    }

    /// Stops monitor and restores all OS/process settings synchronously before exit.
    pub fn shutdown_and_restore(&self) {
        self.stop_flag.store(true, Ordering::Relaxed);
        if let Ok(mut rt) = self.runtime.lock() {
            if rt.boost_active {
                restore_all_locked(&mut rt);
            }
        }
    }
}

fn apply_system_boosts(cfg: &AppConfig, rt: &mut BoostRuntime) {
    // Power Plan
    if cfg.power_plan_boost && !rt.power_plan_applied {
        if let Ok(orig_guid) = win_api::get_active_power_scheme() {
            rt.original_power_scheme = Some(orig_guid.clone());
            if orig_guid.eq_ignore_ascii_case(win_api::HIGH_PERFORMANCE_GUID)
                || orig_guid.eq_ignore_ascii_case(win_api::ULTIMATE_PERFORMANCE_GUID)
            {
                rt.power_plan_applied = true;
            } else if win_api::set_high_performance_power_scheme().is_ok() {
                rt.power_plan_applied = true;
                rt.push_log(
                    "success",
                    "Güç planı Yüksek Performans olarak ayarlandı.",
                    "Power plan switched to High Performance.",
                );
            } else {
                rt.push_log(
                    "warn",
                    "Yüksek Performans güç planı bu sistemde etkinleştirilemedi.",
                    "High Performance power plan could not be activated on this system.",
                );
            }
        }
    }

    // 1ms High-Resolution System Timer
    if cfg.high_res_timer && !rt.timer_applied && win_api::enable_high_res_timer() {
        rt.timer_applied = true;
        rt.push_log(
            "info",
            "Yüksek çözünürlüklü zamanlayıcı (1ms) etkinleştirildi.",
            "High-resolution system timer (1ms) enabled.",
        );
    }
}

fn apply_game_optimizations(profile: &GameProfile, pids: &[u32], rt: &mut BoostRuntime) {
    if is_protected_process(&profile.exe_name) {
        return;
    }

    for &pid in pids {
        // High Priority (never REALTIME)
        if profile.high_priority && !rt.original_priorities.contains_key(&pid) {
            if let Ok(prev_prio) = win_api::set_process_high_priority(pid) {
                rt.original_priorities.insert(pid, prev_prio);
            }
        }

        // Optional CPU Affinity
        if let Some(ref cores) = profile.affinity_cores {
            if !cores.is_empty() && !rt.original_affinities.contains_key(&pid) {
                if let Ok(prev_mask) = win_api::set_process_affinity(pid, cores) {
                    rt.original_affinities.insert(pid, prev_mask);
                }
            }
        }
    }
}

fn apply_background_eco_qos(
    cfg: &AppConfig,
    running_by_exe: &HashMap<String, Vec<u32>>,
    active_game_exe: Option<&str>,
    rt: &mut BoostRuntime,
) {
    if !cfg.eco_qos_enabled {
        return;
    }

    let active_game_lower = active_game_exe.map(|s| s.trim().to_ascii_lowercase());
    let mut newly_throttled = 0usize;

    for app in cfg.background_apps.iter().filter(|a| a.enabled) {
        if is_protected_process(&app.exe_name) {
            continue;
        }
        let lower = app.exe_name.trim().to_ascii_lowercase();
        if active_game_lower.as_deref() == Some(lower.as_str()) {
            continue;
        }

        if let Some(pids) = running_by_exe.get(&lower) {
            for &pid in pids {
                if !rt.throttled_pids.contains_key(&pid) {
                    if win_api::set_process_eco_qos(pid, true).is_ok() {
                        rt.throttled_pids.insert(pid, app.exe_name.clone());
                        newly_throttled += 1;
                    }
                }
            }
        }
    }

    // Prune dead PIDs from throttled_pids
    let live_pids: HashSet<u32> = running_by_exe
        .values()
        .flat_map(|v| v.iter().copied())
        .collect();
    rt.throttled_pids.retain(|pid, _| live_pids.contains(pid));

    if newly_throttled > 0 {
        rt.push_log(
            "info",
            format!("{newly_throttled} arka plan süreci Verimlilik Moduna (EcoQoS) alındı."),
            format!("{newly_throttled} background process(es) moved to Efficiency Mode (EcoQoS)."),
        );
    }
}

fn restore_all_locked(rt: &mut BoostRuntime) {
    // 1. Restore game priorities
    for (pid, orig_prio) in rt.original_priorities.drain() {
        let _ = win_api::restore_process_priority(pid, orig_prio);
    }

    // 2. Restore game affinities
    for (pid, orig_mask) in rt.original_affinities.drain() {
        let _ = win_api::restore_process_affinity(pid, orig_mask);
    }

    // 3. Disable EcoQoS on all throttled background processes
    for (pid, _) in rt.throttled_pids.drain() {
        let _ = win_api::set_process_eco_qos(pid, false);
    }

    // 4. Restore original power plan
    if rt.power_plan_applied {
        if let Some(orig_guid) = rt.original_power_scheme.take() {
            let _ = win_api::set_power_scheme(&orig_guid);
        }
        rt.power_plan_applied = false;
    }

    // 5. Disable 1ms high-res timer
    if rt.timer_applied {
        win_api::disable_high_res_timer();
        rt.timer_applied = false;
    }

    rt.boost_active = false;
    rt.active_game_id = None;
    rt.active_game_name = None;
    rt.active_game_exe = None;
}

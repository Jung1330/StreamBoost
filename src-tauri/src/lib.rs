mod booster;
mod config;
mod safety;
mod win_api;

use booster::{AppState, DashboardStatus, RunningProcessInfo};
use config::{AppConfig, BackgroundApp, GameProfile};
use std::sync::Arc;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, RunEvent, WindowEvent,
};

#[tauri::command]
fn get_status(state: tauri::State<'_, Arc<AppState>>) -> DashboardStatus {
    let rt = state.runtime.lock().unwrap();
    rt.to_dashboard_status()
}

#[tauri::command]
fn get_config(state: tauri::State<'_, Arc<AppState>>) -> AppConfig {
    state.config.lock().unwrap().clone()
}

#[tauri::command]
fn save_config(
    state: tauri::State<'_, Arc<AppState>>,
    new_config: AppConfig,
) -> Result<AppConfig, String> {
    config::save_config(&new_config)?;
    let mut cfg = state.config.lock().unwrap();
    *cfg = new_config.clone();
    Ok(new_config)
}

#[tauri::command]
fn toggle_manual_boost(state: tauri::State<'_, Arc<AppState>>) -> DashboardStatus {
    state.toggle_manual_boost()
}

#[tauri::command]
fn list_running_processes(state: tauri::State<'_, Arc<AppState>>) -> Vec<RunningProcessInfo> {
    state.list_running_processes()
}

#[tauri::command]
fn add_game_profile(
    state: tauri::State<'_, Arc<AppState>>,
    name: String,
    exe_name: String,
) -> Result<AppConfig, String> {
    let trimmed_exe = exe_name.trim().to_string();
    if trimmed_exe.is_empty() {
        return Err("Executable name cannot be empty".into());
    }
    if safety::is_protected_process(&trimmed_exe) {
        return Err("Critical Windows system processes cannot be added.".into());
    }

    let mut cfg = state.config.lock().unwrap();
    if cfg
        .game_profiles
        .iter()
        .any(|g| g.exe_name.eq_ignore_ascii_case(&trimmed_exe))
    {
        return Err("This game executable is already in the list.".into());
    }

    let clean_name = if name.trim().is_empty() {
        trimmed_exe
            .strip_suffix(".exe")
            .or_else(|| trimmed_exe.strip_suffix(".EXE"))
            .unwrap_or(&trimmed_exe)
            .to_string()
    } else {
        name.trim().to_string()
    };

    let id = format!(
        "game_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
    );

    cfg.game_profiles.insert(
        0,
        GameProfile {
            id,
            name: clean_name,
            exe_name: trimmed_exe,
            enabled: true,
            high_priority: true,
            affinity_cores: None,
        },
    );

    config::save_config(&cfg)?;
    Ok(cfg.clone())
}

#[tauri::command]
fn update_game_profile(
    state: tauri::State<'_, Arc<AppState>>,
    profile: GameProfile,
) -> Result<AppConfig, String> {
    if safety::is_protected_process(&profile.exe_name) {
        return Err("Critical Windows system processes cannot be modified.".into());
    }

    let mut cfg = state.config.lock().unwrap();
    if let Some(existing) = cfg.game_profiles.iter_mut().find(|g| g.id == profile.id) {
        *existing = profile;
    } else {
        return Err("Game profile not found.".into());
    }

    config::save_config(&cfg)?;
    Ok(cfg.clone())
}

#[tauri::command]
fn remove_game_profile(
    state: tauri::State<'_, Arc<AppState>>,
    id: String,
) -> Result<AppConfig, String> {
    let mut cfg = state.config.lock().unwrap();
    cfg.game_profiles.retain(|g| g.id != id);
    config::save_config(&cfg)?;
    Ok(cfg.clone())
}

#[tauri::command]
fn add_background_app(
    state: tauri::State<'_, Arc<AppState>>,
    exe_name: String,
    label: String,
) -> Result<AppConfig, String> {
    let trimmed_exe = exe_name.trim().to_string();
    if trimmed_exe.is_empty() {
        return Err("Executable name cannot be empty".into());
    }
    if safety::is_protected_process(&trimmed_exe) {
        return Err("Critical Windows system processes cannot be throttled.".into());
    }

    let mut cfg = state.config.lock().unwrap();
    if cfg
        .background_apps
        .iter()
        .any(|b| b.exe_name.eq_ignore_ascii_case(&trimmed_exe))
    {
        return Err("This background app is already in the list.".into());
    }

    let clean_label = if label.trim().is_empty() {
        trimmed_exe
            .strip_suffix(".exe")
            .or_else(|| trimmed_exe.strip_suffix(".EXE"))
            .unwrap_or(&trimmed_exe)
            .to_string()
    } else {
        label.trim().to_string()
    };

    cfg.background_apps.insert(
        0,
        BackgroundApp {
            exe_name: trimmed_exe,
            label: clean_label,
            enabled: true,
        },
    );

    config::save_config(&cfg)?;
    Ok(cfg.clone())
}

#[tauri::command]
fn remove_background_app(
    state: tauri::State<'_, Arc<AppState>>,
    exe_name: String,
) -> Result<AppConfig, String> {
    let mut cfg = state.config.lock().unwrap();
    cfg.background_apps
        .retain(|b| !b.exe_name.eq_ignore_ascii_case(&exe_name));
    config::save_config(&cfg)?;
    Ok(cfg.clone())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let initial_config = config::load_config();
    let app_state = Arc::new(AppState::new(initial_config));
    app_state.start_monitor();

    let state_for_exit = Arc::clone(&app_state);

    let app = tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            get_status,
            get_config,
            save_config,
            toggle_manual_boost,
            list_running_processes,
            add_game_profile,
            update_game_profile,
            remove_game_profile,
            add_background_app,
            remove_background_app,
        ])
        .setup(|app| {
            // System Tray setup
            let show_item = MenuItem::with_id(app, "show", "StreamBoost'u Göster / Show", true, None::<&str>)?;
            let toggle_item = MenuItem::with_id(app, "toggle", "Boost Aç / Kapat (Toggle)", true, None::<&str>)?;
            let sep = PredefinedMenuItem::separator(app)?;
            let quit_item = MenuItem::with_id(app, "quit", "Çıkış (Ayarları Geri Al) / Quit", true, None::<&str>)?;

            let menu = Menu::with_items(app, &[&show_item, &toggle_item, &sep, &quit_item])?;

            let mut tray_builder = TrayIconBuilder::with_id("streamboost-tray")
                .menu(&menu)
                .tooltip("StreamBoost — Game Performance Manager")
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.unminimize();
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "toggle" => {
                        if let Some(state) = app.try_state::<Arc<AppState>>() {
                            let _ = state.toggle_manual_boost();
                        }
                    }
                    "quit" => {
                        if let Some(state) = app.try_state::<Arc<AppState>>() {
                            state.shutdown_and_restore();
                        }
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.unminimize();
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                });

            if let Some(icon) = app.default_window_icon() {
                tray_builder = tray_builder.icon(icon.clone());
            }

            let _tray = tray_builder.build(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            // Closing window hides to system tray instead of quitting
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .build(tauri::generate_context!())
        .expect("error while building StreamBoost application");

    app.run(move |_app_handle, event| {
        if let RunEvent::Exit = event {
            state_for_exit.shutdown_and_restore();
        }
    });
}

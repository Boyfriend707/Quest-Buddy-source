use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;
use tauri::Emitter;
use tauri::Manager;
use serde_json::Value;

mod plugins;
mod tray;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Progress {
    pub game: String,
    pub running: bool,
    pub percentage: f32,
    pub geo: i64,
    pub items: Vec<String>,
    pub next_steps: Vec<String>,
    pub play_time_formatted: String,
    pub max_health: u32,
    pub soul_vessels: u32,
    pub dream_essence: u32,
    pub grubs: u32,
    pub nail_damage: u32,
    pub nail_name: String,
    pub bosses: Vec<String>,
    pub charms_count: u32,
    pub has_save: bool,
}

fn format_play_time(secs: f64) -> String {
    let total_secs = secs as u64;
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}

fn nail_name(level: u32) -> &'static str {
    match level {
        0 => "Old Nail",
        1 => "Sharpened Nail",
        2 => "Channelled Nail",
        3 => "Coiled Nail",
        _ => "Pure Nail",
    }
}

fn build_progress(
    game: String,
    running: bool,
    save: &plugins::hollow_knight::SaveData,
    items: Vec<String>,
    next_steps: Vec<String>,
) -> Progress {
    Progress {
        game,
        running,
        percentage: save.completion_percentage as f32,
        geo: save.geo,
        items,
        next_steps,
        play_time_formatted: format_play_time(save.play_time),
        max_health: save.max_health,
        soul_vessels: save.soul_vessels,
        dream_essence: save.dream_essence,
        grubs: save.grub_count,
        nail_damage: save.nail_damage,
        nail_name: nail_name(save.nail_damage).into(),
        bosses: save.bosses_defeated.clone(),
        charms_count: save.charms_count,
        has_save: true,
    }
}

fn build_progress_ss(
    game: String,
    running: bool,
    save: &plugins::silksong::SaveData,
    items: Vec<String>,
    next_steps: Vec<String>,
) -> Progress {
    Progress {
        game,
        running,
        percentage: save.completion_percentage as f32,
        geo: save.geo,
        items,
        next_steps,
        play_time_formatted: format_play_time(save.play_time),
        max_health: save.max_health,
        soul_vessels: save.soul_vessels,
        dream_essence: save.dream_essence,
        grubs: save.grub_count,
        nail_damage: save.nail_damage,
        nail_name: "—".into(),
        bosses: save.bosses_defeated.clone(),
        charms_count: save.charms_count,
        has_save: true,
    }
}

fn get_current_progress() -> Progress {
    if plugins::hollow_knight::is_running() {
        if let Some(save) = plugins::hollow_knight::load_most_recent_save() {
            return build_progress("Hollow Knight".into(), true, &save, hk_items(&save), hk_next_steps(&save));
        }
    }
    if plugins::silksong::is_running() {
        if let Some(save) = plugins::silksong::load_most_recent_save() {
            return build_progress_ss("Silksong".into(), true, &save, ss_items(&save), ss_next_steps(&save));
        }
    }
    if let Some(save) = plugins::hollow_knight::load_most_recent_save() {
        return build_progress("Hollow Knight".into(), false, &save, hk_items(&save), hk_next_steps(&save));
    }
    if let Some(save) = plugins::silksong::load_most_recent_save() {
        return build_progress_ss("Silksong".into(), false, &save, ss_items(&save), ss_next_steps(&save));
    }
    Progress {
        game: "Hollow Knight".into(),
        running: false,
        percentage: 0.0,
        geo: 0,
        items: vec!["No save file found".into()],
        next_steps: vec!["Start a new game!".into()],
        play_time_formatted: "—".into(),
        max_health: 0,
        soul_vessels: 0,
        dream_essence: 0,
        grubs: 0,
        nail_damage: 0,
        nail_name: "—".into(),
        bosses: vec![],
        charms_count: 0,
        has_save: false,
    }
}

fn hk_items(_data: &plugins::hollow_knight::SaveData) -> Vec<String> {
    let mut items = Vec::new();
    if _data.has_dash {
        items.push("Mothwing Cloak".into());
    }
    if _data.has_wall_jump {
        items.push("Mantis Claw".into());
    }
    if _data.has_double_jump {
        items.push("Monarch Wings".into());
    }
    if _data.has_super_jump {
        items.push("Crystal Heart".into());
    }
    if _data.has_acid_armour {
        items.push("Isma's Tear".into());
    }
    if _data.has_dream_nail {
        items.push("Dream Nail".into());
    }
    items
}

fn hk_next_steps(data: &plugins::hollow_knight::SaveData) -> Vec<String> {
    let mut steps = Vec::new();
    if !data.has_wall_jump {
        steps.push("Get Mantis Claw (wall jump) from Fungal Wastes".into());
    }
    if !data.has_dash {
        steps.push("Get Mothwing Cloak (dash) from Greenpath".into());
    }
    if !data.has_double_jump {
        steps.push("Get Monarch Wings (double jump) from Kingdom's Edge".into());
    }
    if !data.has_super_jump {
        steps.push("Get Crystal Heart (super dash) from Crystal Peak".into());
    }
    if !data.has_acid_armour {
        steps.push("Get Isma's Tear (acid immunity) from Royal Waterways".into());
    }
    if !data.has_dream_nail {
        steps.push("Get Dream Nail from Resting Grounds".into());
    }
    if steps.is_empty() {
        steps.push("Explore the endgame \u{2014} aim for 100% completion!".into());
    }
    steps
}

fn ss_items(data: &plugins::silksong::SaveData) -> Vec<String> {
    let mut items = Vec::new();
    if data.has_dash {
        items.push("Dash".into());
    }
    if data.has_wall_jump {
        items.push("Wall Jump".into());
    }
    if data.has_double_jump {
        items.push("Double Jump".into());
    }
    items
}

fn ss_next_steps(data: &plugins::silksong::SaveData) -> Vec<String> {
    let mut steps = Vec::new();
    if !data.has_dash {
        steps.push("Find the Dash ability".into());
    }
    if !data.has_wall_jump {
        steps.push("Find the Wall Jump ability".into());
    }
    if !data.has_double_jump {
        steps.push("Find the Double Jump ability".into());
    }
    if steps.is_empty() {
        steps.push("Keep exploring!".into());
    }
    steps
}

fn most_recent_save_raw() -> Option<(PathBuf, Vec<u8>)> {
    let dir = plugins::hollow_knight::save_path().or_else(plugins::silksong::save_path)?;
    let mut saves: Vec<PathBuf> = dir
        .read_dir()
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name();
            let name = name.to_string_lossy();
            name.ends_with(".dat") && name.starts_with("user")
        })
        .map(|e| e.path())
        .collect();
    saves.sort_by_key(|p| {
        std::fs::metadata(p)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
    });
    let path = saves.last()?.clone();
    let raw = std::fs::read(&path).ok()?;
    Some((path, raw))
}

#[tauri::command]
fn get_progress() -> Progress {
    get_current_progress()
}

#[tauri::command]
fn set_auto_start(enabled: bool) -> Result<(), String> {
    use winreg::enums::*;
    use winreg::RegKey;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu
        .open_subkey_with_flags(r"Software\Microsoft\Windows\CurrentVersion\Run", KEY_SET_VALUE)
        .map_err(|e| e.to_string())?;
    if enabled {
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        key.set_value("QuestBuddy", &exe.to_string_lossy().to_string())
            .map_err(|e| e.to_string())?;
    } else {
        key.delete_value("QuestBuddy").map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn get_auto_start() -> bool {
    use winreg::enums::*;
    use winreg::RegKey;
    if let Ok(hkcu) = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags(r"Software\Microsoft\Windows\CurrentVersion\Run", KEY_READ)
    {
        if let Ok(_) = hkcu.get_value::<String, _>("QuestBuddy") {
            return true;
        }
    }
    false
}

#[tauri::command]
fn get_save_fields() -> Result<Value, String> {
    let (_, raw) = most_recent_save_raw().ok_or("No save file found")?;
    let json = plugins::save_crypto::decrypt_save(&raw).ok_or("Failed to decrypt save")?;
    let root: Value = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    let pd = root.get("playerData").ok_or("No playerData in save")?;
    Ok(pd.clone())
}

#[tauri::command]
fn write_save_field(field: String, value: Value) -> Result<(), String> {
    let (path, raw) = most_recent_save_raw().ok_or("No save file found")?;
    let json = plugins::save_crypto::decrypt_save(&raw).ok_or("Failed to decrypt save")?;
    let mut root: Value = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    if let Some(pd) = root.get_mut("playerData") {
        if let Some(obj) = pd.as_object_mut() {
            obj.insert(field, value);
        }
    }
    let modified = serde_json::to_string(&root).map_err(|e| e.to_string())?;
    let encrypted = plugins::save_crypto::encrypt_save(&modified, &raw).ok_or("Failed to encrypt save")?;
    std::fs::write(&path, encrypted).map_err(|e| e.to_string())?;
    Ok(())
}

fn start_watcher(handle: tauri::AppHandle) {
    let (tx, rx) = mpsc::channel();

    let mut watcher = RecommendedWatcher::new(tx, Config::default()).expect("failed to create watcher");

    let dirs_to_watch = [
        plugins::hollow_knight::save_path(),
        plugins::silksong::save_path(),
    ];

    let watched_dirs: Vec<PathBuf> = dirs_to_watch.into_iter().flatten().collect();
    for dir in &watched_dirs {
        let _ = watcher.watch(dir, RecursiveMode::NonRecursive);
    }

    let handle_clone = handle.clone();

    std::thread::spawn(move || {
        let mut was_running = false;

        loop {
            let progress = get_current_progress();

            if progress.running != was_running {
                was_running = progress.running;
                let _ = handle_clone.emit(
                    "game-status-changed",
                    serde_json::json!({ "running": progress.running }),
                );
            }

            let _ = handle_clone.emit("progress-updated", progress.clone());

            let deadline = std::time::Instant::now() + Duration::from_secs(5);

            loop {
                let remaining = deadline.saturating_duration_since(std::time::Instant::now());
                if remaining.is_zero() {
                    break;
                }
                match rx.recv_timeout(remaining) {
                    Ok(Ok(event)) => {
                        if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                            break;
                        }
                    }
                    _ => break,
                }
            }
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .setup(|app| {
            tray::create_tray(app)?;
            start_watcher(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_progress,
            set_auto_start,
            get_auto_start,
            get_save_fields,
            write_save_field,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|_app_handle, _event| {});
}

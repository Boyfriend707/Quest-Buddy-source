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
pub struct NextStep {
    pub title: String,
    pub detail: String,
}

impl NextStep {
    fn new(title: impl Into<String>, detail: impl Into<String>) -> Self {
        NextStep {
            title: title.into(),
            detail: detail.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Progress {
    pub game: String,
    pub running: bool,
    pub percentage: f32,
    pub geo: i64,
    pub items: Vec<String>,
    pub next_steps: Vec<NextStep>,
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
    next_steps: Vec<NextStep>,
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
    next_steps: Vec<NextStep>,
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
        next_steps: vec![NextStep::new(
            "Start a new game!",
            "Create a save in Hollow Knight and begin your journey through Hallownest.",
        )],
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

fn hk_next_steps(data: &plugins::hollow_knight::SaveData) -> Vec<NextStep> {
    let mut steps = Vec::new();

    if !data.has_dash {
        steps.push(NextStep::new(
            "Get the Mothwing Cloak (Dash)",
            "Reach Greenpath and defeat Hornet for the first time. The dash lets you cross gaps and open up the world.",
        ));
    }
    if !data.has_wall_jump {
        steps.push(NextStep::new(
            "Get the Mantis Claw (Wall Jump)",
            "Travel to the Mantis Village in the Fungal Wastes and win the trial to earn the wall jump — you'll need it to climb most of Hallownest.",
        ));
    }
    if !data.has_super_jump {
        steps.push(NextStep::new(
            "Get the Crystal Heart (Super Dash)",
            "In Crystal Peak, take the west path past the Crystal Guardian's room to claim the super dash and blaze across open areas.",
        ));
    }
    if !data.has_acid_armour {
        steps.push(NextStep::new(
            "Get Isma's Tear (Acid Immunity)",
            "Explore the Royal Waterways and follow the southern route to Isma's Grove to earn acid immunity and safely cross acid lakes.",
        ));
    }
    if !data.has_double_jump {
        steps.push(NextStep::new(
            "Get the Monarch Wings (Double Jump)",
            "Defeat Broken Vessel in the Ancient Basin, then climb the ruined path into Kingdom's Edge to claim the wings.",
        ));
    }
    if !data.has_dream_nail {
        steps.push(NextStep::new(
            "Get the Dream Nail",
            "After gaining Soul powers, visit the Seer in the Resting Grounds to receive the Dream Nail — it reveals hidden lore, dream bosses, and the true ending.",
        ));
    }

    if data.has_dream_nail && data.dream_essence < 2400 {
        steps.push(NextStep::new(
            "Awaken the Dream Nail",
            format!(
                "Collect dream essence ({}/2400) from dream warriors, dream bosses, and glowing trees to fully awaken it and unlock the final ending.",
                data.dream_essence
            ),
        ));
    }
    if data.grub_count > 0 && data.grub_count < 46 {
        steps.push(NextStep::new(
            "Rescue the Remaining Grubs",
            format!(
                "You've freed {} of 46 grubs. Find the rest and return them to the Grubfather in the Forgotten Crossroads for rewards.",
                data.grub_count
            ),
        ));
    }
    if data.charms_count > 0 && data.charms_count < 40 {
        steps.push(NextStep::new(
            "Collect More Charms",
            format!(
                "You own {} of 40 charms. Buy from shops, explore secret rooms, and defeat optional bosses to complete the set.",
                data.charms_count
            ),
        ));
    }

    steps.extend(hk_boss_steps(data));

    if steps.is_empty() {
        steps.push(NextStep::new(
            "Challenge the Radiance",
            "Face the Hollow Knight in the Black Egg, then strike the Radiance with the Dream Nail for the final showdown.",
        ));
        if data.completion_percentage < 112.0 {
            steps.push(NextStep::new(
                "Hunt for 112% Completion",
                "Hollow Knight maxes out at 112% with the DLC. Track down remaining grubs, charms, dream bosses, and Godhome content.",
            ));
        }
    }

    steps
}

enum BossGate {
    Always,
    Dream,
    Endgame,
}

const HK_BOSS_HINTS: &[(&str, BossGate, &str, &str)] = &[
    (
        "Soul Master",
        BossGate::Always,
        "Defeat the Soul Master",
        "Climb the Soul Sanctum in the City of Tears. Beating him grants Desolate Dive and unlocks the city's shortcuts.",
    ),
    (
        "Dung Defender",
        BossGate::Always,
        "Defeat the Dung Defender",
        "Find him in the Royal Waterways — he guards the path toward the Beast's Den and the second Dreamer.",
    ),
    (
        "Grey Prince Zote",
        BossGate::Dream,
        "Beat Grey Prince Zote",
        "A dream boss fought in Bretta's house in Dirtmouth. Beware — he gets stronger with every rematch.",
    ),
    (
        "Traitor Lord",
        BossGate::Endgame,
        "Defeat the Traitor Lord",
        "In the deepest part of Queen's Gardens. Beating him opens a path to the White Lady and an ending piece.",
    ),
    (
        "Troupe Master Grimm",
        BossGate::Endgame,
        "Summon and defeat Troupe Master Grimm",
        "Burn the three Grimmkin flames to summon the Grimm Troupe's master in the Howling Cliffs and Dirtmouth.",
    ),
    (
        "Nailsage Sly",
        BossGate::Endgame,
        "Defeat Nailsage Sly",
        "Dream-fight Sly in the shed beneath the Crossroads' Nailsmith. He's blindingly fast — attack once, then dodge.",
    ),
    (
        "Paintmaster Sheo",
        BossGate::Endgame,
        "Defeat Paintmaster Sheo",
        "Dream-fight Sheo in his workshop above the Fungal Wastes. Three graceful phases with a big paintbrush.",
    ),
    (
        "White Defender",
        BossGate::Endgame,
        "Defeat the White Defender",
        "Dream-fight Dung Defender in the Royal Waterways. Every win earns a fragment of a pale secret.",
    ),
    (
        "Hive Knight",
        BossGate::Endgame,
        "Defeat the Hive Knight",
        "At the heart of the Hive, past Queen's Gardens. Defeating him secures the Hiveblood charm.",
    ),
    (
        "Oblobbles",
        BossGate::Endgame,
        "Defeat the Oblobbles",
        "Two floating juggernauts in Queen's Gardens — keep moving to slip between their spit volleys.",
    ),
    (
        "Nightmare King Grimm",
        BossGate::Endgame,
        "Defeat Nightmare King Grimm",
        "The true form of the Grimm Troupe's master. A brutal dance — learn the pufferfish pattern and the fight is yours.",
    ),
];

fn hk_boss_steps(data: &plugins::hollow_knight::SaveData) -> Vec<NextStep> {
    let undefeated: Vec<&str> = plugins::hollow_knight::undefeated_bosses(data);
    let mut steps = Vec::new();
    for (name, gate, title, detail) in HK_BOSS_HINTS {
        if steps.len() >= 3 {
            break;
        }
        if !undefeated.contains(&name) {
            continue;
        }
        let unlocked = match gate {
            BossGate::Always => true,
            BossGate::Dream => data.has_dream_nail,
            BossGate::Endgame => data.has_double_jump,
        };
        if unlocked {
            steps.push(NextStep::new(*title, *detail));
        }
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

fn ss_next_steps(data: &plugins::silksong::SaveData) -> Vec<NextStep> {
    let mut steps = Vec::new();
    if !data.has_dash {
        steps.push(NextStep::new(
            "Find the Dash",
            "Push through Pharloom's early areas — the Dash is the first big movement upgrade and lets you close gaps.",
        ));
    }
    if !data.has_wall_jump {
        steps.push(NextStep::new(
            "Find the Wall Jump",
            "Search Pharloom's ancient structures for the Wall Jump to climb vertical spaces and reach new heights.",
        ));
    }
    if !data.has_double_jump {
        steps.push(NextStep::new(
            "Find the Double Jump",
            "Explore the deeper reaches of Pharloom to find the Double Jump and reach high platforms.",
        ));
    }
    if steps.is_empty() {
        steps.push(NextStep::new(
            "Keep Exploring",
            "With all movement abilities in hand, hunt down hidden rooms, collectibles, and bosses toward 100% completion.",
        ));
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
            plugins::save_crypto::is_save_slot(&name)
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
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
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

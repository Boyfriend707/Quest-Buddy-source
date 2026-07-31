use super::save_crypto;
use serde_json::Value;
#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[derive(Debug)]
pub struct SaveData {
    pub geo: i64,
    pub completion_percentage: f64,
    pub has_dash: bool,
    pub has_double_jump: bool,
    pub has_wall_jump: bool,
    pub play_time: f64,
    pub max_health: u32,
    pub soul_vessels: u32,
    pub dream_essence: u32,
    pub grub_count: u32,
    pub nail_damage: u32,
    pub bosses_defeated: Vec<String>,
    pub charms_count: u32,
}

pub fn is_running() -> bool {
    let process_name = "Silksong.exe";
    std::process::Command::new("tasklist")
        .arg("/FI")
        .arg(format!("IMAGENAME eq {}", process_name))
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output()
        .ok()
        .map(|o| {
            let out = String::from_utf8_lossy(&o.stdout);
            out.contains(process_name)
        })
        .unwrap_or(false)
}

pub fn save_path() -> Option<std::path::PathBuf> {
    if let Ok(profile) = std::env::var("USERPROFILE") {
        let path = std::path::PathBuf::from(profile)
            .join("AppData")
            .join("LocalLow")
            .join("Team Cherry")
            .join("Silksong");
        if path.exists() {
            return Some(path);
        }
    }
    let base = dirs::data_local_dir()?;
    let path = base
        .join("..")
        .join("LocalLow")
        .join("Team Cherry")
        .join("Silksong");
    if path.exists() {
        return Some(path);
    }
    path.canonicalize().ok()
}

pub fn load_most_recent_save() -> Option<SaveData> {
    let dir = save_path()?;
    let mut saves: Vec<std::path::PathBuf> = dir
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
    saves.last().and_then(|p| load_save(&p))
}

pub fn load_save(path: &std::path::Path) -> Option<SaveData> {
    let raw = std::fs::read(path).ok()?;
    let json = save_crypto::decrypt_save(&raw)?;
    parse_json(&json)
}

fn parse_json(json: &str) -> Option<SaveData> {
    let root: Value = serde_json::from_str(json).ok()?;
    let pd = root.get("playerData")?;

    Some(SaveData {
        geo: pd.get("geo").and_then(|v| v.as_i64()).unwrap_or(0),
        completion_percentage: pd
            .get("completionPercentage")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        has_dash: pd.get("hasDash").and_then(|v| v.as_bool()).unwrap_or(false),
        has_double_jump: pd
            .get("hasDoubleJump")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        has_wall_jump: pd
            .get("hasWallJump")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        play_time: pd.get("playTime").and_then(|v| v.as_f64()).unwrap_or(0.0),
        max_health: 5,
        soul_vessels: 0,
        dream_essence: 0,
        grub_count: 0,
        nail_damage: 0,
        bosses_defeated: Vec::new(),
        charms_count: 0,
    })
}

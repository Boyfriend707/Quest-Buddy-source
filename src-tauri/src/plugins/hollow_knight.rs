use super::save_crypto;
use serde_json::Value;
#[cfg(windows)]
use std::os::windows::process::CommandExt;

const BOSS_KILL_FLAGS: &[(&str, &str)] = &[
    ("killedFalseKnight", "False Knight"),
    ("killedHornet", "Hornet"),
    ("killedMantisLord", "Mantis Lords"),
    ("killedMageLord", "Soul Master"),
    ("killedHollowKnight", "The Hollow Knight"),
    ("killedFinalBoss", "The Radiance"),
    ("killedGreyPrince", "Grey Prince Zote"),
    ("killedGrimm", "Troupe Master Grimm"),
    ("killedNightmareGrimm", "Nightmare King Grimm"),
    ("killedHiveKnight", "Hive Knight"),
    ("killedTraitorLord", "Traitor Lord"),
    ("killedWhiteDefender", "White Defender"),
    ("killedNailsage", "Nailsage Sly"),
    ("killedPaintmaster", "Paintmaster Sheo"),
    ("killedDungDefender", "Dung Defender"),
    ("killedOblobble", "Oblobbles"),
];

#[derive(Debug)]
pub struct SaveData {
    pub geo: i64,
    pub completion_percentage: f64,
    pub has_dash: bool,
    pub has_super_jump: bool,
    pub has_double_jump: bool,
    pub has_wall_jump: bool,
    pub has_acid_armour: bool,
    pub has_dream_nail: bool,
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
    let process_name = "HollowKnight.exe";
    let output = std::process::Command::new("tasklist")
        .arg("/FI")
        .arg(format!("IMAGENAME eq {}", process_name))
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output()
        .ok();
    match output {
        Some(o) => String::from_utf8_lossy(&o.stdout).contains(process_name),
        None => false,
    }
}

pub fn save_path() -> Option<std::path::PathBuf> {
    if let Ok(profile) = std::env::var("USERPROFILE") {
        let path = std::path::PathBuf::from(profile)
            .join("AppData")
            .join("LocalLow")
            .join("Team Cherry")
            .join("Hollow Knight");
        if path.exists() {
            return Some(path);
        }
    }
    let base = dirs::data_local_dir()?;
    let path = base
        .join("..")
        .join("LocalLow")
        .join("Team Cherry")
        .join("Hollow Knight");
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

    let mut bosses_defeated = Vec::new();
    for (key, name) in BOSS_KILL_FLAGS {
        if pd.get(*key).and_then(|v| v.as_bool()).unwrap_or(false) {
            bosses_defeated.push(name.to_string());
        }
    }

    let heart_pieces = pd.get("heartPieces").and_then(|v| v.as_i64()).unwrap_or(0);
    let base_health = pd.get("maxHealth").and_then(|v| v.as_i64()).unwrap_or(5);
    let max_health = (base_health + heart_pieces / 4) as u32;

    let mp_reserve = pd.get("MPReserveMax").and_then(|v| v.as_i64()).unwrap_or(0);
    let soul_vessels = (mp_reserve / 33) as u32;

    let charm_ids = pd.get("charmsOwned").and_then(|v| v.as_i64()).unwrap_or(0) as u32;

    Some(SaveData {
        geo: pd.get("geo").and_then(|v| v.as_i64()).unwrap_or(0),
        completion_percentage: pd.get("completionPercentage").and_then(|v| v.as_f64()).unwrap_or(0.0),
        has_dash: pd.get("hasDash").and_then(|v| v.as_bool()).unwrap_or(false),
        has_super_jump: pd.get("hasSuperDash").and_then(|v| v.as_bool()).unwrap_or(false),
        has_double_jump: pd.get("hasDoubleJump").and_then(|v| v.as_bool()).unwrap_or(false),
        has_wall_jump: pd.get("hasWalljump").and_then(|v| v.as_bool()).unwrap_or(false),
        has_acid_armour: pd.get("hasAcidArmour").and_then(|v| v.as_bool()).unwrap_or(false),
        has_dream_nail: pd.get("hasDreamNail").and_then(|v| v.as_bool()).unwrap_or(false),
        play_time: pd.get("playTime").and_then(|v| v.as_f64()).unwrap_or(0.0),
        max_health,
        soul_vessels,
        dream_essence: pd.get("dreamOrbs").and_then(|v| v.as_f64()).unwrap_or(0.0) as u32,
        grub_count: pd.get("grubsCollected").and_then(|v| v.as_i64()).unwrap_or(0) as u32,
        nail_damage: pd.get("nailDamage").and_then(|v| v.as_i64()).unwrap_or(0) as u32,
        bosses_defeated,
        charms_count: charm_ids as u32,
    })
}

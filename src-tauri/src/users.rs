use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

const USERS_VERSION: u32 = 1;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
  pub id: String,
  pub name: String,
  pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserStore {
  pub version: u32,
  pub active_user: String,
  pub users: Vec<UserProfile>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSettings {
  pub version: u32,
  pub active_profile: String,
  pub auto_match: bool,
  pub match_offset: i32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
  pub id: String,
  pub name: String,
  pub created_at: String,
  pub data_dir: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsersSnapshot {
  pub active_user: String,
  pub users: Vec<UserInfo>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LlmKeyStore {
  pub keys: HashMap<String, String>,
}

impl UserStore {
  pub fn load_or_default(path: &Path) -> Self {
    if let Ok(data) = fs::read_to_string(path) {
      if let Ok(store) = serde_json::from_str::<UserStore>(&data) {
        return store;
      }
    }
    Self {
      version: USERS_VERSION,
      active_user: String::new(),
      users: Vec::new(),
    }
  }

  pub fn save(&self, path: &Path) -> Result<(), String> {
    let data = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
    fs::write(path, data).map_err(|e| e.to_string())
  }
}

impl UserSettings {
  pub fn save(&self, path: &Path) -> Result<(), String> {
    let data = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
    fs::write(path, data).map_err(|e| e.to_string())
  }
}

pub fn data_root() -> PathBuf {
  let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  manifest_dir
    .parent()
    .unwrap_or(&manifest_dir)
    .join("data")
}

pub fn users_path() -> PathBuf {
  data_root().join("users.json")
}

pub fn user_dir(id: &str) -> PathBuf {
  data_root().join("users").join(id)
}

pub fn ratings_user_path(id: &str) -> PathBuf {
  user_dir(id).join("ratings_user.json")
}

pub fn user_settings_path(id: &str) -> PathBuf {
  user_dir(id).join("settings.json")
}

pub fn llm_keys_path(id: &str) -> PathBuf {
  user_dir(id).join("llm_keys.json")
}

pub fn ensure_user_dir(id: &str) -> Result<(), String> {
  let dir = user_dir(id);
  fs::create_dir_all(dir).map_err(|e| e.to_string())
}

pub fn ensure_data_dirs() -> Result<(), String> {
  fs::create_dir_all(data_root()).map_err(|e| e.to_string())?;
  fs::create_dir_all(data_root().join("users")).map_err(|e| e.to_string())
}

pub fn new_user_id() -> String {
  let now = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap_or_default();
  format!("user-{}-{}", now.as_secs(), now.subsec_nanos())
}

pub fn now_timestamp() -> String {
  let now = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap_or_default();
  format!("{}", now.as_secs())
}

pub fn snapshot_from_store(store: &UserStore) -> UsersSnapshot {
  let users = store
    .users
    .iter()
    .map(|user| UserInfo {
      id: user.id.clone(),
      name: user.name.clone(),
      created_at: user.created_at.clone(),
      data_dir: user_dir(&user.id).to_string_lossy().to_string(),
    })
    .collect();

  UsersSnapshot {
    active_user: store.active_user.clone(),
    users,
  }
}

impl LlmKeyStore {
  pub fn load_or_default(path: &Path) -> Self {
    if let Ok(data) = fs::read_to_string(path) {
      if let Ok(store) = serde_json::from_str::<LlmKeyStore>(&data) {
        return store;
      }
    }
    LlmKeyStore::default()
  }

  pub fn save(&self, path: &Path) -> Result<(), String> {
    let data = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
    fs::write(path, data).map_err(|e| e.to_string())
  }
}

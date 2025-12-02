use std::{collections::HashMap, path::PathBuf};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use tokio::fs::{create_dir_all, read_to_string};

#[derive(Serialize, Deserialize, Clone, Debug)]
enum Constraint {
    MaxValue(i32),
    MinValue(i32),
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) enum SettingValue {
    Int(i32),
    Bool(bool),
    String(Option<String>),
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Setting {
    constraints: Option<Vec<Constraint>>,
    value: SettingValue,
}
impl Setting {
    pub fn get_value(&self) -> &SettingValue {
        &self.value
    }
}
pub struct Settings;
impl Settings {
    pub async fn load() -> HashMap<String, Setting> {
        let settings: HashMap<String, Setting> = REQUIRED_SETTINGS
            .iter()
            .map(|(name, setting)| ((*name).to_string(), setting.clone()))
            .collect();
        // If path is created, there is no configuration so we can return
        if create_config_path().await.is_ok() {
            return settings;
        }
        let settings_path = get_config_save_file_path(SaveFile::Settings);
        let settings_json = read_to_string(&settings_path).await;
        let json = match settings_json {
            Ok(settings) => settings,
            Err(err) => {
                tokio::fs::File::create(settings_path.clone())
                    .await
                    .unwrap();
                "".to_string()
            }
        };
        let mut user_settings = match serde_json::from_str::<HashMap<String, Setting>>(&json) {
            Ok(s) => s,
            Err(err) => {
                println!("{:?}", err);
                settings.clone()
            }
        };

        // TODO: Set default values to missing options, enforce constraints
        for (opt_key, opt_val) in settings {
            if let Some(setting) = user_settings.get_mut(&opt_key) {
                // TODO: enforce the constraints here
            } else {
                // insert the default if opt is missing
                user_settings.insert(opt_key, opt_val);
            }
        }
        user_settings
    }
    pub async fn save(settings: &HashMap<String, Setting>) {
        let settings_path = get_config_save_file_path(SaveFile::Settings);
        println!("{:?}", settings_path);
        let serialized = serde_json::to_string::<HashMap<String, Setting>>(settings)
            .expect("failed to serialize settings");
        std::fs::write(settings_path, serialized).expect("failed to write settings");
    }
}
async fn create_config_path() -> std::io::Result<()> {
    let proj_dir =
        ProjectDirs::from("com", "Mistr", "p2pchat").expect("Couldnt determine directories");
    create_dir_all(proj_dir.config_dir()).await?;
    Ok(())
}

pub(crate) fn get_config_save_file_path(savefile: SaveFile) -> PathBuf {
    let proj_dirs =
        ProjectDirs::from("com", "Mistr", "p2pchat").expect("Couldnt determine directories");
    proj_dirs.config_dir().join(
        SAVE_FILES
            .iter()
            .find(|x| x.0 == savefile)
            .expect("Save file path not defined")
            .1,
    )
}
static REQUIRED_SETTINGS: &[(&str, Setting)] = &[(
    "name",
    Setting {
        constraints: None,
        value: SettingValue::String(None),
    },
)];
#[derive(PartialEq)]
pub(crate) enum SaveFile {
    Settings,
}
static SAVE_FILES: &[(SaveFile, &str)] = &[(SaveFile::Settings, "settings")];

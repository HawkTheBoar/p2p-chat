use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
enum Constraint {
    MaxValue(i32),
    MinValue(i32),
}
#[derive(Serialize, Deserialize, Clone, Debug)]
enum SettingValue {
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
    fn get_value(&self) -> &SettingValue {
        &self.value
    }
}
pub fn load_settings(json: &str) -> HashMap<String, Setting> {
    let settings: HashMap<String, Setting> = REQUIRED_SETTINGS
        .iter()
        .map(|(name, setting)| ((*name).to_string(), setting.clone()))
        .collect();
    let mut user_settings = match serde_json::from_str::<HashMap<String, Setting>>(json) {
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
pub fn save_settings(settings: &HashMap<String, Setting>, save_dir: &str) {
    let serialized = serde_json::to_string::<HashMap<String, Setting>>(&settings)
        .expect("failed to serialize settings");
    std::fs::write(save_dir, serialized).expect("failed to write settings");
}
static REQUIRED_SETTINGS: &[(&str, Setting)] = &[(
    "name",
    Setting {
        constraints: None,
        value: SettingValue::String(None),
        // value: SettingValue::Int(0),
    },
)];

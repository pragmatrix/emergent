pub use serde::{Deserialize, Serialize};
use std::fs;
pub use std::path::PathBuf;

pub trait Configuration: Serialize + serde::de::DeserializeOwned {
    fn config_path() -> PathBuf;

    fn save(&self) {
        let json = serde_json::to_string(self).expect("failed to serialize");
        let path = get_config_file_path::<Self>();
        fs::create_dir_all(path.clone().parent().expect("no parent directory")).ok();
        fs::write(&path, json).expect(&format!("failed to write to {:?}", &path));
    }

    fn load() -> Option<Self> {
        let path = get_config_file_path::<Self>();
        let str = fs::read_to_string(&path).ok()?;
        serde_json::from_str(&str)
            .map_err(|e| error!("deserialization failed: {}", e))
            .ok()
    }

    fn delete() {
        let path = get_config_file_path::<Self>();
        fs::remove_file(path).ok();
    }
}

fn get_config_file_path<C>() -> PathBuf
where
    C: Configuration,
{
    let dir = dirs::config_dir().expect("failed to get configuration directory");

    let config_path = C::config_path().with_extension("json");
    dir.join(config_path)
}

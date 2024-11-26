use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// All operations related to the medusa.json file

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct JsonConfig {
    compilation: Compilation,
    fuzzing: Fuzzing,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Compilation {
    platform_config: PlatformConfig,
}

#[derive(Serialize, Deserialize, Debug)]
struct PlatformConfig {
    target: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Fuzzing {
    shrink_limit: i32,
}

/// Read and return the compilation:platformConfig:target from medusa.json
/// Throws if not defined in the json or if the contract doesn't exist
pub fn get_entry_point_path(filename: &str) -> Result<PathBuf, anyhow::Error> {
    let medusa_json = Path::new(filename); // Make the filename a clap arg?

    if !medusa_json.exists() {
        return Err(anyhow::anyhow!("No medusa config file found"));
    }

    let config: JsonConfig =
        serde_json::from_str(&std::fs::read_to_string(medusa_json).context("failed to read json")?)
            .context("failed to parse json")?;

    let mut contract_path = PathBuf::new();
    contract_path.push(&config.compilation.platform_config.target);

    if !contract_path.exists() {
        return Err(anyhow::anyhow!("Contract file not found"));
    }

    Ok(contract_path)
}

/// Read and return the fuzzig:shrinkLimit from medusa.json
/// Throws if not defined in the json
pub fn get_shrink_limit(filename: &str) -> Result<i32, anyhow::Error> {
    let medusa_json = Path::new(filename); // Make the filename a clap arg?

    if !medusa_json.exists() {
        return Err(anyhow::anyhow!("No file"));
    }

    let config: JsonConfig =
        serde_json::from_str(&std::fs::read_to_string(medusa_json).context("failed to read json")?)
            .context("failed to parse json")?;

    Ok(config.fuzzing.shrink_limit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_entry_point_path() {
        let path = get_entry_point_path("medusa.json");
        assert!(path.is_ok());
        assert_eq!(path.unwrap(), PathBuf::from("test/FuzzTest.t.sol"));
    }

    #[test]
    fn test_get_shrinking() {
        let shrink = get_shrink_limit("medusa.json");
        assert_eq!(shrink.unwrap(), 500);
    }
}

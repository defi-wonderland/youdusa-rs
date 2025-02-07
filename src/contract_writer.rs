use anyhow::{anyhow, Context, Result};
use askama::Template;
use std::fs::{self, File};
use std::io::Write as WriteIO;
use std::path::Path;
use serde_json::Value;

/// The contract template,
#[derive(Template, Debug, Clone, PartialEq)]
#[template(path = "template.sol", escape = "none")]
pub struct Contract {
    reproducers: String,
    contract_name: String,
    path: String,
}

impl Contract {
    pub fn new(reproducers: &[u8]) -> Result<Contract> {
        let path = get_target_path().context("Failed to get target path")?;

        let contract_name =
            Contract::find_first_unused_filename(path.clone()).context("Failed to find a filename")?;

        Ok(Contract {
            reproducers: String::from_utf8_lossy(reproducers).to_string(),
            contract_name,
            path
        })
    }

    pub fn write_rendered_contract(&self) -> Result<()> {        
        // Ensure the target directory exists (create it if not)
        fs::create_dir_all(&self.path)
            .context("Failed to create target directory")?;
        
        // Construct the full filepath by combining the target directory and contract file name.
        let file_name = format!("{}.t.sol", self.contract_name);
        let output_filepath = std::path::Path::new(&self.path).join(file_name);

        let mut f = File::create_new(&output_filepath)
            .context("Failed to create contract file")?;

        let rendered = self.render().context("Fail to render contract")?;

        f.write_all(rendered.as_bytes())
            .context("Failed to write contract")?;

        Ok(())
    }

    fn find_first_unused_filename(target_path: String) -> Result<String> {
        // Avoiding Regex intensifies
        (0..)
            .map(|i| {
                if i == 0 {
                    "ForgeReproducer".to_owned()
                } else {
                    format!("ForgeReproducer{}", i)
                }
            })
            .find(|base| !Path::new(&format!("{}{}{}", target_path, base, ".t.sol")).exists())
            .ok_or_else(|| anyhow!("No available filename found"))
    }
}

/// Helper function to get the target directory from the medusa.json config.
/// The "target" is expected under "compilation"->"platformConfig" in medusa.json.
/// If the target contains a file (determined by checking for a file extension),
/// the parent directory is returned. Otherwise, the target is assumed to be a directory.
/// If medusa.json does not exist or no target is provided, the default
/// "test/invariants/fuzz/" is returned.
fn get_target_path() -> Result<String> {
    let default_target = "test/invariants/fuzz/".to_owned();
    let medusa_file = "medusa.json";

    if !Path::new(medusa_file).exists() {
        return Ok(default_target);
    }

    let medusa_contents = fs::read_to_string(medusa_file)
        .context("Failed to read medusa.json")?;
    let parsed: Value = serde_json::from_str(&medusa_contents)
        .context("Failed to parse medusa.json")?;

    let target_str = parsed
        .get("compilation")
        .and_then(|comp| comp.get("platformConfig"))
        .and_then(|config| config.get("target"))
        .and_then(|val| val.as_str())
        .unwrap_or("test/invariants/fuzz/");

    // If the target json value is "" or ".", then default to "test/invariants/fuzz/"
    let target_str = if target_str.trim().is_empty() || target_str.trim() == "." {
        "test/invariants/fuzz/"
    } else {
        target_str
    };

    let target_path = Path::new(target_str);
    let final_target = if target_path.extension().is_some() {
        // Target seems to be a file, so use its parent directory.
        if let Some(parent) = target_path.parent() {
            let parent_str = parent.to_str().unwrap_or("");
            if parent_str.is_empty() {
                "./".to_owned()
            } else if parent_str.ends_with('/') {
                parent_str.to_owned()
            } else {
                format!("{}/", parent_str)
            }
        } else {
            default_target
        }
    } else {
        // Target is already a directory, ensure it ends with '/'
        let target_owned = target_str.to_owned();
        if target_owned.ends_with('/') {
            target_owned
        } else {
            format!("{}/", target_owned)
        }
    };

    Ok(final_target)
}

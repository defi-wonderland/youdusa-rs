use anyhow::{Context, Result, anyhow};
use askama::Template;
use std::fs::File;
use std::io::Write as WriteIO;
use std::path::Path;

/// The contract template,
#[derive(Template, Debug, Clone, PartialEq)]
#[template(path = "template.sol", escape = "none")]
pub struct Contract {
    reproducers: String,
    contract_name: String,
}

impl Contract {
    pub fn new(reproducers: &[u8]) -> Result<Contract> {
        let contract_name = Contract::find_first_unused_filename().context("Failed to find a filename")?;

        Ok(Contract { reproducers: String::from_utf8_lossy(reproducers).to_string(), contract_name })
    }

    pub fn write_rendered_contract(&self) -> Result<()> {
        let mut f = File::create_new(format!("{}.t.sol", self.contract_name))
            .context(format!("Failed to create contract"))?;

        let rendered = self
            .render()
            .context(format!("Fail to render contract"))?;

        f.write_all(rendered.as_bytes())
            .context(format!("Failed to write contract"))?;

        Ok(())
    }

    fn find_first_unused_filename() -> Result<String> {
        // Avoiding Regex intensifies
        (0..)
            .map(|i| if i == 0 { "ForgeReproducer".to_owned() } else { format!("ForgeReproducer{}", i) })
            .find(|base| !Path::new(&format!("{}{}", base, ".t.sol")).exists())
            .ok_or_else(|| anyhow!("No available filename found"))
    }
}
use anyhow::{Result, anyhow};

#[derive(Clone, Debug)]
pub struct FormatRules {
    pub name: String,
    pub min_mainboard: u32,
    pub max_sideboard: u32,
    pub default_copy_limit: u32,
    pub singleton: bool,
    pub uses_restricted_list: bool,
}

impl FormatRules {
    fn new(
        name: &str,
        min_mainboard: u32,
        max_sideboard: u32,
        default_copy_limit: u32,
        singleton: bool,
        uses_restricted_list: bool,
    ) -> Self {
        Self {
            name: name.to_string(),
            min_mainboard,
            max_sideboard,
            default_copy_limit,
            singleton,
            uses_restricted_list,
        }
    }
}

pub fn get_format_rules(format_name: &str) -> Result<FormatRules> {
    match format_name.to_lowercase().trim() {
        "standard" => Ok(FormatRules::new("standard", 60, 15, 4, false, false)),
        "alchemy" => Ok(FormatRules::new("alchemy", 60, 15, 4, false, false)),
        "explorer" => Ok(FormatRules::new("explorer", 60, 15, 4, false, false)),
        "historic" => Ok(FormatRules::new("historic", 60, 15, 4, false, false)),
        "timeless" => Ok(FormatRules::new("timeless", 60, 15, 4, false, true)),
        "brawl" => Ok(FormatRules::new("brawl", 100, 0, 1, true, false)),
        "historicbrawl" => Ok(FormatRules::new("historicbrawl", 100, 0, 1, true, false)),
        "historic brawl" => Ok(FormatRules::new("historic brawl", 100, 0, 1, true, false)),
        _ => Err(anyhow!("Unsupported format: {format_name}")),
    }
}

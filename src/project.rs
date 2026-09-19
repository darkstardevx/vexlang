use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectConfig {
    pub name: String,
    pub version: String,
    pub source: String,
}

pub fn load(path: &Path) -> Result<ProjectConfig, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("could not read `{}`: {error}", path.display()))?;
    let mut name = None;
    let mut version = None;
    let mut source = None;
    for line in text.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') || line == "[project]" {
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            return Err(format!("invalid project configuration line `{line}`"));
        };
        let value = value.trim().trim_matches('"').to_string();
        match key.trim() {
            "name" => name = Some(value),
            "version" => version = Some(value),
            "source" => source = Some(value),
            other => return Err(format!("unknown project configuration key `{other}`")),
        }
    }
    Ok(ProjectConfig {
        name: name.ok_or("project configuration is missing `name`")?,
        version: version.ok_or("project configuration is missing `version`")?,
        source: source.unwrap_or_else(|| "src/main.vex".into()),
    })
}

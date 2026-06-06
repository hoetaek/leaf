use anyhow::{Result, bail};

pub(crate) fn validate(value: &str) -> Result<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed != value {
        bail!("invalid slug: {value}");
    }
    if !trimmed
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        bail!("invalid slug: {value}");
    }
    Ok(trimmed.to_string())
}

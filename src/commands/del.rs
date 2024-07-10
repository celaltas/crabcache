use anyhow::{Context, Ok, Result};

pub fn invoke(key: Vec<u8>) -> Result<()> {
    let key = String::from_utf8(key).context("Failed to convert key to string")?;
    println!("del command called key: {key}",);
    Ok(())
}

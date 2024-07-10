use anyhow::{Context, Ok, Result};

pub fn invoke(key: Vec<u8>, value: Vec<u8>) -> Result<()> {
    let key = String::from_utf8(key).context("Failed to convert key to string")?;
    let value = String::from_utf8(value).context("Failed to convert value to string")?;
    println!("set command called key:{key}, value:{value}");
    Ok(())
}

use std::fs::File;
use std::io::Write;
use std::path::Path;

use eyre::{Context, OptionExt, Result};
use serde_json::Value;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

pub fn download_sb3(id: &str, outfile: &Path) -> Result<()> {
    let client = reqwest::blocking::Client::new();
    let metadata = client
        .get(format!("https://api.scratch.mit.edu/projects/{id}"))
        .send()
        .wrap_err("failed to fetch project metadata")?
        .error_for_status()
        .wrap_err("failed to fetch project metadata")?
        .json::<Value>()
        .wrap_err("failed to parse project metadata")?;
    let token = metadata
        .get("project_token")
        .and_then(Value::as_str)
        .ok_or_eyre("missing project token")?;
    let data = client
        .get(format!(
            "https://projects.scratch.mit.edu/{id}?token={token}"
        ))
        .send()
        .wrap_err("failed to fetch project data")?
        .error_for_status()
        .wrap_err("failed to fetch project data")?
        .json::<Value>()
        .wrap_err("failed to parse project data")?;
    let file = File::create(outfile).wrap_err("failed to create sb3")?;
    let mut zip = ZipWriter::new(file);
    zip.start_file("project.json", SimpleFileOptions::default())
        .wrap_err("failed to write project.json")?;
    zip.write_all(
        serde_json::to_string(&data)
            .expect("project json serializes")
            .as_bytes(),
    )
    .wrap_err("failed to write project.json")?;
    for md5ext in collect_assets(&data)? {
        let bytes = client
            .get(format!(
                "https://assets.scratch.mit.edu/internalapi/asset/{md5ext}/get/"
            ))
            .send()
            .wrap_err_with(|| format!("failed to fetch asset {md5ext}"))?
            .error_for_status()
            .wrap_err_with(|| format!("failed to fetch asset {md5ext}"))?
            .bytes()
            .wrap_err_with(|| format!("failed to read asset {md5ext}"))?;
        zip.start_file(md5ext, SimpleFileOptions::default())
            .wrap_err("failed to write asset")?;
        zip.write_all(&bytes).wrap_err("failed to write asset")?;
    }
    zip.finish().wrap_err("failed to finish sb3")?;
    Ok(())
}

fn collect_assets(data: &Value) -> Result<Vec<String>> {
    let mut assets = Vec::new();
    for target in array_key(data, "targets")? {
        for costume in array_key(target, "costumes")? {
            push_unique(&mut assets, string_key(costume, "md5ext")?);
        }
        for sound in array_key(target, "sounds")? {
            push_unique(&mut assets, string_key(sound, "md5ext")?);
        }
    }
    Ok(assets)
}

fn array_key<'a>(value: &'a Value, key: &str) -> Result<&'a [Value]> {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| eyre::eyre!("missing array key {key}"))
}

fn string_key<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| eyre::eyre!("missing string key {key}"))
}

fn push_unique(values: &mut Vec<String>, value: &str) {
    if !values.iter().any(|existing| existing == value) {
        values.push(value.to_string());
    }
}

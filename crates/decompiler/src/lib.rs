mod emit;
mod errors;
mod file_system;
pub mod json_ext;
mod signatures;
pub mod syntax;

use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{Cursor, Read, Seek, Write};
use std::path::Path;

use image::{DynamicImage, ImageFormat, RgbaImage, imageops};
use indexmap::IndexMap;
use serde_json::Value;
use xmltree::{Element, EmitterConfig, XMLNode};
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

pub use emit::{Ctx, decompile_sprite};
pub use errors::DecompileError;

use crate::errors::{
    image_error, invalid_project, io, json_error, utf8_error, xml_error, xml_parse_error, zip_error,
};
pub use crate::file_system::{DiskFS, FS, InMemoryFS};
use crate::json_ext::{array, bool_key, f64_key, i64_key, key, object, opt_key, string_key};
use crate::syntax::{Identifiers, toml_string};

/// Generates goboscript for one Scratch target.
pub fn generate_sprite(target: Value, assets: &IndexMap<String, String>) -> String {
    let mut syntax = Identifiers::new();
    let mut ctx = Ctx::new(target, assets, &mut syntax);
    decompile_sprite(&mut ctx);
    ctx.finish()
}

/// Scratch project metadata prepared for code and asset decompilation.
#[derive(Debug)]
pub struct DecompiledAssets {
    project: Value,
    names: IndexMap<String, String>,
}

/// Decompiles an `.sb3` zip archive into an output directory.
pub fn decompile<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    output: &Path,
) -> Result<(), DecompileError> {
    match fs::remove_dir_all(output) {
        Ok(()) => {}
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => {}
        Err(source) => return Err(io("failed to remove output directory", source)),
    }
    fs::create_dir_all(output).map_err(|source| io("failed to create output directory", source))?;
    let assets_path = output.join("assets");
    let assets = decompile_assets(archive)?;
    let mut fs = DiskFS::new(output);
    generate_code_files(&assets.project, &assets.names, &mut fs)?;
    extract_asset_files(archive, &assets, &assets_path)?;
    Ok(())
}

/// Decompiles `.sb3` bytes into a zip archive containing generated goboscript project files.
pub fn decompile_sb3_to_zip(input: &[u8]) -> Result<Vec<u8>, DecompileError> {
    let mut archive = ZipArchive::new(Cursor::new(input))
        .map_err(|source| zip_error("failed to read sb3 zip", source))?;
    let assets = decompile_assets(&mut archive)?;
    let mut files = decompile_code(&assets)?;
    let mut asset_files = read_asset_files(&mut archive, &assets.names)?;
    fix_asset_bytes(&assets.project, &assets.names, &mut asset_files)?;
    for (name, bytes) in asset_files {
        files.push((format!("assets/{name}"), bytes));
    }
    write_project_zip(files)
}

/// Decompiles `.sb3` bytes into asset files keyed by goboscript project path.
pub fn decompile_asset_files(input: &[u8]) -> Result<IndexMap<String, Vec<u8>>, DecompileError> {
    let mut archive = ZipArchive::new(Cursor::new(input))
        .map_err(|source| zip_error("failed to read sb3 zip", source))?;
    let assets = decompile_assets(&mut archive)?;
    let mut asset_files = read_asset_files(&mut archive, &assets.names)?;
    fix_asset_bytes(&assets.project, &assets.names, &mut asset_files)?;
    Ok(root_asset_paths(asset_files))
}

/// Reads Scratch project metadata from an `.sb3` zip archive.
pub fn decompile_assets<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
) -> Result<DecompiledAssets, DecompileError> {
    let project = read_project_json_result(archive)?;
    let mut names = get_asset_names(&project, "costumes");
    names.extend(get_asset_names(&project, "sounds"));
    Ok(DecompiledAssets { project, names })
}

/// Extracts Scratch project asset files into an output directory.
pub fn extract_asset_files<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    assets: &DecompiledAssets,
    output: &Path,
) -> Result<(), DecompileError> {
    fs::create_dir_all(output).map_err(|source| io("failed to create assets directory", source))?;
    let mut files = read_asset_files(archive, &assets.names)?;
    fix_asset_bytes(&assets.project, &assets.names, &mut files)?;
    write_asset_files(output, files)
}

/// Decompiles goboscript code using decompiled assets.
pub fn decompile_code(assets: &DecompiledAssets) -> Result<Vec<(String, Vec<u8>)>, DecompileError> {
    let mut fs = InMemoryFS::new();
    generate_code_files(&assets.project, &assets.names, &mut fs)?;
    Ok(fs
        .into_files()
        .into_iter()
        .map(|(path, text)| (path, text.into_bytes()))
        .collect())
}

fn read_project_json_result<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
) -> Result<Value, DecompileError> {
    let mut project_file = archive
        .by_name("project.json")
        .map_err(|source| zip_error("failed to open project.json", source))?;
    let mut text = String::new();
    project_file
        .read_to_string(&mut text)
        .map_err(|source| io("failed to read project.json", source))?;
    serde_json::from_str(&text).map_err(|source| json_error("failed to parse project.json", source))
}

fn get_asset_names(project: &Value, key_name: &str) -> IndexMap<String, String> {
    let mut assets = IndexMap::new();
    let mut filenames = HashMap::new();
    for target in array(key(project, "targets")) {
        for asset in array(key(target, key_name)) {
            let md5ext = string_key(asset, "md5ext");
            if assets.contains_key(md5ext) {
                continue;
            }
            let mut index = 2;
            let mut filename = get_asset_filename(string_key(asset, "name"), md5ext);
            while filenames.contains_key(&filename) {
                filename =
                    get_asset_filename(&format!("{} ({index})", string_key(asset, "name")), md5ext);
                index += 1;
            }
            assets.insert(md5ext.to_string(), filename.clone());
            filenames.insert(filename, md5ext.to_string());
        }
    }
    assets
}

fn get_asset_filename(name: &str, md5ext: &str) -> String {
    let mut name = if name == "/" {
        "forward-slash".to_string()
    } else {
        name.replace('/', "")
    };
    if cfg!(windows) || name.is_empty() {
        name = md5ext.chars().take(8).collect();
    }
    format!("{name}{}", extension_with_dot(md5ext))
}

fn extension_with_dot(path: &str) -> String {
    Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
        .map_or_else(String::new, |extension| format!(".{extension}"))
}

fn read_asset_files<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    assets: &IndexMap<String, String>,
) -> Result<IndexMap<String, Vec<u8>>, DecompileError> {
    let mut output = IndexMap::new();
    for (md5ext, name) in assets {
        let mut source = archive
            .by_name(md5ext)
            .map_err(|source| zip_error(format!("failed to open asset {md5ext}"), source))?;
        let mut bytes = Vec::new();
        source
            .read_to_end(&mut bytes)
            .map_err(|source| io(format!("failed to copy asset {name}"), source))?;
        output.insert(name.clone(), bytes);
    }
    Ok(output)
}

fn generate_code_files(
    project: &Value,
    assets: &IndexMap<String, String>,
    fs: &mut impl FS,
) -> Result<(), DecompileError> {
    let stage = array(key(project, "targets"))
        .iter()
        .find(|target| bool_key(target, "isStage"))
        .ok_or_else(|| invalid_project("stage not found"))?
        .clone();
    let sprites = array(key(project, "targets"))
        .iter()
        .filter(|target| !bool_key(target, "isStage"))
        .cloned()
        .collect::<Vec<_>>();
    let mut syntax = Identifiers::new();
    let mut stage_ctx = Ctx::new(stage, assets, &mut syntax);
    decompile_sprite(&mut stage_ctx);
    fs.write_text_file("stage.gs", &stage_ctx.finish())
        .map_err(|source| io("failed to write generated code", source))?;
    for sprite in sprites {
        let mut ctx = Ctx::new(sprite.clone(), assets, &mut syntax);
        decompile_sprite(&mut ctx);
        fs.write_text_file(
            &format!("{}.gs", string_key(&sprite, "name")),
            &ctx.finish(),
        )
        .map_err(|source| io("failed to write generated code", source))?;
    }
    fs.write_text_file("goboscript.toml", &config_text(project))
        .map_err(|source| io("failed to write generated code", source))?;
    Ok(())
}

fn fix_asset_bytes(
    project: &Value,
    assets: &IndexMap<String, String>,
    output_assets: &mut IndexMap<String, Vec<u8>>,
) -> Result<(), DecompileError> {
    let mut fixed = HashSet::new();
    for sprite in array(key(project, "targets")) {
        if bool_key(sprite, "isStage") {
            continue;
        }
        for costume in array(key(sprite, "costumes")) {
            fix_center_bytes(costume, assets, output_assets, &mut fixed)?;
        }
    }
    Ok(())
}

fn root_asset_paths(files: IndexMap<String, Vec<u8>>) -> IndexMap<String, Vec<u8>> {
    files
        .into_iter()
        .map(|(name, bytes)| (format!("assets/{name}"), bytes))
        .collect()
}

fn write_project_zip(files: Vec<(String, Vec<u8>)>) -> Result<Vec<u8>, DecompileError> {
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    for (name, bytes) in files {
        zip.start_file(name, SimpleFileOptions::default())
            .map_err(|source| zip_error("failed to write generated zip entry", source))?;
        zip.write_all(&bytes)
            .map_err(|source| io("failed to write generated zip bytes", source))?;
    }
    zip.finish()
        .map(Cursor::into_inner)
        .map_err(|source| zip_error("failed to finish generated zip", source))
}

fn fix_center_bytes(
    costume: &Value,
    assets: &IndexMap<String, String>,
    output_assets: &mut IndexMap<String, Vec<u8>>,
    fixed: &mut HashSet<String>,
) -> Result<(), DecompileError> {
    let md5ext = string_key(costume, "md5ext");
    if fixed.contains(md5ext) {
        return Ok(());
    }
    fixed.insert(md5ext.to_string());
    let name = assets
        .get(md5ext)
        .ok_or_else(|| invalid_project(format!("missing asset name {md5ext}")))?;
    let bytes = output_assets
        .get_mut(name)
        .ok_or_else(|| invalid_project(format!("missing asset {name}")))?;
    let input = std::mem::take(bytes);
    *bytes = if string_key(costume, "dataFormat") == "svg" {
        fix_vector_center_bytes(costume, input)?
    } else {
        fix_bitmap_center_bytes(costume, input)?
    };
    Ok(())
}

fn fix_vector_center_bytes(costume: &Value, bytes: Vec<u8>) -> Result<Vec<u8>, DecompileError> {
    let text = String::from_utf8(bytes.clone())
        .map_err(|source| utf8_error("failed to decode svg text", source))?;
    let mut root = Element::parse(text.as_bytes())
        .map_err(|source| xml_parse_error("failed to parse svg", source))?;
    let width = root
        .attributes
        .get("width")
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(0.0);
    let height = root
        .attributes
        .get("height")
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(0.0);
    if width / 2.0 == f64_key(costume, "rotationCenterX")
        && height / 2.0 == f64_key(costume, "rotationCenterY")
    {
        return Ok(bytes);
    }
    root.attributes
        .insert("width".to_string(), "480".to_string());
    root.attributes
        .insert("height".to_string(), "360".to_string());
    root.attributes
        .insert("viewBox".to_string(), "0,0,480,360".to_string());
    if let Some(group) = root.children.iter_mut().find_map(svg_group_mut) {
        group.attributes.remove("transform");
    }
    let mut out = Vec::new();
    root.write_with_config(
        &mut out,
        EmitterConfig::new().write_document_declaration(false),
    )
    .map_err(|source| xml_error("failed to write svg", source))?;
    Ok(out)
}

fn svg_group_mut(node: &mut XMLNode) -> Option<&mut Element> {
    let XMLNode::Element(element) = node else {
        return None;
    };
    if element.name == "g" {
        return Some(element);
    }
    None
}

fn fix_bitmap_center_bytes(costume: &Value, bytes: Vec<u8>) -> Result<Vec<u8>, DecompileError> {
    let image = image::load_from_memory(&bytes)
        .map_err(|source| image_error("failed to open bitmap", source))?;
    if f64_key(costume, "rotationCenterX") == f64::from(image.width() / 2)
        && f64_key(costume, "rotationCenterY") == f64::from(image.height() / 2)
    {
        return Ok(bytes);
    }
    let mut fixed = RgbaImage::new(960, 720);
    let x = (480.0 - f64_key(costume, "rotationCenterX")).trunc() as i64;
    let y = (360.0 - f64_key(costume, "rotationCenterY")).trunc() as i64;
    imageops::overlay(&mut fixed, &image.to_rgba8(), x, y);
    let format = ImageFormat::from_extension(string_key(costume, "dataFormat"))
        .ok_or_else(|| invalid_project("unknown bitmap format"))?;
    let mut out = Cursor::new(Vec::new());
    DynamicImage::ImageRgba8(fixed)
        .write_to(&mut out, format)
        .map_err(|source| image_error("failed to save bitmap", source))?;
    Ok(out.into_inner())
}

fn write_asset_files(
    assets_path: &Path,
    files: IndexMap<String, Vec<u8>>,
) -> Result<(), DecompileError> {
    for (name, bytes) in files {
        fs::write(assets_path.join(&name), bytes)
            .map_err(|source| io(format!("failed to write asset {name}"), source))?;
    }
    Ok(())
}

fn config_text(project: &Value) -> String {
    let mut out = String::new();
    let data = config_data(project);
    write_layers(project, &mut out);
    out.push_str("bitmap_resolution = 2\n");
    if let Some(frame_rate) = data
        .as_ref()
        .and_then(|data| opt_key(data, "framerate"))
        .and_then(Value::as_i64)
    {
        out.push_str(&format!("frame_rate = {frame_rate}\n"));
    }
    if let Some(max_clones) = data
        .as_ref()
        .and_then(|data| opt_key(data, "runtimeOptions"))
        .and_then(|options| opt_key(options, "maxClones"))
        .and_then(Value::as_f64)
    {
        out.push_str(&format!("max_clones = {max_clones}\n"));
    }
    out.push_str(&format!(
        "no_miscellaneous_limits = {}\n",
        data.as_ref()
            .and_then(|data| opt_key(data, "runtimeOptions"))
            .and_then(|options| opt_key(options, "miscLimits"))
            .is_some_and(|value| value == &Value::Bool(false))
    ));
    out.push_str(&format!(
        "no_sprite_fencing = {}\n",
        data.as_ref()
            .and_then(|data| opt_key(data, "runtimeOptions"))
            .and_then(|options| opt_key(options, "fencing"))
            .is_some_and(|value| value == &Value::Bool(false))
    ));
    out.push_str(&format!(
        "frame_interpolation = {}\n",
        data.as_ref()
            .and_then(|data| opt_key(data, "interpolation"))
            .is_some_and(|value| value == &Value::Bool(true))
    ));
    out.push_str(&format!(
        "high_quality_pen = {}\n",
        data.as_ref()
            .and_then(|data| opt_key(data, "hq"))
            .is_some_and(|value| value == &Value::Bool(true))
    ));
    if let Some(stage_width) = data
        .as_ref()
        .and_then(|data| opt_key(data, "width"))
        .and_then(Value::as_i64)
    {
        out.push_str(&format!("stage_width = {stage_width}\n"));
    }
    if let Some(stage_height) = data
        .as_ref()
        .and_then(|data| opt_key(data, "height"))
        .and_then(Value::as_i64)
    {
        out.push_str(&format!("stage_height = {stage_height}\n"));
    }
    out
}

fn write_layers(project: &Value, out: &mut String) {
    let mut layers = array(key(project, "targets")).iter().collect::<Vec<_>>();
    layers.sort_by_key(|target| i64_key(target, "layerOrder"));
    let layers = layers
        .into_iter()
        .filter(|target| string_key(target, "name") != "Stage")
        .map(|target| toml_string(string_key(target, "name")))
        .collect::<Vec<_>>();
    out.push_str("layers = [");
    out.push_str(&layers.join(", "));
    out.push_str("]\n");
}

fn config_data(project: &Value) -> Option<Value> {
    let text = find_turbowarp_config_comment(project)?;
    let start = text.find('{')?;
    let end = text.rfind('}')?;
    serde_json::from_str(&text[start..=end]).ok()
}

fn find_turbowarp_config_comment(project: &Value) -> Option<&str> {
    let stage = array(key(project, "targets"))
        .iter()
        .find(|target| bool_key(target, "isStage"))?;
    for comment in object(key(stage, "comments")).values() {
        let text = string_key(comment, "text");
        if text.ends_with("_twconfig_") {
            return Some(text);
        }
    }
    None
}

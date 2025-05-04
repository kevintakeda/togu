use ignore::WalkBuilder;
use regex::Regex;
use serde_json::json;
use std::collections::{BTreeMap, HashSet};
use std::fs::{self, File};
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;

fn get_translations(
    root: &Path,
) -> anyhow::Result<Vec<(File, BTreeMap<String, serde_json::Value>)>> {
    if !root.exists() {
        return Err(anyhow::anyhow!("Directory does not exist"));
    }
    if !root.is_dir() {
        return Err(anyhow::anyhow!("Path is not a directory"));
    }
    let mut entries = Vec::new();
    for entry in WalkBuilder::new(root)
        .standard_filters(true)
        .max_depth(Some(10))
        .build()
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_some_and(|t| t.is_dir()))
    {
        if entry.file_name() == "lang" {
            for entry in entry.path().read_dir()? {
                if let Ok(entry) = entry {
                    if entry.path().extension().map_or(false, |ext| ext == "json") {
                        let file = fs::File::options()
                            .read(true)
                            .write(true)
                            .open(entry.path())?;
                        let json_value: BTreeMap<String, serde_json::Value> =
                            serde_json::from_reader(&file)?;

                        entries.push((file, json_value));
                    }
                }
            }
            return Ok(entries);
        }
    }
    return Ok(entries);
}

pub fn extract_translations<P: AsRef<Path>>(
    directory: P,
) -> anyhow::Result<Vec<(File, BTreeMap<String, serde_json::Value>)>> {
    let mut translations = get_translations(directory.as_ref())?;
    let mut translations_keys = HashSet::new();
    let translation_regex = Regex::new(r#"(?:@lang|__)\s*\(\s*['"](.*?)['"]\s*(?:,.*)?\s*\)"#)?;

    for entry in WalkBuilder::new(directory)
        .standard_filters(true)
        .max_depth(Some(10))
        .build()
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "php") {
            let content = fs::read_to_string(path)?;
            for cap in translation_regex.captures_iter(&content) {
                if let Some(translation) = cap.get(1) {
                    translations_keys.insert(translation.as_str().to_string());
                }
            }
        }
    }

    for translation_key in translations_keys.iter() {
        for (_, map) in translations.iter_mut() {
            if !map.contains_key(translation_key) {
                map.insert(translation_key.to_string(), json!("@@@TODO@@@"));
            }
        }
    }

    for (file, map) in translations.iter_mut() {
        file.set_len(0)?;
        file.seek(SeekFrom::Start(0))?;
        file.write_all(serde_json::to_string_pretty(&map)?.as_bytes())?;
    }

    Ok(translations)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs::{File, create_dir_all},
        io::Write,
    };
    use tempdir::TempDir;

    #[test]
    fn test_traslation() {
        let tmp_dir = TempDir::new("example").unwrap();
        let tmp_path = tmp_dir.path().to_owned();
        let file_path = tmp_path.join("test.blade.php");
        let mut tmp_file = File::create(file_path.as_path()).unwrap();
        tmp_file
            .write_all(b"@lang('test1')\n__('test2')\n__('test2')\n__('test3')\nother('test5')\n@lang('test4', ['name' => 'test'])")
            .unwrap();

        create_dir_all(tmp_path.join("lang")).unwrap();
        let lang_path = tmp_path.join("lang").join("en.json");
        let mut lang_file = File::create(lang_path.as_path()).unwrap();
        lang_file
            .write_all(json!({"test1": "keep"}).to_string().as_bytes())
            .unwrap();

        let lang_path = tmp_path.join("lang").join("fr.json");
        let mut lang_file = File::create(lang_path.as_path()).unwrap();
        lang_file
            .write_all(json!({"test1": "keep"}).to_string().as_bytes())
            .unwrap();

        extract_translations(tmp_path.as_path()).unwrap();

        let files = fs::read_dir(tmp_path.join("lang")).unwrap();
        for file in files {
            let file = file.unwrap();
            let file_path = file.path();
            let file_content = fs::read_to_string(file_path).unwrap();
            let json_content: BTreeMap<String, serde_json::Value> =
                serde_json::from_str(&file_content).unwrap();
            assert_eq!(json_content.get("test1").unwrap(), "keep");
            assert_eq!(json_content.get("test2").unwrap(), "@@@TODO@@@");
            assert_eq!(json_content.get("test3").unwrap(), "@@@TODO@@@");
            assert_eq!(json_content.get("test4").unwrap(), "@@@TODO@@@");
            assert_eq!(json_content.get("test5").is_none(), true);
        }
    }
}

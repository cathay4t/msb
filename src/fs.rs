// SPDX-License-Identifier: Apache-2.0

use std::io::Read;

use crate::CliError;

pub(crate) fn read_file(file_path: &str) -> Result<String, CliError> {
    let mut fd = std::fs::File::open(file_path)?;
    let mut contents = String::new();
    fd.read_to_string(&mut contents)?;
    Ok(contents.trim_ascii().to_string())
}

pub(crate) fn read_file_as_u64(file_path: &str) -> Result<u64, CliError> {
    let content = read_file(file_path)?;
    Ok(content.trim().parse::<u64>()?)
}

pub(crate) fn read_file_as_i64(file_path: &str) -> Result<i64, CliError> {
    let content = read_file(file_path)?;
    Ok(content.trim().parse::<i64>()?)
}

pub(crate) fn read_dir(dir_path: &str) -> Result<Vec<String>, CliError> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir_path)? {
        let entry = if let Ok(e) = entry {
            e
        } else {
            continue;
        };
        let path = entry.path();
        if let Ok(content) = path.strip_prefix(dir_path) {
            if let Some(content_str) = content.to_str() {
                files.push(content_str.to_string());
            }
        }
    }
    Ok(files)
}

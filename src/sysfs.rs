// SPDX-License-Identifier: Apache-2.0

use std::io::Read;

use crate::CliError;

fn read_file(file_path: &str) -> Result<String, CliError> {
    let mut fd = std::fs::File::open(file_path)?;
    let mut contents = String::new();
    fd.read_to_string(&mut contents)?;
    Ok(contents)
}

pub(crate) fn read_file_as_u64(file_path: &str) -> Result<u64, CliError> {
    let content = read_file(file_path)?;
    Ok(content.trim().parse::<u64>()?)
}

pub(crate) fn read_file_as_i64(file_path: &str) -> Result<i64, CliError> {
    let content = read_file(file_path)?;
    Ok(content.trim().parse::<i64>()?)
}

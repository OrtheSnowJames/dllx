// SPDX-License-Identifier: AGPL-3.0

use std::{env, fs::{self, File}, io::{self, Read}, path::{Path, PathBuf}};
use zip::read::ZipArchive;
use serde::{Deserialize};
use libloading::{Library, Symbol};

// Manifest structure
#[derive(Deserialize)]
struct Manifest {
    name: String,
    platforms: std::collections::HashMap<String, String>,
}

// Extracts the .dllx (zip file) into a temporary directory
fn extract_dllx(dllx_file: &str, target_dir: &str) -> io::Result<()> {
    let file = File::open(dllx_file)?;
    let mut archive = ZipArchive::new(file)?;
    
    fs::create_dir_all(target_dir)?;
    
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = Path::new(target_dir).join(file.name());
        
        if (&*file.name()).ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            let mut outfile = File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }
    }
    
    Ok(())
}

// Read the manifest.json inside the .dllx and determine the platform-specific file to extract
fn read_manifest_from_dllx(dllx_file: &str) -> Result<Manifest, io::Error> {
    let file = File::open(dllx_file)?;
    let mut archive = ZipArchive::new(file)?;
    
    // Look for manifest.json in the archive
    let mut manifest_file = None;
    let mut file_names = Vec::new();
    for i in 0..archive.len() {
        file_names.push(archive.by_index(i)?.name().to_string());
    }
    
    for name in file_names {
        if name == "manifest.json" {
            let file = archive.by_name(&name)?;
            manifest_file = Some(file);
            break;
        }
    }
    
    match manifest_file {
        Some(file) => {
            let mut file = file;
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            let manifest: Manifest = serde_json::from_str(&content)?;
            Ok(manifest)
        }
        None => Err(io::Error::new(io::ErrorKind::NotFound, "manifest.json not found")),
    }
}

// Determine the appropriate file based on the platform
fn get_platform_file(manifest: &Manifest) -> Option<String> {
    let current_platform = env::consts::OS;
    
    match current_platform {
        "windows" => manifest.platforms.get("windows").cloned(),
        "macos" => manifest.platforms.get("macos").cloned(),
        "linux" => manifest.platforms.get("linux").cloned(),
        "ios" => manifest.platforms.get("ios").cloned(),
        "android" => manifest.platforms.get("android").cloned(),
        _ => None,
    }
}

// Load the appropriate shared library for the platform
fn load_library(dllx_file: &str, manifest: &Manifest) -> Result<Library, Box<dyn std::error::Error>> {
    let platform_file = get_platform_file(manifest)
        .ok_or("No platform-specific file found")?;
    
    // Extract the .dllx file into the target directory
    let target_dir = "./extracted";
    extract_dllx(dllx_file, target_dir)?;
    
    // The platform-specific shared library file
    let plugin_path = Path::new(target_dir).join(platform_file);
    
    // Load the shared library
    let lib = unsafe { Library::new(plugin_path)? };
    Ok(lib)
}

// Function lookup in the shared library
fn lookup_function<'a>(lib: &'a Library, func_name: &str) -> Result<Symbol<'a, unsafe fn()>, Box<dyn std::error::Error>> {
    unsafe {
        let func: Symbol<unsafe fn()> = lib.get(func_name.as_bytes())?;
        Ok(func)
    }
}

pub fn load_and_call(dllx_file: &str, function_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Read manifest from the .dllx file
    let manifest = read_manifest_from_dllx(dllx_file)?;
    
    // Load the library based on the platform
    let lib = load_library(dllx_file, &manifest)?;
    
    // Lookup and call the function 'Foo'
    let func: Symbol<unsafe fn()> = lookup_function(&lib, function_name)?;
    
    // Call the function (this assumes it's a function with no arguments and no return value)
    unsafe { func() };
    
    Ok(())
}


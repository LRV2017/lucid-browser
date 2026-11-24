//! Assemble a portable Windows distribution using the CEF binaries downloaded by cef-dll-sys.

#[cfg(target_os = "windows")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::env;
    use std::fs;
    use std::io;
    use std::path::{Path, PathBuf};

    fn copy_runtime_tree(source: &Path, destination: &Path) -> io::Result<()> {
        fs::create_dir_all(destination)?;
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            let source_path = entry.path();
            let destination_path = destination.join(entry.file_name());
            let kind = entry.file_type()?;

            if kind.is_dir() {
                copy_runtime_tree(&source_path, &destination_path)?;
            } else if kind.is_file()
                && source_path
                    .extension()
                    .is_none_or(|extension| !extension.eq_ignore_ascii_case("exe"))
            {
                fs::copy(&source_path, &destination_path)?;
            }
        }
        Ok(())
    }

    let output = env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("dist"));
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.join("../..");
    let target_dir = env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace_root.join("target"));
    let release_dir = target_dir.join("release");

    let cef_dir = cef::sys::get_cef_dir().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "cef-dll-sys could not locate its downloaded CEF runtime directory",
        )
    })?;

    let client_dll = release_dir.join("lucid_browser.dll");
    if !client_dll.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Lucid CEF client DLL was not built: {}", client_dll.display()),
        )
        .into());
    }

    let bootstrap = cef_dir.join("bootstrap.exe");
    if !bootstrap.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("CEF sandbox bootstrap was not found: {}", bootstrap.display()),
        )
        .into());
    }

    fs::create_dir_all(&output)?;
    copy_runtime_tree(&cef_dir, &output)?;
    fs::copy(&client_dll, output.join("lucid_browser.dll"))?;
    fs::copy(&bootstrap, output.join("lucid_browser.exe"))?;

    println!(
        "Packaged Lucid's CEF client and runtime from {} into {}",
        cef_dir.display(),
        output.display()
    );
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn main() {
    eprintln!("The portable CEF package helper is only available on Windows.");
}

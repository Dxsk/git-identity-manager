use std::env;
use std::path::Path;

use console::{Term, style};

use crate::git;

/// How the file was opened, which decides whether it can be checked afterwards.
pub enum Opened {
    /// The editor ran until the user closed it (or confirmed they were done).
    Finished,
    /// The file was handed to another program and we cannot tell when it is saved.
    InBackground,
}

/// Opens `path` for editing.
///
/// An editor the user configured for Git (`GIT_EDITOR`, `core.editor`,
/// `VISUAL`, `EDITOR`) wins. Otherwise the file opens in the system's default
/// application; those launchers return immediately, so in a terminal we wait
/// for the user to press Enter. Without a desktop session we fall back to
/// Git's own default editor.
pub fn open(path: &Path) -> Result<Opened, String> {
    let system = !editor_configured() && has_desktop() && open_with_system(path).is_ok();
    if !system {
        git::edit(path)?;
        return Ok(Opened::Finished);
    }

    let term = Term::stderr();
    if !term.is_term() {
        println!("Opened {} in your default editor.", path.display());
        return Ok(Opened::InBackground);
    }

    eprint!(
        "Opened in your default editor. {} ",
        style("Press Enter once you have saved the file.")
            .bold()
            .for_stderr()
    );
    term.read_line().map_err(|e| e.to_string())?;
    Ok(Opened::Finished)
}

fn editor_configured() -> bool {
    ["GIT_EDITOR", "VISUAL", "EDITOR"]
        .iter()
        .any(|var| env::var_os(var).is_some_and(|v| !v.is_empty()))
        || git::get_effective("core.editor").is_some()
}

#[cfg(target_os = "linux")]
fn has_desktop() -> bool {
    env::var_os("DISPLAY").is_some() || env::var_os("WAYLAND_DISPLAY").is_some()
}

#[cfg(not(target_os = "linux"))]
fn has_desktop() -> bool {
    true
}

#[cfg(windows)]
fn open_with_system(path: &Path) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    use std::ptr::null;
    use windows_sys::Win32::UI::Shell::{SE_ERR_NOASSOC, ShellExecuteW};
    use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    let wide = |s: &std::ffi::OsStr| s.encode_wide().chain([0]).collect::<Vec<u16>>();
    let file = wide(path.as_os_str());
    let verb = wide("open".as_ref());

    // SAFETY: every pointer is either null or a NUL-terminated UTF-16 buffer
    // that outlives the call.
    let code = unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            verb.as_ptr(),
            file.as_ptr(),
            null(),
            null(),
            SW_SHOWNORMAL,
        )
    } as usize;

    match code {
        c if c > 32 => Ok(()),
        // No application is registered for .json files: Notepad is always there.
        c if c == SE_ERR_NOASSOC as usize => std::process::Command::new("notepad.exe")
            .arg(path)
            .spawn()
            .map(drop)
            .map_err(|e| format!("Could not start Notepad: {e}")),
        c => Err(format!(
            "Could not open {} (ShellExecute error {c}).",
            path.display()
        )),
    }
}

#[cfg(not(windows))]
fn open_with_system(path: &Path) -> Result<(), String> {
    // `open -t` picks the default text editor rather than the app tied to .json.
    let mut cmd = if cfg!(target_os = "macos") {
        let mut c = std::process::Command::new("open");
        c.arg("-t");
        c
    } else {
        std::process::Command::new("xdg-open")
    };
    let status = cmd
        .arg(path)
        .status()
        .map_err(|e| format!("Could not open {}: {e}", path.display()))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "Could not open {} with the default application.",
            path.display()
        ))
    }
}

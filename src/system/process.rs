#![cfg(windows)]

use std::{
    ffi::OsString,
    mem::MaybeUninit,
    os::windows::ffi::OsStringExt,
    path::{Path, PathBuf},
};

use windows::Win32::{
    Foundation::{CloseHandle, INVALID_HANDLE_VALUE},
    System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, MODULEENTRY32W, Module32FirstW, Module32NextW, PROCESSENTRY32W,
        Process32FirstW, Process32NextW, TH32CS_SNAPMODULE, TH32CS_SNAPPROCESS,
    },
};

struct ModuleIterator {
    h_module_snap: windows::Win32::Foundation::HANDLE,
    me32: MODULEENTRY32W,
    first: bool,
}

impl ModuleIterator {
    fn new(h_module_snap: windows::Win32::Foundation::HANDLE) -> Self {
        #[allow(
            clippy::cast_possible_truncation,
            clippy::uninit_assumed_init,
            invalid_value
        )]
        Self {
            h_module_snap,
            me32: MODULEENTRY32W {
                dwSize: std::mem::size_of::<MODULEENTRY32W>() as u32,
                ..unsafe { MaybeUninit::uninit().assume_init() }
            },
            first: true,
        }
    }
}

impl Iterator for ModuleIterator {
    type Item = MODULEENTRY32W;

    fn next(&mut self) -> Option<Self::Item> {
        if self.first {
            unsafe { Module32FirstW(self.h_module_snap, &mut self.me32) }.ok()?;
        } else {
            unsafe { Module32NextW(self.h_module_snap, &mut self.me32) }.ok()?;
        }

        self.first = false;

        Some(self.me32)
    }
}

impl Drop for ModuleIterator {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.h_module_snap).ok() };
    }
}

struct ProcessIterator {
    h_process_snap: windows::Win32::Foundation::HANDLE,
    pe32: PROCESSENTRY32W,
    first: bool,
}

impl ProcessIterator {
    fn new(h_process_snap: windows::Win32::Foundation::HANDLE) -> Self {
        #[allow(
            clippy::cast_possible_truncation,
            clippy::uninit_assumed_init,
            invalid_value
        )]
        Self {
            h_process_snap,
            pe32: PROCESSENTRY32W {
                dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
                ..unsafe { MaybeUninit::uninit().assume_init() }
            },
            first: true,
        }
    }
}

impl Iterator for ProcessIterator {
    type Item = PROCESSENTRY32W;

    fn next(&mut self) -> Option<Self::Item> {
        if self.first {
            unsafe { Process32FirstW(self.h_process_snap, &mut self.pe32) }.ok()?;
        } else {
            unsafe { Process32NextW(self.h_process_snap, &mut self.pe32) }.ok()?;
        }

        self.first = false;

        Some(self.pe32)
    }
}

impl Drop for ProcessIterator {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.h_process_snap).ok() };
    }
}

unsafe fn match_process_path(
    pe32: &PROCESSENTRY32W,
    base_path: impl AsRef<Path>,
) -> windows::core::Result<Vec<PathBuf>> {
    let pid = pe32.th32ProcessID;
    let mut paths = Vec::new();

    let h_module_snap = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPMODULE, pid)? };

    if h_module_snap == INVALID_HANDLE_VALUE {
        return Ok(Vec::new());
    }

    let module_iterator = ModuleIterator::new(h_module_snap);

    for me32 in module_iterator {
        let path = PathBuf::from(get_compare_string(&me32.szExePath));

        if path.starts_with(&base_path) {
            paths.push(path);
        }
    }

    Ok(paths)
}

fn get_compare_string(exe_file: &[u16]) -> String {
    let os_string = OsString::from_wide(exe_file);
    let utf8_string = os_string.to_string_lossy();
    let trimmed = utf8_string.trim_end_matches('\0');

    trimmed.to_string()
}

pub enum Process {
    #[allow(dead_code)]
    ExactExe(PathBuf),
    BaseDir(PathBuf),
}

impl Process {
    /// Finds if the process is running
    ///
    /// If passed a [`Process::BaseDir`], it will check if any processes are running in that directory
    /// If passed a [`Process::ExactExe`], it will check if any processes are running using that exact executable
    pub unsafe fn find_running(self) -> windows::core::Result<bool> {
        let mut proc_running = false;

        let h_process_snap = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)? };

        if h_process_snap == INVALID_HANDLE_VALUE {
            proc_running = false;
        } else {
            let path = match &self {
                Process::ExactExe(path) | Process::BaseDir(path) => path,
            };

            let process_iterator = ProcessIterator::new(h_process_snap);

            for pe32 in process_iterator {
                if let Self::ExactExe(path) = &self {
                    if let Some(file_name) = path.file_name() {
                        if get_compare_string(&pe32.szExeFile) == file_name.to_string_lossy() {
                            proc_running = true;
                            break;
                        }
                    }
                }

                // This can sometimes return an error, but we don't care about it (it usually means the process is irrevelant)
                let compare = unsafe { match_process_path(&pe32, path) }.unwrap_or_default();

                if !compare.is_empty() {
                    proc_running = true;
                    break;
                }
            }
        }

        Ok(proc_running)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // This test looks for 'cargo.exe', which can only exist on Windows
    #[cfg_attr(not(windows), ignore)]
    fn test_find_running_process_exact() {
        let cargo_path = which::which("cargo").unwrap();
        let process = Process::ExactExe(cargo_path);
        let result = unsafe { process.find_running() };

        match result {
            Err(e) => {
                eprintln!("Finding process returned an error: {e}");
                panic!("Finding process returned an error");
            }
            Ok(false) => panic!("Could not find running process"),
            _ => {}
        }
    }

    #[test]
    // This test looks for 'cargo.exe', which can only exist on Windows
    #[cfg_attr(not(windows), ignore)]
    fn test_find_running_process_base() {
        let cargo_path = which::which("cargo").unwrap();
        let base_dir = cargo_path.parent().unwrap();
        let process = Process::BaseDir(base_dir.to_path_buf());
        let result = unsafe { process.find_running() };

        match result {
            Err(e) => {
                eprintln!("Finding process returned an error: {e}");
                panic!("Finding process returned an error");
            }
            Ok(false) => panic!("Could not find running process"),
            _ => {}
        }
    }
}

use std::{ffi::OsString, fs::File, path::PathBuf};

use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::HMODULE,
        System::{Environment::GetCommandLineW, LibraryLoader::GetModuleFileNameW},
    },
};

const MAX_FILENAME_SIZE: usize = 512;

// fn compute_program_length(const wchar_t* commandline) -> usize
// {
//   int i = 0;

//   if (commandline[0] == L'"') {
//     // Wait till end of string
//     i++;

//     for (;;) {
//       wchar_t c = commandline[i++];

//       if (c == 0)
//         return i - 1;
//       else if (c == L'\\')
//         i++;
//       else if (c == L'"')
//         return i;
//     }
//   } else {
//     for (;;) {
//       wchar_t c = commandline[i++];

//       if (c == 0)
//         return i - 1;
//       else if (c == L'\\')
//         i++;
//       else if (c == L' ')
//         return i;
//     }
//   }
// }

struct ExePath {
    path: PathBuf,
}

impl ExePath {
    pub fn new() -> std::io::Result<Self> {
        Ok(Self {
            path: std::env::current_exe()?,
        })
    }

    pub fn shim_path(&self) -> PathBuf {
        self.path.with_extension("shim")
    }
}

fn main() {
    let command_line: PCWSTR = unsafe { GetCommandLineW() };
    let file_path = ExePath::new().expect("valid executable path");

    let shim_file = File::open(file_path.shim_path()).expect("present and readable shim file");

    println!("Hello, world!");
}

use std::ffi::OsString;

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

struct FileName {
    file_name: [u16; MAX_FILENAME_SIZE + 2],
}

impl FileName {
    pub fn load() -> Self {
        let mut file_name = [0; MAX_FILENAME_SIZE + 2];

        unsafe {
            GetModuleFileNameW(HMODULE(0), &mut file_name);
        }

        Self { file_name }
    }
}

impl std::fmt::Display for FileName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string_lossy = String::from_utf16_lossy(&self.file_name);
        let trimmed = string_lossy.trim_end_matches('\0');

        write!(f, "{trimmed}")
    }
}

fn main() {
    let command_line: PCWSTR = unsafe { GetCommandLineW() };
    let file_name = FileName::load();
    dbg!(file_name.to_string());

    println!("Hello, world!");
}

#![warn(clippy::all, clippy::pedantic, rust_2018_idioms)]

// TODO: Replace regex with glob

mod commands;
mod logging;
mod opt;

use std::{mem::MaybeUninit, path::Path};

use clap::Parser;

use commands::Commands;

/// Scoop utilities that can replace the slowest parts of Scoop, and run anywhere from 30-100 times faster
#[derive(Debug, Parser)]
#[clap(about, long_about, author, version)]
struct Args {
    #[command(subcommand)]
    command: Commands,

    #[clap(long, global = true, help = "Disable terminal formatting")]
    no_color: bool,
}

fn main() -> anyhow::Result<()> {
    logging::panics::handle();

    let args = Args::parse();
    if args.no_color {
        colored::control::set_override(false);
    }

    args.command.run()
}

/// Get the owner of a file path
///
/// # Errors
/// - Interacting with system I/O
///
/// # Panics
/// - Owner's name isn't valid utf8
#[deprecated(note = "Doesn't work properly, and I have no current plans to fix it ")]
pub fn file_owner(path: impl AsRef<Path>) -> std::io::Result<String> {
    use std::{fs::File, os::windows::io::AsRawHandle};
    use windows::{
        core::{PCSTR, PSTR},
        Win32::{
            Foundation::{HANDLE, PSID},
            Security::{
                Authorization::{GetSecurityInfo, SE_FILE_OBJECT},
                LookupAccountSidA, OWNER_SECURITY_INFORMATION,
            },
        },
    };

    let file = File::open(path.as_ref().join("current/install.json"))?;
    let handle = HANDLE(file.as_raw_handle() as isize);

    let owner_psid: MaybeUninit<PSID> = MaybeUninit::uninit();

    unsafe {
        GetSecurityInfo(
            handle,
            SE_FILE_OBJECT,
            OWNER_SECURITY_INFORMATION,
            Some(owner_psid.as_ptr().cast_mut()),
            None,
            None,
            None,
            None,
        )?;
    }

    let owner_name = PSTR::null();

    unsafe {
        LookupAccountSidA(
            PCSTR::null(),
            owner_psid.assume_init(),
            owner_name,
            std::ptr::null_mut(),
            PSTR::null(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )?;
    }

    Ok(unsafe { owner_name.to_string().expect("valid utf8 name") })
}

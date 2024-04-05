use clap::Parser;

use sfsu::diagnostics::{Diagnostics, LongPathsStatus};

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(from_global)]
    json: bool,
}

impl super::Command for Args {
    fn runner(self) -> Result<(), anyhow::Error> {
        let diagnostics = Diagnostics::collect()?;

        if diagnostics.main_bucket {
            println!("✅ Main bucket is installed");
        } else {
            println!("❌ Main bucket is not installed");
            println!("\tRun `scoop bucket add main` to install it");
        }

        // if diagnostics.windows_defender {
        //     println!("✅ Windows Defender is ignoring the Scoop directory");
        // } else {
        //     println!("❌ Windows Defender is not ignoring the Scoop directory");
        // }

        if diagnostics.windows_developer {
            println!("✅ Windows Developer Mode is enabled");
        } else {
            println!("❌ Windows Developer Mode is not enabled");
            println!("\tWindows Developer Mode is not enabled. Operations relevant to symlinks may fail without proper rights");
        }

        match diagnostics.long_paths {
            LongPathsStatus::Enabled => println!("✅ Long paths are enabled"),
            LongPathsStatus::OldWindows => {
                println!("❌ This version of Windows does not support long paths");
            }
            LongPathsStatus::Disabled => {
                println!("❌ Long paths are disabled");
                println!("\tRun `Set-ItemProperty 'HKLM:\\SYSTEM\\CurrentControlSet\\Control\\FileSystem' -Name 'LongPathsEnabled' -Value 1` as admin to enable it");
            }
        }

        Ok(())
    }
}

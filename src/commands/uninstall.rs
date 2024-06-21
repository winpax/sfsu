use clap::Parser;
use itertools::Itertools;
use quork::traits::truthy::ContainsTruth;
use sprinkles::{
    config,
    contexts::ScoopContext,
    packages::reference::{manifest, package},
    Architecture,
};

use crate::output::colours::eprintln_red;

#[derive(Debug, Clone, Parser)]
/// Uninstall an app
pub struct Args {
    /// The package(s) to uninstall
    packages: Vec<package::Reference>,

    #[clap(short, long)]
    /// Remove all persistent data
    purge: bool,

    #[clap(from_global)]
    global: bool,
}

impl super::Command for Args {
    fn needs_elevation(&self) -> bool {
        self.global
    }

    async fn runner(self, ctx: &impl ScoopContext) -> anyhow::Result<()> {
        let valid_packages = self.packages.into_iter().filter(|package| {
            if matches!(
                package.manifest,
                manifest::Reference::BucketNamePair { .. } | manifest::Reference::Name(_)
            ) {
                if package.name().is_some_and(|name| name == "scoop") {
                    eprintln_red!("Uninstalling Scoop is not supported yet. Please run `scoop.ps1 uninstall scoop` instead");
                    false
                } else if package.installed(ctx).contains_truth() {
                    true
                } else {
                    eprintln_red!("'{}' not installed", package.name().expect("name exists. this is a bug please report it"));
                    false
                }
            } else {
                eprintln_red!("Invalid package reference. You cannot reference a file or url for uninstallation");
                false
            }
        }).collect_vec();

        if valid_packages.is_empty() {
            eprintln_red!("No packages provided");
            return Ok(());
        }

        let packages_with_manifest = {
            let packages_future = valid_packages
                .into_iter()
                .map(|package| async { package.open_handle(ctx).await });

            futures::future::try_join_all(packages_future).await?
        };

        for handle in packages_with_manifest {
            handle.unlink_current()?;

            // let version_dir = handle.version_dir();
            // let persist_dir = handle.persist_dir();

            let manifest = handle.local_manifest()?;

            let install_config = manifest.install_config(Architecture::ARCH);

            // Run the pre-uninstall script
            if let Some(ref pre_uninstall) = install_config.pre_uninstall {
                let script_runner = pre_uninstall.save(ctx)?;
                script_runner.run().await?;
            }
        }

        todo!()
    }
}

// Copyright © Nabob Labs
// SPDX-License-Identifier: Apache-2.0

use crate::{
    common::types::{AccountAddressWrapper, CliCommand, CliTypedResult, MovePackageOptions},
    move_tool::IncludedArtifacts,
};
use nabob_framework::{BuildOptions, BuiltPackage};
use async_trait::async_trait;
use clap::Parser;
use move_compiler_v2::Experiment;
use move_linter::MoveLintChecks;
use move_model::metadata::{CompilerVersion, LanguageVersion, LATEST_STABLE_LANGUAGE_VERSION};
use move_package::source_package::std_lib::StdVersion;
use std::{collections::BTreeMap, path::PathBuf};

/// Run a Lint tool to show additional warnings about the current package, in addition to ordinary
/// warnings and/or errors generated by the Move 2 compiler.
#[derive(Debug, Clone, Parser)]
pub struct LintPackage {
    /// Path to a move package (the folder with a Move.toml file).  Defaults to current directory.
    #[clap(long, value_parser)]
    pub package_dir: Option<PathBuf>,

    /// Specify the path to save the compiled bytecode files which lint generates while
    /// running checks.
    /// Defaults to `<package_dir>/build`
    #[clap(long, value_parser)]
    pub output_dir: Option<PathBuf>,

    /// ...or --language LANGUAGE_VERSION
    /// Specify the language version to be supported.
    /// Defaults to the latest stable language version.
    #[clap(long, value_parser = clap::value_parser!(LanguageVersion),
           alias = "language",
           default_value = LATEST_STABLE_LANGUAGE_VERSION,
           verbatim_doc_comment)]
    pub language_version: Option<LanguageVersion>,

    /// Named addresses for the move binary
    ///
    /// Example: alice=0x1234, bob=0x5678
    ///
    /// Note: This will fail if there are duplicates in the Move.toml file remove those first.
    #[clap(long, value_parser = crate::common::utils::parse_map::<String, AccountAddressWrapper>, default_value = "")]
    pub(crate) named_addresses: BTreeMap<String, AccountAddressWrapper>,

    /// Override the standard library version by mainnet/testnet/devnet
    #[clap(long, value_parser)]
    pub override_std: Option<StdVersion>,

    /// Skip pulling the latest git dependencies
    ///
    /// If you don't have a network connection, the compiler may fail due
    /// to no ability to pull git dependencies.  This will allow overriding
    /// this for local development.
    #[clap(long)]
    pub(crate) skip_fetch_latest_git_deps: bool,

    /// Do not complain about unknown attributes in Move code.
    #[clap(long)]
    pub skip_attribute_checks: bool,

    /// Enables dev mode, which uses all dev-addresses and dev-dependencies
    ///
    /// Dev mode allows for changing dependencies and addresses to the preset [dev-addresses] and
    /// [dev-dependencies] fields.  This works both inside and out of tests for using preset values.
    ///
    /// Currently, it also additionally pulls in all test compilation artifacts
    #[clap(long)]
    pub dev: bool,

    /// Experiments
    #[clap(long, hide(true))]
    pub experiments: Vec<String>,
}

impl LintPackage {
    fn to_move_options(&self) -> MovePackageOptions {
        let LintPackage {
            dev,
            package_dir,
            output_dir,
            named_addresses,
            override_std,
            skip_fetch_latest_git_deps,
            language_version,
            skip_attribute_checks,
            experiments,
        } = self.clone();
        MovePackageOptions {
            dev,
            package_dir,
            output_dir,
            named_addresses,
            override_std,
            skip_fetch_latest_git_deps,
            language_version,
            skip_attribute_checks,
            experiments,
            ..MovePackageOptions::new()
        }
    }
}

#[async_trait]
impl CliCommand<&'static str> for LintPackage {
    fn command_name(&self) -> &'static str {
        "LintPackage"
    }

    async fn execute(self) -> CliTypedResult<&'static str> {
        let move_options = MovePackageOptions {
            compiler_version: Some(CompilerVersion::latest_stable()),
            ..self.to_move_options()
        };
        let more_experiments = vec![
            Experiment::LINT_CHECKS.to_string(),
            Experiment::SPEC_CHECK.to_string(),
            Experiment::SEQS_IN_BINOPS_CHECK.to_string(),
            Experiment::ACCESS_CHECK.to_string(),
            Experiment::STOP_AFTER_EXTENDED_CHECKS.to_string(),
        ];
        let package_path = move_options.get_package_path()?;
        let included_artifacts = IncludedArtifacts::Sparse;
        let build_options = BuildOptions {
            ..included_artifacts.build_options_with_experiments(
                &move_options,
                more_experiments,
                true,
            )?
        };

        let build_config = BuiltPackage::create_build_config(&build_options)?;
        let resolved_graph =
            BuiltPackage::prepare_resolution_graph(package_path, build_config.clone())?;
        BuiltPackage::build_with_external_checks(
            resolved_graph,
            build_options,
            build_config,
            vec![MoveLintChecks::make()],
        )?;

        Ok("succeeded")
    }
}

// Copyright © Nabob Labs
// SPDX-License-Identifier: Apache-2.0

use nabob_framework::extended_checks;
use nabob_gas_schedule::{MiscGasParameters, NativeGasParameters, LATEST_GAS_FEATURE_VERSION};
use nabob_types::on_chain_config::{Features, TimedFeaturesBuilder};
use nabob_vm::natives;
use move_vm_runtime::native_functions::NativeFunctionTable;

// move_stdlib has the testing feature enabled to include debug native functions
pub fn nabob_debug_natives(
    native_gas_parameters: NativeGasParameters,
    misc_gas_params: MiscGasParameters,
) -> NativeFunctionTable {
    // As a side effect, also configure for unit testing
    natives::configure_for_unit_test();
    extended_checks::configure_extended_checks_for_unit_test();
    // Return all natives -- build with the 'testing' feature, therefore containing
    // debug related functions.
    natives::nabob_natives(
        LATEST_GAS_FEATURE_VERSION,
        native_gas_parameters,
        misc_gas_params,
        TimedFeaturesBuilder::enable_all().build(),
        Features::default(),
    )
}

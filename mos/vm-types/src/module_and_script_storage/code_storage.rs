// Copyright © Nabob Labs
// SPDX-License-Identifier: Apache-2.0

use crate::module_and_script_storage::module_storage::NabobModuleStorage;
use move_vm_runtime::CodeStorage;

/// Represents code storage used by the Nabob blockchain, capable of caching scripts and modules.
pub trait NabobCodeStorage: NabobModuleStorage + CodeStorage {}

impl<T: NabobModuleStorage + CodeStorage> NabobCodeStorage for T {}

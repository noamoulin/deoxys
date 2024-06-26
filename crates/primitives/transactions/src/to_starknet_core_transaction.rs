use std::vec::Vec;

/// Here we transform starknet-api transactions into starknet-core trasnactions
use mp_felt::Felt252Wrapper;
use starknet_api::transaction::{
    DeclareTransaction, DeclareTransactionV0V1, DeclareTransactionV2, DeclareTransactionV3, DeployAccountTransaction,
    DeployAccountTransactionV1, DeployAccountTransactionV3, InvokeTransaction, InvokeTransactionV0,
    InvokeTransactionV1, InvokeTransactionV3, Resource, ResourceBoundsMapping, Transaction,
};
use starknet_core::types::{ResourceBounds, ResourceBoundsMapping as CoreResourceBoundsMapping};
use starknet_crypto::FieldElement;

// TODO: is this function needed?
#[allow(dead_code)]
fn cast_vec_of_felt_252_wrappers(data: Vec<Felt252Wrapper>) -> Vec<FieldElement> {
    // Non-copy but less dangerous than transmute
    // https://doc.rust-lang.org/std/mem/fn.transmute.html#alternatives

    // Unsafe code but all invariants are checked:

    // 1. ptr must have been allocated using the global allocator -> data is allocated with the Global
    //    allocator.
    // 2. T needs to have the same alignment as what ptr was allocated with -> Felt252Wrapper uses
    //    transparent representation of the inner type.
    // 3. The allocated size in bytes needs to be the same as the pointer -> As FieldElement and
    //    Felt252Wrapper have the same size, and capacity is taken directly from the data Vector, we
    //    will have the same allocated byte size.
    // 4. Length needs to be less than or equal to capacity -> data.len() is always less than or equal
    //    to data.capacity()
    // 5. The first length values must be properly initialized values of type T -> ok since we use data
    //    which was correctly allocated
    // 6. capacity needs to be the capacity that the pointer was allocated with -> data.as_mut_ptr()
    //    returns a pointer to memory having at least capacity initialized memory
    // 7. The allocated size in bytes must be no larger than isize::MAX -> data.capacity() will never be
    //    bigger than isize::MAX (https://doc.rust-lang.org/std/vec/struct.Vec.html#panics-7)
    let mut data = core::mem::ManuallyDrop::new(data);
    unsafe { alloc::vec::Vec::from_raw_parts(data.as_mut_ptr() as *mut FieldElement, data.len(), data.capacity()) }
}

pub fn to_starknet_core_tx(tx: Transaction, transaction_hash: FieldElement) -> starknet_core::types::Transaction {
    match tx {
        Transaction::Declare(tx) => {
            let tx = match tx {
                DeclareTransaction::V0(DeclareTransactionV0V1 {
                    max_fee,
                    signature,
                    nonce: _,
                    class_hash,
                    sender_address,
                }) => starknet_core::types::DeclareTransaction::V0(starknet_core::types::DeclareTransactionV0 {
                    transaction_hash,
                    max_fee: Felt252Wrapper::from(max_fee.0).into(),
                    signature: signature
                        .0
                        .iter()
                        .map(|x| Felt252Wrapper::from(*x).into())
                        .collect::<Vec<FieldElement>>(),
                    class_hash: Felt252Wrapper::from(class_hash.0).into(),
                    sender_address: Felt252Wrapper::from(sender_address.0).into(),
                }),
                DeclareTransaction::V1(DeclareTransactionV0V1 {
                    max_fee,
                    signature,
                    nonce,
                    class_hash,
                    sender_address,
                    ..
                }) => starknet_core::types::DeclareTransaction::V1(starknet_core::types::DeclareTransactionV1 {
                    transaction_hash,
                    max_fee: Felt252Wrapper::from(max_fee.0).into(),
                    signature: signature
                        .0
                        .iter()
                        .map(|x| Felt252Wrapper::from(*x).into())
                        .collect::<Vec<FieldElement>>(),
                    nonce: Felt252Wrapper::from(nonce.0).into(),
                    class_hash: Felt252Wrapper::from(class_hash.0).into(),
                    sender_address: Felt252Wrapper::from(sender_address.0).into(),
                }),
                DeclareTransaction::V2(DeclareTransactionV2 {
                    max_fee,
                    signature,
                    nonce,
                    class_hash,
                    sender_address,
                    compiled_class_hash,
                    ..
                }) => starknet_core::types::DeclareTransaction::V2(starknet_core::types::DeclareTransactionV2 {
                    transaction_hash,
                    max_fee: Felt252Wrapper::from(max_fee.0).into(),
                    signature: signature
                        .0
                        .iter()
                        .map(|x| Felt252Wrapper::from(*x).into())
                        .collect::<Vec<FieldElement>>(),
                    nonce: Felt252Wrapper::from(nonce.0).into(),
                    class_hash: Felt252Wrapper::from(class_hash.0).into(),
                    sender_address: Felt252Wrapper::from(sender_address.0).into(),
                    compiled_class_hash: Felt252Wrapper::from(compiled_class_hash.0).into(),
                }),
                DeclareTransaction::V3(DeclareTransactionV3 {
                    resource_bounds,
                    tip,
                    signature,
                    nonce,
                    class_hash,
                    compiled_class_hash,
                    sender_address,
                    nonce_data_availability_mode,
                    fee_data_availability_mode,
                    paymaster_data,
                    account_deployment_data,
                }) => starknet_core::types::DeclareTransaction::V3(starknet_core::types::DeclareTransactionV3 {
                    transaction_hash,
                    resource_bounds: api_resources_to_core_ressources(resource_bounds),
                    tip: tip.0,
                    signature: signature
                        .0
                        .iter()
                        .map(|x| Felt252Wrapper::from(*x).into())
                        .collect::<Vec<FieldElement>>(),
                    nonce: Felt252Wrapper::from(nonce.0).into(),
                    class_hash: Felt252Wrapper::from(class_hash.0).into(),
                    compiled_class_hash: Felt252Wrapper::from(compiled_class_hash.0).into(),
                    sender_address: Felt252Wrapper::from(sender_address.0).into(),
                    nonce_data_availability_mode: api_da_to_core_da(nonce_data_availability_mode).unwrap(),
                    fee_data_availability_mode: api_da_to_core_da(fee_data_availability_mode).unwrap(),
                    paymaster_data: paymaster_data
                        .0
                        .iter()
                        .map(|x| Felt252Wrapper::from(*x).into())
                        .collect::<Vec<FieldElement>>(),
                    account_deployment_data: account_deployment_data
                        .0
                        .iter()
                        .map(|x| Felt252Wrapper::from(*x).into())
                        .collect::<Vec<FieldElement>>(),
                }),
            };

            starknet_core::types::Transaction::Declare(tx)
        }
        Transaction::DeployAccount(tx) => {
            let tx = match tx {
                DeployAccountTransaction::V1(DeployAccountTransactionV1 {
                    max_fee,
                    signature,
                    nonce,
                    contract_address_salt,
                    constructor_calldata,
                    class_hash,
                    ..
                }) => starknet_core::types::DeployAccountTransaction::V1(
                    starknet_core::types::DeployAccountTransactionV1 {
                        transaction_hash,
                        max_fee: Felt252Wrapper::from(max_fee.0).into(),
                        signature: signature
                            .0
                            .iter()
                            .map(|x| Felt252Wrapper::from(*x).into())
                            .collect::<Vec<FieldElement>>(),
                        nonce: Felt252Wrapper::from(nonce.0).into(),
                        contract_address_salt: Felt252Wrapper::from(contract_address_salt.0).into(),
                        constructor_calldata: constructor_calldata
                            .0
                            .iter()
                            .map(|x| Felt252Wrapper::from(*x).into())
                            .collect::<Vec<FieldElement>>(),
                        class_hash: Felt252Wrapper::from(class_hash.0).into(),
                    },
                ),
                DeployAccountTransaction::V3(DeployAccountTransactionV3 {
                    resource_bounds,
                    tip,
                    signature,
                    nonce,
                    class_hash,
                    contract_address_salt,
                    constructor_calldata,
                    nonce_data_availability_mode,
                    fee_data_availability_mode,
                    paymaster_data,
                }) => starknet_core::types::DeployAccountTransaction::V3(
                    starknet_core::types::DeployAccountTransactionV3 {
                        transaction_hash,
                        resource_bounds: api_resources_to_core_ressources(resource_bounds),
                        tip: tip.0,
                        signature: signature
                            .0
                            .iter()
                            .map(|x| Felt252Wrapper::from(*x).into())
                            .collect::<Vec<FieldElement>>(),
                        nonce: Felt252Wrapper::from(nonce.0).into(),
                        class_hash: Felt252Wrapper::from(class_hash.0).into(),
                        contract_address_salt: Felt252Wrapper::from(contract_address_salt.0).into(),
                        constructor_calldata: constructor_calldata
                            .0
                            .iter()
                            .map(|x| Felt252Wrapper::from(*x).into())
                            .collect::<Vec<FieldElement>>(),
                        nonce_data_availability_mode: api_da_to_core_da(nonce_data_availability_mode).unwrap(),
                        fee_data_availability_mode: api_da_to_core_da(fee_data_availability_mode).unwrap(),
                        paymaster_data: paymaster_data
                            .0
                            .iter()
                            .map(|x| Felt252Wrapper::from(*x).into())
                            .collect::<Vec<FieldElement>>(),
                    },
                ),
            };

            starknet_core::types::Transaction::DeployAccount(tx)
        }
        Transaction::Deploy(tx) => {
            let tx = starknet_core::types::DeployTransaction {
                transaction_hash,
                contract_address_salt: Felt252Wrapper::from(tx.contract_address_salt.0).into(),
                constructor_calldata: tx
                    .constructor_calldata
                    .0
                    .iter()
                    .map(|x| Felt252Wrapper::from(*x).into())
                    .collect::<Vec<FieldElement>>(),
                class_hash: Felt252Wrapper::from(tx.class_hash.0).into(),
                version: Felt252Wrapper::ZERO.into(),
            };

            starknet_core::types::Transaction::Deploy(tx)
        }
        Transaction::Invoke(tx) => {
            let tx = match tx {
                InvokeTransaction::V0(InvokeTransactionV0 {
                    max_fee,
                    signature,
                    contract_address,
                    entry_point_selector,
                    calldata,
                }) => starknet_core::types::InvokeTransaction::V0(starknet_core::types::InvokeTransactionV0 {
                    transaction_hash,
                    max_fee: Felt252Wrapper::from(max_fee.0).into(),
                    signature: signature
                        .0
                        .iter()
                        .map(|x| Felt252Wrapper::from(*x).into())
                        .collect::<Vec<FieldElement>>(),
                    contract_address: Felt252Wrapper::from(contract_address.0).into(),
                    entry_point_selector: Felt252Wrapper::from(entry_point_selector.0).into(),
                    calldata: calldata.0.iter().map(|x| Felt252Wrapper::from(*x).into()).collect::<Vec<FieldElement>>(),
                }),
                InvokeTransaction::V1(InvokeTransactionV1 {
                    max_fee,
                    signature,
                    nonce,
                    sender_address,
                    calldata,
                    ..
                }) => starknet_core::types::InvokeTransaction::V1(starknet_core::types::InvokeTransactionV1 {
                    transaction_hash,
                    max_fee: Felt252Wrapper::from(max_fee.0).into(),
                    signature: signature
                        .0
                        .iter()
                        .map(|x| Felt252Wrapper::from(*x).into())
                        .collect::<Vec<FieldElement>>(),
                    nonce: Felt252Wrapper::from(nonce.0).into(),
                    sender_address: Felt252Wrapper::from(sender_address.0).into(),
                    calldata: calldata.0.iter().map(|x| Felt252Wrapper::from(*x).into()).collect::<Vec<FieldElement>>(),
                }),
                InvokeTransaction::V3(InvokeTransactionV3 {
                    resource_bounds,
                    tip,
                    signature,
                    nonce,
                    sender_address,
                    calldata,
                    nonce_data_availability_mode,
                    fee_data_availability_mode,
                    paymaster_data,
                    account_deployment_data,
                }) => starknet_core::types::InvokeTransaction::V3(starknet_core::types::InvokeTransactionV3 {
                    transaction_hash,
                    resource_bounds: api_resources_to_core_ressources(resource_bounds),
                    tip: tip.0,
                    signature: signature
                        .0
                        .iter()
                        .map(|x| Felt252Wrapper::from(*x).into())
                        .collect::<Vec<FieldElement>>(),
                    nonce: Felt252Wrapper::from(nonce.0).into(),
                    sender_address: Felt252Wrapper::from(sender_address.0).into(),
                    calldata: calldata.0.iter().map(|x| Felt252Wrapper::from(*x).into()).collect::<Vec<FieldElement>>(),
                    nonce_data_availability_mode: api_da_to_core_da(nonce_data_availability_mode).unwrap(),
                    fee_data_availability_mode: api_da_to_core_da(fee_data_availability_mode).unwrap(),
                    paymaster_data: paymaster_data
                        .0
                        .iter()
                        .map(|x| Felt252Wrapper::from(*x).into())
                        .collect::<Vec<FieldElement>>(),
                    account_deployment_data: account_deployment_data
                        .0
                        .iter()
                        .map(|x| Felt252Wrapper::from(*x).into())
                        .collect::<Vec<FieldElement>>(),
                }),
            };

            starknet_core::types::Transaction::Invoke(tx)
        }
        Transaction::L1Handler(tx) => {
            let tx = starknet_core::types::L1HandlerTransaction {
                transaction_hash,
                version: FieldElement::ZERO,
                nonce: u64::try_from(Felt252Wrapper::from(tx.nonce.0)).unwrap(),
                contract_address: Felt252Wrapper::from(tx.contract_address).into(),
                entry_point_selector: Felt252Wrapper::from(tx.entry_point_selector).into(),
                calldata: tx.calldata.0.iter().map(|x| Felt252Wrapper::from(*x).into()).collect::<Vec<FieldElement>>(),
            };

            starknet_core::types::Transaction::L1Handler(tx)
        }
    }
}

// TODO (Tbelleng): Custom function here so check if value are correct
pub fn api_resources_to_core_ressources(resource: ResourceBoundsMapping) -> CoreResourceBoundsMapping {
    let l1_gas = resource.0.get(&Resource::L1Gas).unwrap();

    let l2_gas = resource.0.get(&Resource::L2Gas).unwrap();

    let resource_for_l1: starknet_core::types::ResourceBounds =
        ResourceBounds { max_amount: l1_gas.max_amount, max_price_per_unit: l1_gas.max_price_per_unit };

    let resource_for_l2: starknet_core::types::ResourceBounds =
        ResourceBounds { max_amount: l2_gas.max_amount, max_price_per_unit: l2_gas.max_price_per_unit };

    CoreResourceBoundsMapping { l1_gas: resource_for_l1, l2_gas: resource_for_l2 }
}

pub fn api_da_to_core_da(
    mode: starknet_api::data_availability::DataAvailabilityMode,
) -> Option<starknet_core::types::DataAvailabilityMode> {
    match mode {
        starknet_api::data_availability::DataAvailabilityMode::L1 => {
            Some(starknet_core::types::DataAvailabilityMode::L1)
        }
        starknet_api::data_availability::DataAvailabilityMode::L2 => {
            Some(starknet_core::types::DataAvailabilityMode::L2)
        }
    }
}

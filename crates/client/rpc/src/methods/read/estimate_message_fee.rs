use jsonrpsee::core::RpcResult;
use mc_genesis_data_provider::GenesisProvider;
use mp_hashers::HasherT;
use mp_types::block::DBlockT;
use pallet_starknet_runtime_api::{ConvertTransactionRuntimeApi, StarknetRuntimeApi};
use sc_client_api::backend::{Backend, StorageProvider};
use sc_client_api::BlockBackend;
use sc_transaction_pool::ChainApi;
use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use starknet_core::types::{BlockId, FeeEstimate, MsgFromL1, PriceUnit};
use starknet_ff::FieldElement;

use crate::errors::StarknetRpcApiError;
use crate::Starknet;

/// Estimate the L2 fee of a message sent on L1
///
/// # Arguments
///
/// * `message` - the message to estimate
/// * `block_id` - hash, number (height), or tag of the requested block
///
/// # Returns
///
/// * `FeeEstimate` - the fee estimation (gas consumed, gas price, overall fee, unit)
///
/// # Errors
///
/// BlockNotFound : If the specified block does not exist.
/// ContractNotFound : If the specified contract address does not exist.
/// ContractError : If there is an error with the contract.
pub async fn estimate_message_fee<A, BE, G, C, P, H>(
    starknet: &Starknet<A, BE, G, C, P, H>,
    message: MsgFromL1,
    block_id: BlockId,
) -> RpcResult<FeeEstimate>
where
    A: ChainApi<Block = DBlockT> + 'static,
    P: TransactionPool<Block = DBlockT> + 'static,
    BE: Backend<DBlockT> + 'static,
    C: HeaderBackend<DBlockT> + BlockBackend<DBlockT> + StorageProvider<DBlockT, BE> + 'static,
    C: ProvideRuntimeApi<DBlockT>,
    C::Api: StarknetRuntimeApi<DBlockT> + ConvertTransactionRuntimeApi<DBlockT>,
    G: GenesisProvider + Send + Sync + 'static,
    H: HasherT + Send + Sync + 'static,
{
    let substrate_block_hash = starknet.substrate_block_hash_from_starknet_block(block_id).map_err(|e| {
        log::error!("'{e}'");
        StarknetRpcApiError::BlockNotFound
    })?;

    // TODO: correct this with the correct conversion
    // let message = message.try_into().map_err(|e| {
    //     log::error!("Failed to convert MsgFromL1 to UserTransaction: {e}");
    //     StarknetRpcApiError::InternalServerError
    // })?;

    // let fee_estimate = starknet
    //     .client
    //     .runtime_api()
    //     .estimate_message_fee(substrate_block_hash, message)
    //     .map_err(|e| {
    //         log::error!("Runtime api error: {e}");
    //         StarknetRpcApiError::InternalServerError
    //     })?
    //     .map_err(|e| {
    //         log::error!("function execution failed: {:#?}", e);
    //         StarknetRpcApiError::ContractError
    //     })?;

    // TODO: Check if fee estimation is correct (spoiler alert it is not)
    let estimate = FeeEstimate {
        gas_price: FieldElement::ZERO,
        data_gas_consumed: FieldElement::ZERO,
        data_gas_price: FieldElement::ZERO,
        gas_consumed: FieldElement::ZERO,
        overall_fee: FieldElement::ZERO,
        unit: PriceUnit::Fri,
    };

    Ok(estimate)
}

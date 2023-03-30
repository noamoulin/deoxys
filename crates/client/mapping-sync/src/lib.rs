// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0
// This file is part of Frontier.
//
// Copyright (c) 2020-2022 Parity Technologies (UK) Ltd.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

#![allow(clippy::too_many_arguments)]
#![deny(unused_crate_dependencies)]

mod worker;

use pallet_starknet::runtime_api::StarknetRuntimeApi;
// Substrate
use sc_client_api::backend::{Backend, StorageProvider};
use sp_api::{ApiExt, ProvideRuntimeApi};
use sp_blockchain::{Backend as _, HeaderBackend};
use sp_runtime::traits::{Block as BlockT, Header as HeaderT, Zero};
pub use worker::{MappingSyncWorker, SyncStrategy};

pub fn sync_block<Block: BlockT, C, BE>(
    client: &C,
    backend: &madara_db::Backend<Block>,
    header: &Block::Header,
) -> Result<(), String>
where
    C: HeaderBackend<Block> + StorageProvider<Block, BE>,
    BE: Backend<Block>,
{
    let substrate_block_hash = header.hash();
    let digest = header.digest();
    for log in digest.logs.iter() {
        // println!("---  DIGEST: {:?}", log);
    }
    // match fp_consensus::find_log(header.digest()) {
    //     Ok(log) => {
    //         let gen_from_hashes = |hashes: Hashes| -> fc_db::MappingCommitment<Block> {
    //             fc_db::MappingCommitment {
    //                 block_hash: substrate_block_hash,
    //                 ethereum_block_hash: hashes.block_hash,
    //                 ethereum_transaction_hashes: hashes.transaction_hashes,
    //             }
    //         };
    //         let gen_from_block = |block| -> fc_db::MappingCommitment<Block> {
    //             let hashes = Hashes::from_block(block);
    //             gen_from_hashes(hashes)
    //         };

    //         match log {
    //             Log::Pre(PreLog::Block(block)) => {
    //                 let mapping_commitment = gen_from_block(block);
    //                 backend.mapping().write_hashes(mapping_commitment)
    //             }
    //             Log::Post(post_log) => match post_log {
    //                 PostLog::Hashes(hashes) => {
    //                     let mapping_commitment = gen_from_hashes(hashes);
    //                     backend.mapping().write_hashes(mapping_commitment)
    //                 }
    //                 PostLog::Block(block) => {
    //                     let mapping_commitment = gen_from_block(block);
    //                     backend.mapping().write_hashes(mapping_commitment)
    //                 }
    //                 PostLog::BlockHash(expect_eth_block_hash) => {
    //                     let schema = fc_storage::onchain_storage_schema(client,
    // substrate_block_hash);                     let ethereum_block = overrides
    //                         .schemas
    //                         .get(&schema)
    //                         .unwrap_or(&overrides.fallback)
    //                         .current_block(substrate_block_hash);
    //                     match ethereum_block {
    //                         Some(block) => {
    //                             let got_eth_block_hash = block.header.hash();
    //                             if got_eth_block_hash != expect_eth_block_hash {
    //                                 Err(format!(
    //                                     "Ethereum block hash mismatch: frontier consensus digest
    // \                                      ({expect_eth_block_hash:?}), db state
    // ({got_eth_block_hash:?})"                                 ))
    //                             } else {
    //                                 let mapping_commitment = gen_from_block(block);
    //                                 backend.mapping().write_hashes(mapping_commitment)
    //                             }
    //                         }
    //                         None => backend.mapping().write_none(substrate_block_hash),
    //                     }
    //                 }
    //             },
    //         }
    //     }
    //     Err(FindLogError::NotFound) => backend.mapping().write_none(substrate_block_hash),
    //     Err(FindLogError::MultipleLogs) => Err("Multiple logs found".to_string()),
    // }
    Ok(())
}

pub fn sync_genesis_block<Block: BlockT, C>(
    client: &C,
    backend: &madara_db::Backend<Block>,
    header: &Block::Header,
) -> Result<(), String>
where
    C: ProvideRuntimeApi<Block>,
    C::Api: StarknetRuntimeApi<Block>,
{
    let substrate_block_hash = header.hash();

    if let Some(api_version) = client
        .runtime_api()
        .api_version::<dyn StarknetRuntimeApi<Block>>(substrate_block_hash)
        .map_err(|e| format!("{:?}", e))?
    {
        let block = client.runtime_api().current_block(substrate_block_hash).map_err(|e| format!("{:?}", e))?;
        let block_hash = block.header.hash();
        let mapping_commitment = madara_db::MappingCommitment::<Block> {
            block_hash: substrate_block_hash,
            starknet_block_hash: block_hash,
            starknet_transaction_hashes: Vec::new(),
        };
        backend.mapping().write_hashes(mapping_commitment)?;
    } else {
        backend.mapping().write_none(substrate_block_hash)?;
    };

    Ok(())
}

pub fn sync_one_block<Block: BlockT, C, BE>(
    client: &C,
    substrate_backend: &BE,
    madara_backend: &madara_db::Backend<Block>,
    sync_from: <Block::Header as HeaderT>::Number,
    strategy: SyncStrategy,
) -> Result<bool, String>
where
    C: ProvideRuntimeApi<Block>,
    C::Api: StarknetRuntimeApi<Block>,
    C: HeaderBackend<Block> + StorageProvider<Block, BE>,
    BE: Backend<Block>,
{
    let mut current_syncing_tips = madara_backend.meta().current_syncing_tips()?;

    if current_syncing_tips.is_empty() {
        let mut leaves = substrate_backend.blockchain().leaves().map_err(|e| format!("{:?}", e))?;
        if leaves.is_empty() {
            return Ok(false);
        }
        current_syncing_tips.append(&mut leaves);
    }

    let mut operating_header = None;
    while let Some(checking_tip) = current_syncing_tips.pop() {
        if let Some(checking_header) =
            fetch_header(substrate_backend.blockchain(), madara_backend, checking_tip, sync_from)?
        {
            operating_header = Some(checking_header);
            break;
        }
    }
    let operating_header = match operating_header {
        Some(operating_header) => operating_header,
        None => {
            madara_backend.meta().write_current_syncing_tips(current_syncing_tips)?;
            return Ok(false);
        }
    };

    if operating_header.number() == &Zero::zero() {
        sync_genesis_block(client, madara_backend, &operating_header)?;

        madara_backend.meta().write_current_syncing_tips(current_syncing_tips)?;
        Ok(true)
    } else {
        if SyncStrategy::Parachain == strategy && operating_header.number() > &client.info().best_number {
            return Ok(false);
        }
        sync_block(client, madara_backend, &operating_header)?;

        current_syncing_tips.push(*operating_header.parent_hash());
        madara_backend.meta().write_current_syncing_tips(current_syncing_tips)?;
        Ok(true)
    }
}

pub fn sync_blocks<Block: BlockT, C, BE>(
    client: &C,
    substrate_backend: &BE,
    madara_backend: &madara_db::Backend<Block>,
    limit: usize,
    sync_from: <Block::Header as HeaderT>::Number,
    strategy: SyncStrategy,
) -> Result<bool, String>
where
    C: ProvideRuntimeApi<Block>,
    C::Api: StarknetRuntimeApi<Block>,
    C: HeaderBackend<Block> + StorageProvider<Block, BE>,
    BE: Backend<Block>,
{
    let mut synced_any = false;

    for _ in 0..limit {
        synced_any = synced_any || sync_one_block(client, substrate_backend, madara_backend, sync_from, strategy)?;
    }

    Ok(synced_any)
}

pub fn fetch_header<Block: BlockT, BE>(
    substrate_backend: &BE,
    madara_backend: &madara_db::Backend<Block>,
    checking_tip: Block::Hash,
    sync_from: <Block::Header as HeaderT>::Number,
) -> Result<Option<Block::Header>, String>
where
    BE: HeaderBackend<Block>,
{
    if madara_backend.mapping().is_synced(&checking_tip)? {
        return Ok(None);
    }

    match substrate_backend.header(checking_tip) {
        Ok(Some(checking_header)) if checking_header.number() >= &sync_from => Ok(Some(checking_header)),
        Ok(Some(_)) => Ok(None),
        Ok(None) | Err(_) => Err("Header not found".to_string()),
    }
}
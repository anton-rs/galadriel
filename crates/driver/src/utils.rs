use crate::types::OutputAtBlockResponse;
use anyhow::Result;
use ethers::{
    providers::{Http, Provider},
    types::H256,
};
use std::sync::Arc;

/// Compares the untrusted output root against the trusted output root from the node at a given
/// block number.
///
/// # Arguments
/// * `node_provider` - The HTTP provider used to query the node for the trusted output root.
/// * `untrusted_root` - The untrusted output root.
/// * `block_number` - The block number to query the node for the trusted output root.
///
/// # Returns
/// * Some(bool) if the request was successful, None otherwise. The bool represents whether the
/// untrusted root matches the trusted root.
pub async fn compare_output_root(
    node_provider: Arc<Provider<Http>>,
    untrusted_root: &H256,
    block_number: u64,
) -> Result<(bool, OutputAtBlockResponse)> {
    let trusted_root: OutputAtBlockResponse = node_provider
        .request(
            "optimism_outputAtBlock",
            vec![format!("0x{:x}", block_number)],
        )
        .await?;

    Ok((untrusted_root == &trusted_root.output_root, trusted_root))
}

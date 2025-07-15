use crate::{DagBlock, StorageService};
use icn_common::{CommonError, Did};
use icn_mesh::JobAssignment;

/// Compute the average final price for jobs executed by the given executor.
/// Returns `None` if no assignments are found.
pub fn average_executor_price<S: StorageService<DagBlock>>(
    store: &S,
    executor: &Did,
) -> Result<Option<u64>, CommonError> {
    let mut total = 0u64;
    let mut count = 0u64;
    for block in store.list_blocks()? {
        if let Ok(assignment) = serde_json::from_slice::<JobAssignment>(&block.data) {
            if assignment.assigned_executor_did == *executor {
                total += assignment.final_price_mana;
                count += 1;
            }
        }
    }
    if count > 0 {
        Ok(Some(total / count))
    } else {
        Ok(None)
    }
}

#[cfg(feature = "async")]
use crate::AsyncStorageService;
#[cfg(feature = "async")]
/// Asynchronous variant of [`average_executor_price`].
pub async fn average_executor_price_async<S>(
    store: &mut S,
    executor: &Did,
) -> Result<Option<u64>, CommonError>
where
    S: AsyncStorageService<DagBlock> + ?Sized,
{
    let blocks = store.list_blocks().await?;
    let mut total = 0u64;
    let mut count = 0u64;
    for block in blocks {
        if let Ok(assign) = serde_json::from_slice::<JobAssignment>(&block.data) {
            if assign.assigned_executor_did == *executor {
                total += assign.final_price_mana;
                count += 1;
            }
        }
    }
    if count > 0 {
        Ok(Some(total / count))
    } else {
        Ok(None)
    }
}

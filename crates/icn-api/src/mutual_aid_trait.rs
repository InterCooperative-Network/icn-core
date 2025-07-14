use async_trait::async_trait;
use icn_common::CommonError;
use icn_dag::mutual_aid::AidResource;

#[async_trait]
pub trait MutualAidApi {
    async fn list_resources(&self) -> Result<Vec<AidResource>, CommonError>;
    async fn register_resource(&self, resource: AidResource) -> Result<(), CommonError>;
}

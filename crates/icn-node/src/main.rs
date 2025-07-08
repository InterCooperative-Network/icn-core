use icn_node::node::run_node;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_node().await
}

use icn_common::*;

fn main() {
    let modified_block1_data = format!("modified data for {}", "block1_service_test").into_bytes();
    let timestamp = 1u64;
    let author = Did::new("key", "tester");
    let sig = None;
    let scope = None;
    let modified_cid = compute_merkle_cid(
        0x71,
        &modified_block1_data,
        &[],
        timestamp,
        &author,
        &sig,
        &scope,
    );
    
    let modified_block1 = DagBlock {
        cid: modified_cid.clone(),
        data: modified_block1_data.clone(),
        links: vec![],
        timestamp,
        author_did: author,
        signature: sig,
        scope,
    };
    
    println!("Block CID: {}", modified_block1.cid.to_string());
    
    // Now verify
    match verify_block_integrity(&modified_block1) {
        Ok(()) => println!("Block integrity verified successfully"),
        Err(e) => println!("Block integrity verification failed: {}", e),
    }
}

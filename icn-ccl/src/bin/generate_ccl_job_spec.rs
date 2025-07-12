use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use icn_common::parse_cid_from_string;
use serde::Serialize;

#[derive(Serialize)]
struct DagBlockPayload {
    data: Vec<u8>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <wasm-file> <node-api-url>", args[0]);
        std::process::exit(1);
    }
    let wasm_path = PathBuf::from(&args[1]);
    let api_url = args[2].trim_end_matches('/');

    let wasm_bytes = std::fs::read(&wasm_path)?;
    let payload = DagBlockPayload { data: wasm_bytes };
    let client = reqwest::blocking::Client::new();
    let resp = client
        .post(format!("{api_url}/dag/put"))
        .json(&payload)
        .send()?;

    if !resp.status().is_success() {
        return Err(format!("dag/put failed: {}", resp.status()).into());
    }

    let cid_str: String = resp.json()?;
    let _cid = parse_cid_from_string(&cid_str).map_err(|e| format!("Invalid CID returned: {e}"))?;

    let spec = serde_json::json!({
        "manifest_cid": cid_str,
        "spec_json": {
            "kind": "CclWasm",
            "inputs": [],
            "outputs": [],
            "required_resources": {"cpu_cores": 1, "memory_mb": 64}
        },
        "cost_mana": 0
    });

    let mut file = File::create("ccl_job_spec.json")?;
    file.write_all(serde_json::to_string_pretty(&spec)?.as_bytes())?;
    println!("Generated ccl_job_spec.json");
    Ok(())
}

# Creating and Running a CCL Contract

This quick guide walks through writing a simple Cooperative Contract Language (CCL) file, compiling it with `icn-cli`, uploading the compiled module to a node, and submitting a job that executes the contract. The examples under [`icn-ccl/examples/governance_templates/`](../icn-ccl/examples/governance_templates/) provide additional templates you can adapt.

## 1. Write the contract

Create a file `policy.ccl` containing your policy logic:

```ccl
// simple example
fn run() -> Integer {
    return 42;
}
```

Browse the [examples directory](../icn-ccl/examples/governance_templates/) for more realistic contracts such as `cooperative_housing_maintenance.ccl` or `childcare_cooperative_schedule.ccl`.

## 2. Compile the contract

Use `icn-cli` to compile the CCL source to WebAssembly. This command emits a `.wasm` file and metadata used by the node:

```bash
icn-cli ccl compile policy.ccl
```

## 3. Upload the module to a node

Send the compiled module to a running ICN node to obtain its content identifier (CID):

```bash
icn-cli --api-url http://localhost:7845 upload-ccl policy.wasm
```

The command prints a CID that references the stored module.

## 4. Submit a mesh job

Use the returned CID when creating a job. The job will execute the contract's `run` function:

```bash
icn-cli --api-url http://localhost:7845 submit-job \
  '{"manifest_cid":"<CID>","spec_bytes":"BASE64_SPEC","cost_mana":0}'
```

Check the job status with:

```bash
icn-cli --api-url http://localhost:7845 job-status <JOB_ID>
```

When the job completes, fetch the execution receipt using the returned result CID. See the [Deployment Guide](deployment-guide.md) for information on running nodes and verifying DAG data.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub license: String,
    pub repository: Option<String>,
    pub dependencies: HashMap<String, String>,
    pub main_file: String,
    pub files: Vec<String>,
    pub keywords: Vec<String>,
    pub category: PackageCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PackageCategory {
    Governance,
    Economics,
    Identity,
    Utility,
    Template,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManifest {
    pub package: Package,
    pub build: Option<BuildConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    pub compile_flags: Vec<String>,
    pub optimization_level: String,
}

pub struct PackageManager {
    package_dir: PathBuf,
    global_registry: crate::registry::PackageRegistry,
}

impl PackageManager {
    pub fn new(package_dir: PathBuf) -> Self {
        Self {
            package_dir,
            global_registry: crate::registry::PackageRegistry::new(),
        }
    }
    
    /// Initialize a new CCL package in the current directory
    pub fn init_package(
        &self,
        name: String,
        author: String,
        description: String,
        category: PackageCategory,
    ) -> Result<(), String> {
        let manifest = PackageManifest {
            package: Package {
                name: name.clone(),
                version: "0.1.0".to_string(),
                description,
                author,
                license: "MIT".to_string(),
                repository: None,
                dependencies: HashMap::new(),
                main_file: format!("{}.ccl", name),
                files: vec![format!("{}.ccl", name)],
                keywords: vec![],
                category,
            },
            build: Some(BuildConfig {
                compile_flags: vec![],
                optimization_level: "basic".to_string(),
            }),
        };
        
        // Write package.toml
        let manifest_content = toml::to_string_pretty(&manifest)
            .map_err(|e| format!("Failed to serialize manifest: {}", e))?;
        
        std::fs::write("package.toml", manifest_content)
            .map_err(|e| format!("Failed to write package.toml: {}", e))?;
        
        // Create main CCL file
        let main_content = match manifest.package.category {
            PackageCategory::Governance => create_governance_template(&name),
            PackageCategory::Economics => create_economics_template(&name),
            PackageCategory::Identity => create_identity_template(&name),
            PackageCategory::Utility => create_utility_template(&name),
            PackageCategory::Template => create_basic_template(&name),
        };
        
        std::fs::write(&manifest.package.main_file, main_content)
            .map_err(|e| format!("Failed to write main file: {}", e))?;
        
        println!("✅ Initialized package: {}", name);
        Ok(())
    }
    
    /// Install a package from the registry
    pub async fn install_package(&mut self, package_name: &str, version: Option<&str>) -> Result<(), String> {
        let package_info = self.global_registry.search_package(package_name).await?
            .ok_or_else(|| format!("Package '{}' not found", package_name))?;
        
        let version = version.unwrap_or(&package_info.version);
        
        println!("📦 Installing {} v{}", package_name, version);
        
        // Download package
        let package_data = self.global_registry.download_package(package_name, version).await?;
        
        // Extract to packages directory
        let package_dir = self.package_dir.join(package_name);
        crate::installer::extract_package(&package_data, &package_dir)?;
        
        println!("✅ Installed {} v{}", package_name, version);
        Ok(())
    }
    
    /// Add a dependency to the current package
    pub fn add_dependency(&self, name: String, version: String) -> Result<(), String> {
        let mut manifest = self.load_manifest()?;
        manifest.package.dependencies.insert(name.clone(), version.clone());
        self.save_manifest(&manifest)?;
        
        println!("✅ Added dependency: {} v{}", name, version);
        Ok(())
    }
    
    /// List installed packages
    pub fn list_packages(&self) -> Result<Vec<Package>, String> {
        let mut packages = Vec::new();
        
        if !self.package_dir.exists() {
            return Ok(packages);
        }
        
        for entry in std::fs::read_dir(&self.package_dir)
            .map_err(|e| format!("Failed to read packages directory: {}", e))? {
            
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();
            
            if path.is_dir() {
                let manifest_path = path.join("package.toml");
                if manifest_path.exists() {
                    let manifest = self.load_manifest_from_path(&manifest_path)?;
                    packages.push(manifest.package);
                }
            }
        }
        
        Ok(packages)
    }
    
    /// Build the current package
    pub fn build_package(&self) -> Result<(), String> {
        let manifest = self.load_manifest()?;
        
        println!("🔨 Building package: {}", manifest.package.name);
        
        // Compile main file
        let source = std::fs::read_to_string(&manifest.package.main_file)
            .map_err(|e| format!("Failed to read main file: {}", e))?;
        
        match icn_ccl::compile_ccl_source_to_wasm(&source) {
            Ok((wasm_bytes, metadata)) => {
                // Write WASM output
                let output_file = format!("{}.wasm", manifest.package.name);
                std::fs::write(&output_file, wasm_bytes)
                    .map_err(|e| format!("Failed to write WASM: {}", e))?;
                
                println!("✅ Built {} -> {}", manifest.package.main_file, output_file);
                println!("📊 Metadata: CID = {}", metadata.cid);
            }
            Err(e) => {
                return Err(format!("Compilation failed: {:?}", e));
            }
        }
        
        Ok(())
    }
    
    fn load_manifest(&self) -> Result<PackageManifest, String> {
        self.load_manifest_from_path(&PathBuf::from("package.toml"))
    }
    
    fn load_manifest_from_path(&self, path: &Path) -> Result<PackageManifest, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read package.toml: {}", e))?;
        
        toml::from_str(&content)
            .map_err(|e| format!("Failed to parse package.toml: {}", e))
    }
    
    fn save_manifest(&self, manifest: &PackageManifest) -> Result<(), String> {
        let content = toml::to_string_pretty(manifest)
            .map_err(|e| format!("Failed to serialize manifest: {}", e))?;
        
        std::fs::write("package.toml", content)
            .map_err(|e| format!("Failed to write package.toml: {}", e))
    }
}

fn create_governance_template(name: &str) -> String {
    format!(r#"// {} - Governance Module
// Generated by ICN CCL Package Manager

fn create_proposal(title: String, description: String) -> String {{
    // Create a new governance proposal
    let proposal_id = create_proposal(title, description, "standard");
    return proposal_id;
}}

fn conduct_vote(proposal_id: String, voter: did:key:*, choice: String) -> Bool {{
    // Cast a vote on the proposal
    return vote_on_proposal(proposal_id, choice, voter);
}}

fn execute_if_passed(proposal_id: String) -> Bool {{
    // Execute proposal if it has passed
    return execute_proposal(proposal_id);
}}
"#, name)
}

fn create_economics_template(name: &str) -> String {
    format!(r#"// {} - Economics Module
// Generated by ICN CCL Package Manager

fn create_budget(name: String, total: Integer) -> String {{
    // Create a new budget allocation
    let categories = ["operations", "development", "community"];
    let allocations = [total * 50 / 100, total * 30 / 100, total * 20 / 100];
    return create_budget(name, total, "cooperative_token", categories, allocations);
}}

fn distribute_surplus(amount: Integer, members: Array<did:key:*>) -> Bool {{
    // Distribute surplus to members equally
    let shares = [];
    let i = 0;
    while i < array_len(members) {{
        array_push(shares, 1);
        i = i + 1;
    }}
    
    let amounts = distribute_dividends(amount, members, shares, "equal");
    return true;
}}
"#, name)
}

fn create_identity_template(name: &str) -> String {
    format!(r#"// {} - Identity Module
// Generated by ICN CCL Package Manager

fn verify_member(member: did:key:*) -> Bool {{
    // Verify if a DID is a valid cooperative member
    return has_role(member, "member");
}}

fn assign_member_role(member: did:key:*) -> Bool {{
    // Assign member role to a DID
    return assign_role(member, "member");
}}

fn check_permissions(member: did:key:*, action: String) -> Bool {{
    // Check if member has permission to perform action
    return check_permission(member, action);
}}
"#, name)
}

fn create_utility_template(name: &str) -> String {
    format!(r#"// {} - Utility Module
// Generated by ICN CCL Package Manager

fn calculate_percentage(amount: Integer, percentage: Integer) -> Integer {{
    return (amount * percentage) / 100;
}}

fn format_did(key: String) -> String {{
    return "did:key:" + key;
}}

fn current_timestamp() -> Integer {{
    // Return current timestamp (placeholder)
    return 1640995200; // 2022-01-01
}}
"#, name)
}

fn create_basic_template(name: &str) -> String {
    format!(r#"// {} - CCL Contract
// Generated by ICN CCL Package Manager

fn main() -> Integer {{
    // Main contract function
    return 42;
}}

fn hello_world() -> String {{
    return "Hello from {}!";
}}
"#, name, name)
}
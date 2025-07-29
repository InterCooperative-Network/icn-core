// icn-ccl/src/migration.rs
//! CCL Migration Tools
//!
//! This module provides tools for migrating CCL contracts between versions
//! and converting from other contract languages to CCL.

use crate::error::CclError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Represents a CCL version
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CclVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl CclVersion {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    pub fn parse(version_str: &str) -> Result<Self, CclError> {
        let parts: Vec<&str> = version_str.split('.').collect();
        if parts.len() != 3 {
            return Err(CclError::CliArgumentError(format!(
                "Invalid version format: {}. Expected x.y.z",
                version_str
            )));
        }

        let major = parts[0].parse().map_err(|_| {
            CclError::CliArgumentError(format!("Invalid major version: {}", parts[0]))
        })?;
        let minor = parts[1].parse().map_err(|_| {
            CclError::CliArgumentError(format!("Invalid minor version: {}", parts[1]))
        })?;
        let patch = parts[2].parse().map_err(|_| {
            CclError::CliArgumentError(format!("Invalid patch version: {}", parts[2]))
        })?;

        Ok(Self::new(major, minor, patch))
    }
}

impl std::fmt::Display for CclVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Current CCL version
pub const CURRENT_CCL_VERSION: CclVersion = CclVersion {
    major: 0,
    minor: 2,
    patch: 0,
};

/// Migration rule for transforming code between versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationRule {
    pub from_version: CclVersion,
    pub to_version: CclVersion,
    pub rule_type: MigrationRuleType,
    pub pattern: String,
    pub replacement: String,
    pub description: String,
}

/// Types of migration rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MigrationRuleType {
    /// Simple text replacement
    TextReplace,
    /// Regular expression replacement
    RegexReplace,
    /// Keyword replacement
    KeywordReplace,
    /// Function signature update
    FunctionSignature,
    /// Syntax structure change
    SyntaxChange,
}

/// Migration engine for CCL contracts
pub struct MigrationEngine {
    rules: Vec<MigrationRule>,
}

impl MigrationEngine {
    pub fn new() -> Self {
        Self {
            rules: Self::load_builtin_rules(),
        }
    }

    /// Load built-in migration rules
    fn load_builtin_rules() -> Vec<MigrationRule> {
        vec![
            // Example migration rule: v0.1.x -> v0.2.x
            MigrationRule {
                from_version: CclVersion::new(0, 1, 0),
                to_version: CclVersion::new(0, 2, 0),
                rule_type: MigrationRuleType::KeywordReplace,
                pattern: "rule".to_string(),
                replacement: "policy".to_string(),
                description: "Replace deprecated 'rule' keyword with 'policy'".to_string(),
            },
            MigrationRule {
                from_version: CclVersion::new(0, 1, 0),
                to_version: CclVersion::new(0, 2, 0),
                rule_type: MigrationRuleType::FunctionSignature,
                pattern: "charge\\s*\\(([^)]+)\\)".to_string(),
                replacement: "require_payment($1)".to_string(),
                description: "Update charge() function to require_payment()".to_string(),
            },
            MigrationRule {
                from_version: CclVersion::new(0, 1, 0),
                to_version: CclVersion::new(0, 2, 0),
                rule_type: MigrationRuleType::SyntaxChange,
                pattern: "when\\s+(.+)\\s+then\\s+(.+)".to_string(),
                replacement: "if $1 { $2 }".to_string(),
                description: "Convert when-then syntax to if-block syntax".to_string(),
            },
        ]
    }

    /// Add a custom migration rule
    pub fn add_rule(&mut self, rule: MigrationRule) {
        self.rules.push(rule);
    }

    /// Detect the CCL version of a contract
    pub fn detect_version(&self, content: &str) -> Result<CclVersion, CclError> {
        // Look for version comment at the top of the file
        if let Some(version_line) = content
            .lines()
            .find(|line| line.contains("// CCL Version:"))
        {
            if let Some(version_str) = version_line.split("// CCL Version:").nth(1) {
                return CclVersion::parse(version_str.trim());
            }
        }

        // Heuristic detection based on syntax patterns
        if content.contains("rule ") || content.contains("when ") || content.contains("charge(") {
            return Ok(CclVersion::new(0, 1, 0)); // Old syntax
        }

        if content.contains("policy ") || content.contains("require_payment(") {
            return Ok(CclVersion::new(0, 2, 0)); // Current syntax
        }

        // Default to current version if unclear
        Ok(CURRENT_CCL_VERSION.clone())
    }

    /// Migrate a contract from one version to another
    pub fn migrate(
        &self,
        content: &str,
        from_version: &CclVersion,
        to_version: &CclVersion,
    ) -> Result<String, CclError> {
        if from_version >= to_version {
            return Err(CclError::CliArgumentError(
                "Cannot migrate to an older or same version".to_string(),
            ));
        }

        let mut migrated_content = content.to_string();

        // Find all applicable rules for the migration path
        let mut applicable_rules: Vec<&MigrationRule> = self
            .rules
            .iter()
            .filter(|rule| {
                // Rule is applicable if it's part of the migration path
                rule.from_version >= *from_version && rule.to_version <= *to_version
            })
            .collect();

        // Sort rules by from_version to ensure proper sequential application
        applicable_rules.sort_by(|a, b| a.from_version.cmp(&b.from_version));

        // Apply migration rules in version order
        for rule in applicable_rules {
            migrated_content = self.apply_rule(&migrated_content, rule)?;
        }

        // Add version comment at the top
        let version_comment = format!("// CCL Version: {}\n", to_version);
        if !migrated_content.starts_with("// CCL Version:") {
            migrated_content = version_comment + &migrated_content;
        } else {
            // Replace existing version comment
            let lines: Vec<&str> = migrated_content.lines().collect();
            if let Some(first_line) = lines.first() {
                if first_line.starts_with("// CCL Version:") {
                    migrated_content = version_comment + &lines[1..].join("\n");
                }
            }
        }

        Ok(migrated_content)
    }

    /// Apply a single migration rule
    fn apply_rule(&self, content: &str, rule: &MigrationRule) -> Result<String, CclError> {
        use regex::Regex;

        match rule.rule_type {
            MigrationRuleType::TextReplace => Ok(content.replace(&rule.pattern, &rule.replacement)),
            MigrationRuleType::KeywordReplace => {
                // Use word boundaries to avoid partial replacements
                let pattern = format!(r"\b{}\b", regex::escape(&rule.pattern));
                let regex = Regex::new(&pattern)
                    .map_err(|e| CclError::ParsingError(format!("Invalid regex pattern: {}", e)))?;
                Ok(regex
                    .replace_all(content, rule.replacement.as_str())
                    .to_string())
            }
            MigrationRuleType::RegexReplace
            | MigrationRuleType::FunctionSignature
            | MigrationRuleType::SyntaxChange => {
                let regex = Regex::new(&rule.pattern)
                    .map_err(|e| CclError::ParsingError(format!("Invalid regex pattern: {}", e)))?;
                Ok(regex
                    .replace_all(content, rule.replacement.as_str())
                    .to_string())
            }
        }
    }

    /// Generate a migration report
    pub fn generate_migration_report(
        &self,
        content: &str,
        from_version: &CclVersion,
        to_version: &CclVersion,
    ) -> MigrationReport {
        let mut report = MigrationReport {
            from_version: from_version.clone(),
            to_version: to_version.clone(),
            applicable_rules: Vec::new(),
            warnings: Vec::new(),
            estimated_changes: 0,
        };

        for rule in &self.rules {
            if rule.from_version >= *from_version && rule.to_version <= *to_version {
                report.applicable_rules.push(rule.clone());

                // Estimate number of changes
                match rule.rule_type {
                    MigrationRuleType::TextReplace | MigrationRuleType::KeywordReplace => {
                        report.estimated_changes += content.matches(&rule.pattern).count();
                    }
                    _ => {
                        // For regex patterns, this is a rough estimate
                        report.estimated_changes += 1;
                    }
                }
            }
        }

        // Add warnings for potential issues
        if content.contains("unsafe") {
            report
                .warnings
                .push("Contract contains 'unsafe' code that may need manual review".to_string());
        }
        if content.contains("experimental") {
            report.warnings.push("Contract uses experimental features that may not be available in the target version".to_string());
        }

        report
    }
}

/// Migration report showing what changes will be made
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationReport {
    pub from_version: CclVersion,
    pub to_version: CclVersion,
    pub applicable_rules: Vec<MigrationRule>,
    pub warnings: Vec<String>,
    pub estimated_changes: usize,
}

impl MigrationReport {}

impl std::fmt::Display for MigrationReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "Migration Report: {} -> {}",
            self.from_version, self.to_version
        )?;
        writeln!(f, "Estimated changes: {}\n", self.estimated_changes)?;

        if !self.applicable_rules.is_empty() {
            writeln!(f, "Applicable migration rules:")?;
            for (i, rule) in self.applicable_rules.iter().enumerate() {
                writeln!(f, "{}. {}", i + 1, rule.description)?;
            }
            writeln!(f)?;
        }

        if !self.warnings.is_empty() {
            writeln!(f, "Warnings:")?;
            for warning in &self.warnings {
                writeln!(f, "⚠️  {}", warning)?;
            }
        }

        Ok(())
    }
}

/// Convert from Solidity to CCL (basic conversion)
pub fn convert_from_solidity(solidity_content: &str) -> Result<String, CclError> {
    let mut ccl_content = String::new();

    // Add version header
    ccl_content.push_str(&format!("// CCL Version: {}\n", CURRENT_CCL_VERSION));
    ccl_content.push_str("// Converted from Solidity\n\n");

    // Basic Solidity to CCL mappings
    let mut content = solidity_content.to_string();

    // Contract definition
    content = content.replace("pragma solidity", "// pragma solidity");

    // Data types
    content = content.replace("uint256", "u64");
    content = content.replace("uint", "u32");
    // Note: address, bool, string remain the same in CCL

    // Visibility modifiers
    content = content.replace("public", "pub");
    content = content.replace("private", "");
    content = content.replace("internal", "");
    content = content.replace("external", "pub");

    // Function definitions
    content = content.replace("function ", "fn ");
    content = content.replace(" returns (", " -> ");
    content = content.replace(")", "");

    // State variables
    content = content.replace("mapping(", "Map<");
    content = content.replace(" => ", ", ");

    ccl_content.push_str(&content);

    // Add warning comment
    ccl_content.push_str("\n\n// WARNING: This is an automated conversion from Solidity.\n");
    ccl_content
        .push_str("// Manual review and adjustment is required for correct functionality.\n");

    Ok(ccl_content)
}

/// Convert from JavaScript/TypeScript governance code to CCL
pub fn convert_from_javascript(js_content: &str) -> Result<String, CclError> {
    let mut ccl_content = String::new();

    // Add version header
    ccl_content.push_str(&format!("// CCL Version: {}\n", CURRENT_CCL_VERSION));
    ccl_content.push_str("// Converted from JavaScript/TypeScript\n\n");

    // Basic JS to CCL mappings
    let mut content = js_content.to_string();

    // Class to contract
    content = content.replace("class ", "contract ");
    content = content.replace("export class ", "contract ");

    // Functions
    content = content.replace("function ", "fn ");
    content = content.replace("async function ", "async fn ");

    // Types
    content = content.replace(": number", ": u32");
    // Note: string remains the same in CCL
    content = content.replace(": boolean", ": bool");

    ccl_content.push_str(&content);

    // Add warning comment
    ccl_content.push_str("\n\n// WARNING: This is an automated conversion from JavaScript.\n");
    ccl_content
        .push_str("// Manual review and adjustment is required for correct functionality.\n");

    Ok(ccl_content)
}

/// Migrate a CCL file
pub fn migrate_file(
    input_path: &Path,
    output_path: &Path,
    target_version: Option<CclVersion>,
) -> Result<(), CclError> {
    let content = fs::read_to_string(input_path)
        .map_err(|e| CclError::IoError(format!("Failed to read input file: {}", e)))?;

    let engine = MigrationEngine::new();
    let detected_version = engine.detect_version(&content)?;
    let target = target_version.unwrap_or(CURRENT_CCL_VERSION.clone());

    println!("Detected version: {}", detected_version);
    println!("Target version: {}", target);

    if detected_version >= target {
        println!("File is already at or newer than target version. No migration needed.");
        return Ok(());
    }

    // Generate migration report
    let report = engine.generate_migration_report(&content, &detected_version, &target);
    println!("{}", report);

    // Perform migration
    let migrated_content = engine.migrate(&content, &detected_version, &target)?;

    // Write output
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| CclError::IoError(format!("Failed to create output directory: {}", e)))?;
    }

    fs::write(output_path, migrated_content)
        .map_err(|e| CclError::IoError(format!("Failed to write output file: {}", e)))?;

    println!("✅ Migration completed: {}", output_path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing() {
        let version = CclVersion::parse("1.2.3").unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
    }

    #[test]
    fn test_version_detection() {
        let engine = MigrationEngine::new();

        let content_v1 = "rule test_rule when condition then action";
        let version = engine.detect_version(content_v1).unwrap();
        assert_eq!(version, CclVersion::new(0, 1, 0));

        let content_v2 = "policy test_policy { condition: true }";
        let version = engine.detect_version(content_v2).unwrap();
        assert_eq!(version, CclVersion::new(0, 2, 0));
    }

    #[test]
    fn test_simple_migration() {
        let engine = MigrationEngine::new();
        let content = "rule test_rule when true then log(\"test\")";

        let migrated = engine
            .migrate(
                content,
                &CclVersion::new(0, 1, 0),
                &CclVersion::new(0, 2, 0),
            )
            .unwrap();

        // Should replace 'rule' keyword with 'policy'
        assert!(migrated.contains("policy"));
        assert!(migrated.starts_with("// CCL Version: 0.2.0\npolicy"));

        // Should convert when-then to if-block syntax
        assert!(migrated.contains("if true { log(\"test\") }"));
        assert!(!migrated.contains("when true then"));
    }

    #[test]
    fn test_multi_version_migration() {
        let mut engine = MigrationEngine::new();

        // Add a rule for 0.2.0 -> 0.3.0 migration
        engine.add_rule(MigrationRule {
            from_version: CclVersion::new(0, 2, 0),
            to_version: CclVersion::new(0, 3, 0),
            rule_type: MigrationRuleType::KeywordReplace,
            pattern: "require_payment".to_string(),
            replacement: "charge_mana".to_string(),
            description: "Update require_payment() to charge_mana()".to_string(),
        });

        // Test content with both old (0.1.0) and intermediate (0.2.0) syntax
        let content = "rule test_rule when true then charge(100)";

        // Migrate from 0.1.0 to 0.3.0 (should apply rules from 0.1.0->0.2.0 AND 0.2.0->0.3.0)
        let migrated = engine
            .migrate(
                content,
                &CclVersion::new(0, 1, 0),
                &CclVersion::new(0, 3, 0),
            )
            .unwrap();

        // Should have applied both migration steps:
        // 1. rule -> policy (0.1.0 -> 0.2.0)
        // 2. charge() -> require_payment() (0.1.0 -> 0.2.0)
        // 3. require_payment -> charge_mana (0.2.0 -> 0.3.0)
        assert!(migrated.contains("policy"));
        assert!(!migrated.contains("rule test_rule"));
        assert!(migrated.contains("charge_mana"));
        assert!(!migrated.contains("charge("));
    }
}

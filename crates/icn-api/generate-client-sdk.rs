//! Simple TypeScript Client SDK Generator
//!
//! This is a standalone script to generate TypeScript client SDK files
//! without heavy dependencies that might cause compilation issues.

use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = "./client-sdk";
    println!("Generating TypeScript SDK files in: {}", output_dir);

    // Create directories
    let src_dir = format!("{}/src", output_dir);
    fs::create_dir_all(&src_dir)?;

    // Generate main TypeScript file
    let ts_content = include_str!("client-sdk-template.ts");
    fs::write(format!("{}/index.ts", src_dir), ts_content)?;

    // Generate package.json
    let package_json = r#"{
  "name": "@icn/client-sdk",
  "version": "0.1.0",
  "description": "TypeScript client SDK for InterCooperative Network (ICN) APIs",
  "main": "dist/index.js",
  "types": "dist/index.d.ts",
  "files": [
    "dist/**/*",
    "README.md"
  ],
  "scripts": {
    "build": "tsc",
    "dev": "tsc --watch",
    "test": "jest",
    "lint": "eslint src/**/*.ts",
    "format": "prettier --write src/**/*.ts"
  },
  "keywords": [
    "icn",
    "intercooperative",
    "network",
    "api",
    "client",
    "sdk",
    "governance",
    "mesh",
    "federation"
  ],
  "author": "ICN Core Contributors",
  "license": "Apache-2.0",
  "devDependencies": {
    "@types/jest": "^29.0.0",
    "@typescript-eslint/eslint-plugin": "^6.0.0",
    "@typescript-eslint/parser": "^6.0.0",
    "eslint": "^8.0.0",
    "jest": "^29.0.0",
    "prettier": "^3.0.0",
    "typescript": "^5.0.0"
  },
  "peerDependencies": {
    "typescript": ">=4.0.0"
  }
}"#;
    fs::write(format!("{}/package.json", output_dir), package_json)?;

    // Generate tsconfig.json
    let tsconfig = r#"{
  "compilerOptions": {
    "target": "ES2020",
    "module": "commonjs",
    "lib": ["ES2020", "DOM"],
    "declaration": true,
    "outDir": "./dist",
    "rootDir": "./src",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "moduleResolution": "node",
    "resolveJsonModule": true,
    "allowSyntheticDefaultImports": true
  },
  "include": ["src/**/*"],
  "exclude": ["node_modules", "dist", "**/*.test.ts"]
}"#;
    fs::write(format!("{}/tsconfig.json", output_dir), tsconfig)?;

    // Generate README.md
    let readme = include_str!("client-sdk-readme.md");
    fs::write(format!("{}/README.md", output_dir), readme)?;

    // Generate .gitignore
    let gitignore = r#"node_modules/
dist/
*.log
.DS_Store
*.tsbuildinfo
"#;
    fs::write(format!("{}/.gitignore", output_dir), gitignore)?;

    println!("✅ TypeScript SDK generated successfully!");
    println!("📁 Files created in: {}", output_dir);
    println!();
    println!("To use the generated SDK:");
    println!("1. cd {}", output_dir);
    println!("2. npm install");
    println!("3. npm run build");

    Ok(())
} 
#!/usr/bin/env node
/**
 * Simple Frontend-Backend Integration Validator
 * 
 * This script validates the structure and documentation of the ICN frontend-backend
 * integration without requiring backend services or complex builds.
 */

const fs = require('fs');
const path = require('path');

class StructureValidator {
  constructor() {
    this.rootPath = path.join(__dirname, '..');
    this.results = {
      passed: 0,
      failed: 0,
      failures: [],
    };
  }

  log(message, level = 'info') {
    const prefix = {
      info: '📋',
      success: '✅',
      error: '❌',
      warning: '⚠️',
    }[level] || '📋';
    
    console.log(`${prefix} ${message}`);
  }

  async runTest(name, testFn) {
    try {
      await testFn();
      this.log(`${name} - PASSED`, 'success');
      this.results.passed++;
    } catch (error) {
      this.log(`${name} - FAILED: ${error.message}`, 'error');
      this.results.failed++;
      this.results.failures.push(`${name}: ${error.message}`);
    }
  }

  checkFileExists(filePath, description) {
    if (!fs.existsSync(filePath)) {
      throw new Error(`${description} not found at ${filePath}`);
    }
  }

  checkDirectoryExists(dirPath, description) {
    if (!fs.existsSync(dirPath) || !fs.statSync(dirPath).isDirectory()) {
      throw new Error(`${description} directory not found at ${dirPath}`);
    }
  }

  async validateProjectStructure() {
    await this.runTest('Project Structure - Root Directory', () => {
      const justfile = path.join(this.rootPath, 'justfile');
      this.checkFileExists(justfile, 'Justfile');
      
      const packageJson = path.join(this.rootPath, 'package.json');
      this.checkFileExists(packageJson, 'Root package.json');
      
      const appsDir = path.join(this.rootPath, 'apps');
      this.checkDirectoryExists(appsDir, 'Apps directory');
      
      const packagesDir = path.join(this.rootPath, 'packages');
      this.checkDirectoryExists(packagesDir, 'Packages directory');
      
      const cratesDir = path.join(this.rootPath, 'crates');
      this.checkDirectoryExists(cratesDir, 'Crates directory');
    });

    await this.runTest('Project Structure - Frontend Apps', () => {
      const frontendApps = ['web-ui', 'wallet-ui', 'agoranet', 'explorer'];
      
      for (const app of frontendApps) {
        const appPath = path.join(this.rootPath, 'apps', app);
        this.checkDirectoryExists(appPath, `${app} app`);
        
        const packageJson = path.join(appPath, 'package.json');
        this.checkFileExists(packageJson, `${app} package.json`);
      }
    });

    await this.runTest('Project Structure - TypeScript SDK', () => {
      const sdkPath = path.join(this.rootPath, 'packages', 'ts-sdk');
      this.checkDirectoryExists(sdkPath, 'TypeScript SDK');
      
      const packageJson = path.join(sdkPath, 'package.json');
      this.checkFileExists(packageJson, 'TypeScript SDK package.json');
      
      const srcDir = path.join(sdkPath, 'src');
      this.checkDirectoryExists(srcDir, 'TypeScript SDK src');
      
      const indexFile = path.join(srcDir, 'index.ts');
      this.checkFileExists(indexFile, 'TypeScript SDK index.ts');
    });

    await this.runTest('Project Structure - Backend API', () => {
      const apiCrate = path.join(this.rootPath, 'crates', 'icn-api');
      this.checkDirectoryExists(apiCrate, 'ICN API crate');
      
      const clientSdk = path.join(apiCrate, 'client-sdk');
      this.checkDirectoryExists(clientSdk, 'Client SDK crate');
      
      const nodeCrate = path.join(this.rootPath, 'crates', 'icn-node');
      this.checkDirectoryExists(nodeCrate, 'ICN Node crate');
    });
  }

  async validateTypeScriptSDK() {
    await this.runTest('TypeScript SDK - Core Files', () => {
      const sdkSrc = path.join(this.rootPath, 'packages', 'ts-sdk', 'src');
      
      const coreFiles = [
        'index.ts',
        'client.ts', 
        'types.ts',
        'storage.ts',
        'errors.ts',
        'utils.ts',
        'tests.ts',
        'integration.test.ts'
      ];
      
      for (const file of coreFiles) {
        const filePath = path.join(sdkSrc, file);
        this.checkFileExists(filePath, `TypeScript SDK ${file}`);
      }
    });

    await this.runTest('TypeScript SDK - Package Configuration', () => {
      const packageJsonPath = path.join(this.rootPath, 'packages', 'ts-sdk', 'package.json');
      const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'));
      
      if (!packageJson.scripts.build) {
        throw new Error('TypeScript SDK missing build script');
      }
      
      if (!packageJson.scripts.test) {
        throw new Error('TypeScript SDK missing test script');
      }
      
      if (!packageJson.scripts['test:integration']) {
        throw new Error('TypeScript SDK missing integration test script');
      }
      
      if (!packageJson.dependencies['@icn/client-sdk']) {
        throw new Error('TypeScript SDK missing client-sdk dependency');
      }
    });

    await this.runTest('TypeScript SDK - Integration Test Content', () => {
      const integrationTestPath = path.join(this.rootPath, 'packages', 'ts-sdk', 'src', 'integration.test.ts');
      const content = fs.readFileSync(integrationTestPath, 'utf8');
      
      const requiredSections = [
        'System and Connection Tests',
        'Job Management',
        'Identity',
        'Governance',
        'Federation',
        'Account',
      ];
      
      for (const section of requiredSections) {
        if (!content.includes(section)) {
          throw new Error(`Integration test missing ${section} tests`);
        }
      }
      
      if (!content.includes('runIntegrationTests')) {
        throw new Error('Integration test missing main export function');
      }
    });
  }

  async validateFrontendApps() {
    const apps = [
      { name: 'web-ui', description: 'Web UI Dashboard' },
      { name: 'wallet-ui', description: 'Wallet UI' },
      { name: 'agoranet', description: 'Agoranet App' },
      { name: 'explorer', description: 'Explorer App' },
    ];

    for (const app of apps) {
      await this.runTest(`Frontend App - ${app.description}`, () => {
        const appPath = path.join(this.rootPath, 'apps', app.name);
        
        const packageJsonPath = path.join(appPath, 'package.json');
        const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'));
        
        if (!packageJson.dependencies['@icn/ts-sdk']) {
          throw new Error(`${app.name} missing @icn/ts-sdk dependency`);
        }
        
        if (!packageJson.scripts.dev) {
          throw new Error(`${app.name} missing dev script`);
        }
        
        if (!packageJson.scripts.build) {
          throw new Error(`${app.name} missing build script`);
        }
        
        // Check for main app files
        const srcPath = path.join(appPath, 'src');
        if (fs.existsSync(srcPath)) {
          // Check for typical app entry points
          const appFiles = fs.readdirSync(srcPath, { recursive: true })
            .filter(file => typeof file === 'string')
            .map(file => file.toLowerCase());
          
          const hasAppFile = appFiles.some(file => 
            file.includes('app.') || 
            file.includes('index.') || 
            file.includes('main.') ||
            file.includes('_layout.') ||
            file.includes('app/index.')
          );
          
          if (!hasAppFile) {
            throw new Error(`${app.name} missing main app file in src/`);
          }
        }
      });
    }
  }

  async validateDocumentation() {
    await this.runTest('Documentation - Integration Guide', () => {
      const docPath = path.join(this.rootPath, 'docs', 'FRONTEND_BACKEND_INTEGRATION.md');
      this.checkFileExists(docPath, 'Frontend-Backend Integration documentation');
      
      const content = fs.readFileSync(docPath, 'utf8');
      
      const requiredSections = [
        '## Overview',
        '## API Integration Points',
        '## Configuration',
        '## Testing Integration',
        '## Troubleshooting',
      ];
      
      for (const section of requiredSections) {
        if (!content.includes(section)) {
          throw new Error(`Documentation missing ${section} section`);
        }
      }
      
      if (!content.includes('typescript')) {
        throw new Error('Documentation missing TypeScript examples');
      }
    });

    await this.runTest('Documentation - README Files', () => {
      const readmeFiles = [
        'README.md',
        'packages/ts-sdk/README.md',
      ];
      
      for (const readme of readmeFiles) {
        const readmePath = path.join(this.rootPath, readme);
        this.checkFileExists(readmePath, readme);
      }
    });
  }

  async validateJustfile() {
    await this.runTest('Justfile - Integration Commands', () => {
      const justfilePath = path.join(this.rootPath, 'justfile');
      const content = fs.readFileSync(justfilePath, 'utf8');
      
      const requiredCommands = [
        'validate-integration',
        'validate-integration-offline',
        'test-sdk-integration',
        'dev-frontend',
        'build-frontend',
      ];
      
      for (const command of requiredCommands) {
        if (!content.includes(command + ':')) {
          throw new Error(`Justfile missing ${command} command`);
        }
      }
    });
  }

  async validateTestingInfrastructure() {
    await this.runTest('Testing Infrastructure - Validation Script', () => {
      const scriptPath = path.join(this.rootPath, 'scripts', 'validate-frontend-integration.js');
      this.checkFileExists(scriptPath, 'Frontend validation script');
      
      const content = fs.readFileSync(scriptPath, 'utf8');
      
      if (!content.includes('FrontendValidator')) {
        throw new Error('Validation script missing FrontendValidator class');
      }
      
      if (!content.includes('validateBackendAvailability')) {
        throw new Error('Validation script missing backend availability check');
      }
    });

    await this.runTest('Testing Infrastructure - SDK Tests', () => {
      const sdkPath = path.join(this.rootPath, 'packages', 'ts-sdk', 'src');
      
      const testFiles = [
        'tests.ts',
        'storage.test.ts',
        'integration.test.ts',
      ];
      
      for (const testFile of testFiles) {
        const testPath = path.join(sdkPath, testFile);
        this.checkFileExists(testPath, `SDK ${testFile}`);
      }
    });
  }

  printSummary() {
    const total = this.results.passed + this.results.failed;
    console.log('\n📊 Frontend-Backend Integration Structure Validation:');
    console.log(`   ✅ Passed: ${this.results.passed}`);
    console.log(`   ❌ Failed: ${this.results.failed}`);
    console.log(`   📈 Success Rate: ${total > 0 ? ((this.results.passed / total) * 100).toFixed(1) : 0}%`);

    if (this.results.failures.length > 0) {
      console.log('\n❌ Failures:');
      this.results.failures.forEach((failure, index) => {
        console.log(`   ${index + 1}. ${failure}`);
      });
    }

    if (this.results.failed === 0) {
      console.log('\n🎉 All structure validation tests passed!');
      console.log('The ICN frontend-backend integration infrastructure is properly set up.');
    } else {
      console.log('\n⚠️  Some structure tests failed. Please review the failures above.');
    }

    return this.results.failed === 0;
  }

  async run() {
    this.log('🔍 ICN Frontend-Backend Integration Structure Validation');
    this.log(`📁 Root path: ${this.rootPath}`);
    
    try {
      await this.validateProjectStructure();
      await this.validateTypeScriptSDK();
      await this.validateFrontendApps();
      await this.validateDocumentation();
      await this.validateJustfile();
      await this.validateTestingInfrastructure();
      
    } catch (error) {
      this.log(`Validation failed with error: ${error.message}`, 'error');
      this.results.failed++;
      this.results.failures.push(`Validation error: ${error.message}`);
    }

    return this.printSummary();
  }
}

// CLI handling
async function main() {
  const args = process.argv.slice(2);
  
  if (args.includes('--help') || args.includes('-h')) {
    console.log(`
ICN Frontend-Backend Integration Structure Validator

This script validates that the frontend-backend integration infrastructure
is properly set up, including:
- Project structure and organization
- TypeScript SDK configuration
- Frontend application setup
- Documentation completeness
- Testing infrastructure

Usage: node validate-structure.js [options]

Options:
  --help, -h    Show this help message

This validation runs offline and doesn't require a running backend.
`);
    process.exit(0);
  }

  const validator = new StructureValidator();
  const success = await validator.run();
  
  process.exit(success ? 0 : 1);
}

if (require.main === module) {
  main().catch(error => {
    console.error('❌ Structure validation failed:', error);
    process.exit(1);
  });
}

module.exports = { StructureValidator };
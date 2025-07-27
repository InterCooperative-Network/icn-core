#!/usr/bin/env node
/**
 * Frontend App Validation Script
 * 
 * This script validates that frontend applications can properly connect to
 * and interact with ICN backend APIs. It performs automated tests of key
 * user flows to ensure frontend-backend integration is working correctly.
 */

const { execSync, spawn } = require('child_process');
const fs = require('fs');
const path = require('path');
const http = require('http');

// Configuration
const CONFIG = {
  nodeEndpoint: process.env.ICN_TEST_ENDPOINT || 'http://localhost:8080',
  nodePort: 8080,
  webUIPort: 3000,
  timeout: 30000,
  retryAttempts: 3,
  retryDelay: 2000,
};

class FrontendValidator {
  constructor() {
    this.results = {
      passed: 0,
      failed: 0,
      failures: [],
    };
  }

  log(message, level = 'info') {
    const timestamp = new Date().toISOString();
    const prefix = {
      info: '📋',
      success: '✅',
      error: '❌',
      warning: '⚠️',
      debug: '🔍',
    }[level] || '📋';
    
    console.log(`${prefix} [${timestamp}] ${message}`);
  }

  async sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  async checkPortAvailable(port) {
    return new Promise((resolve) => {
      const server = http.createServer();
      server.listen(port, () => {
        server.close();
        resolve(true);
      });
      server.on('error', () => {
        resolve(false);
      });
    });
  }

  async waitForService(url, maxAttempts = 10, delay = 2000) {
    for (let i = 0; i < maxAttempts; i++) {
      try {
        const response = await fetch(url);
        if (response.ok) {
          return true;
        }
      } catch (error) {
        this.log(`Waiting for service at ${url} (attempt ${i + 1}/${maxAttempts})`, 'debug');
      }
      await this.sleep(delay);
    }
    return false;
  }

  async runTest(name, testFn) {
    try {
      this.log(`Running test: ${name}`, 'info');
      await testFn();
      this.log(`Test passed: ${name}`, 'success');
      this.results.passed++;
    } catch (error) {
      this.log(`Test failed: ${name} - ${error.message}`, 'error');
      this.results.failed++;
      this.results.failures.push(`${name}: ${error.message}`);
    }
  }

  async validateBackendAvailability() {
    await this.runTest('Backend Node Availability', async () => {
      const healthUrl = `${CONFIG.nodeEndpoint}/health`;
      
      try {
        const response = await fetch(healthUrl, {
          method: 'GET',
          timeout: CONFIG.timeout,
        });
        
        if (!response.ok) {
          throw new Error(`Health check failed: ${response.status} ${response.statusText}`);
        }
        
        this.log(`Backend node is available at ${CONFIG.nodeEndpoint}`, 'success');
      } catch (error) {
        throw new Error(`Backend node not available at ${CONFIG.nodeEndpoint}: ${error.message}`);
      }
    });
  }

  async validateSystemAPIs() {
    await this.runTest('System Info API', async () => {
      const response = await fetch(`${CONFIG.nodeEndpoint}/system/info`);
      if (!response.ok) {
        throw new Error(`System info API failed: ${response.status}`);
      }
      
      const data = await response.json();
      if (!data.version || !data.name) {
        throw new Error('System info response missing required fields');
      }
      
      this.log(`System info: ${data.name} v${data.version}`, 'info');
    });

    await this.runTest('System Status API', async () => {
      const response = await fetch(`${CONFIG.nodeEndpoint}/system/status`);
      if (!response.ok) {
        throw new Error(`System status API failed: ${response.status}`);
      }
      
      const data = await response.json();
      if (typeof data.is_online !== 'boolean') {
        throw new Error('System status response missing required fields');
      }
      
      this.log(`Node online: ${data.is_online}, Peers: ${data.peer_count}`, 'info');
    });
  }

  async validateJobAPIs() {
    await this.runTest('Job Submission API', async () => {
      const jobSpec = {
        name: 'validation-test-job',
        image: 'hello-world',
        command: ['echo', 'Integration test job'],
        resources: {
          cpu: '100m',
          memory: '128Mi',
        },
      };

      const response = await fetch(`${CONFIG.nodeEndpoint}/mesh/jobs`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          job_spec: jobSpec,
          submitter_did: 'did:key:test_validator',
          max_cost: 100,
        }),
      });

      if (!response.ok) {
        throw new Error(`Job submission failed: ${response.status}`);
      }

      const data = await response.json();
      if (!data.job_id) {
        throw new Error('Job submission response missing job_id');
      }

      this.log(`Job submitted: ${data.job_id}`, 'info');
    });

    await this.runTest('Job List API', async () => {
      const response = await fetch(`${CONFIG.nodeEndpoint}/mesh/jobs`);
      if (!response.ok) {
        throw new Error(`Job list API failed: ${response.status}`);
      }
      
      const data = await response.json();
      if (!Array.isArray(data)) {
        throw new Error('Job list response should be an array');
      }
      
      this.log(`Found ${data.length} jobs`, 'info');
    });
  }

  async validateFederationAPIs() {
    await this.runTest('Federation Status API', async () => {
      const response = await fetch(`${CONFIG.nodeEndpoint}/federation/status`);
      if (!response.ok) {
        throw new Error(`Federation status API failed: ${response.status}`);
      }
      
      const data = await response.json();
      if (!data.name) {
        throw new Error('Federation status response missing name');
      }
      
      this.log(`Federation: ${data.name}`, 'info');
    });

    await this.runTest('Federation Peers API', async () => {
      const response = await fetch(`${CONFIG.nodeEndpoint}/federation/peers`);
      if (!response.ok) {
        throw new Error(`Federation peers API failed: ${response.status}`);
      }
      
      const data = await response.json();
      if (!Array.isArray(data)) {
        throw new Error('Federation peers response should be an array');
      }
      
      this.log(`Federation has ${data.length} peers`, 'info');
    });
  }

  async validateFrontendBuild() {
    await this.runTest('TypeScript SDK Build', async () => {
      const sdkPath = path.join(__dirname, '../../packages/ts-sdk');
      
      try {
        execSync('npm run build', {
          cwd: sdkPath,
          stdio: 'pipe',
          timeout: CONFIG.timeout,
        });
        
        // Check that build artifacts exist
        const distPath = path.join(sdkPath, 'dist');
        if (!fs.existsSync(distPath)) {
          throw new Error('TypeScript SDK build did not produce dist directory');
        }
        
        this.log('TypeScript SDK built successfully', 'success');
      } catch (error) {
        throw new Error(`TypeScript SDK build failed: ${error.message}`);
      }
    });

    await this.runTest('Web UI Build', async () => {
      const webUIPath = path.join(__dirname, '../../apps/web-ui');
      
      try {
        execSync('npm run build', {
          cwd: webUIPath,
          stdio: 'pipe',
          timeout: CONFIG.timeout * 2, // Frontend builds can take longer
        });
        
        // Check that build artifacts exist
        const distPath = path.join(webUIPath, 'dist');
        if (!fs.existsSync(distPath)) {
          throw new Error('Web UI build did not produce dist directory');
        }
        
        this.log('Web UI built successfully', 'success');
      } catch (error) {
        throw new Error(`Web UI build failed: ${error.message}`);
      }
    });
  }

  async validateTypescriptSDKIntegration() {
    await this.runTest('TypeScript SDK Integration Tests', async () => {
      const sdkPath = path.join(__dirname, '../../packages/ts-sdk');
      
      try {
        // Run the integration tests in offline mode
        execSync('npm run test:integration:offline', {
          cwd: sdkPath,
          stdio: 'pipe',
          timeout: CONFIG.timeout,
        });
        
        this.log('TypeScript SDK integration tests passed', 'success');
      } catch (error) {
        throw new Error(`TypeScript SDK integration tests failed: ${error.message}`);
      }
    });
  }

  async startFrontendApp(appName, port) {
    return new Promise((resolve, reject) => {
      const appPath = path.join(__dirname, `../../apps/${appName}`);
      
      if (!fs.existsSync(appPath)) {
        reject(new Error(`App directory not found: ${appPath}`));
        return;
      }

      const child = spawn('npm', ['run', 'dev'], {
        cwd: appPath,
        stdio: 'pipe',
        env: {
          ...process.env,
          VITE_ICN_NODE_ENDPOINT: CONFIG.nodeEndpoint,
          VITE_ICN_NETWORK: 'devnet',
          PORT: port.toString(),
        },
      });

      let output = '';
      child.stdout.on('data', (data) => {
        output += data.toString();
        if (output.includes('Local:') || output.includes(`localhost:${port}`)) {
          resolve(child);
        }
      });

      child.stderr.on('data', (data) => {
        const errorOutput = data.toString();
        if (errorOutput.includes('Error') || errorOutput.includes('EADDRINUSE')) {
          reject(new Error(`Failed to start ${appName}: ${errorOutput}`));
        }
      });

      child.on('exit', (code) => {
        if (code !== 0) {
          reject(new Error(`${appName} exited with code ${code}`));
        }
      });

      // Timeout if app doesn't start
      setTimeout(() => {
        reject(new Error(`${appName} failed to start within timeout`));
      }, CONFIG.timeout);
    });
  }

  async validateFrontendApps() {
    const apps = [
      { name: 'web-ui', port: 3000 },
      // Add other apps when ready
      // { name: 'wallet-ui', port: 3001 },
      // { name: 'agoranet', port: 3002 },
    ];

    for (const app of apps) {
      await this.runTest(`Frontend App: ${app.name}`, async () => {
        // Check if port is available
        const portAvailable = await this.checkPortAvailable(app.port);
        if (!portAvailable) {
          throw new Error(`Port ${app.port} is already in use`);
        }

        let childProcess;
        try {
          // Start the frontend app
          childProcess = await this.startFrontendApp(app.name, app.port);
          
          // Wait for the app to be available
          const appUrl = `http://localhost:${app.port}`;
          const isAvailable = await this.waitForService(appUrl, 10, 3000);
          
          if (!isAvailable) {
            throw new Error(`Frontend app ${app.name} not available at ${appUrl}`);
          }

          // Test that the app loads
          const response = await fetch(appUrl);
          if (!response.ok) {
            throw new Error(`Frontend app ${app.name} returned ${response.status}`);
          }

          const content = await response.text();
          if (!content.includes('ICN') && !content.includes('root')) {
            throw new Error(`Frontend app ${app.name} did not return expected content`);
          }

          this.log(`Frontend app ${app.name} is working at ${appUrl}`, 'success');
        } finally {
          // Clean up: kill the child process
          if (childProcess) {
            childProcess.kill('SIGTERM');
            await this.sleep(1000); // Give it time to shut down
          }
        }
      });
    }
  }

  async validateEnvironmentConfig() {
    await this.runTest('Environment Configuration', async () => {
      const webUIPath = path.join(__dirname, '../../apps/web-ui');
      const envFile = path.join(webUIPath, '.env.example');
      
      if (fs.existsSync(envFile)) {
        const envContent = fs.readFileSync(envFile, 'utf-8');
        if (!envContent.includes('VITE_ICN_NODE_ENDPOINT')) {
          throw new Error('.env.example missing VITE_ICN_NODE_ENDPOINT');
        }
      }
      
      // Check that the SDK can be imported
      const sdkPath = path.join(__dirname, '../../packages/ts-sdk/dist/index.js');
      if (!fs.existsSync(sdkPath)) {
        throw new Error('TypeScript SDK not built - run npm run build in packages/ts-sdk');
      }
      
      this.log('Environment configuration is valid', 'success');
    });
  }

  printSummary() {
    const total = this.results.passed + this.results.failed;
    console.log('\n📊 Frontend-Backend Integration Validation Results:');
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
      console.log('\n🎉 All frontend-backend integration tests passed!');
      console.log('The ICN frontend applications are properly integrated with the backend APIs.');
    } else {
      console.log('\n⚠️  Some tests failed. Please review the failures above.');
      console.log('Check the documentation in docs/FRONTEND_BACKEND_INTEGRATION.md for troubleshooting.');
    }
  }

  async run() {
    this.log('Starting ICN Frontend-Backend Integration Validation', 'info');
    this.log(`Backend endpoint: ${CONFIG.nodeEndpoint}`, 'info');
    
    try {
      // Validate environment and build
      await this.validateEnvironmentConfig();
      await this.validateFrontendBuild();
      
      // Validate backend APIs
      await this.validateBackendAvailability();
      await this.validateSystemAPIs();
      await this.validateJobAPIs();
      await this.validateFederationAPIs();
      
      // Validate TypeScript SDK integration
      await this.validateTypescriptSDKIntegration();
      
      // Validate frontend apps (optional - may be skipped if backend not running)
      if (this.results.failed === 0) {
        await this.validateFrontendApps();
      } else {
        this.log('Skipping frontend app tests due to backend API failures', 'warning');
      }
      
    } catch (error) {
      this.log(`Validation failed with error: ${error.message}`, 'error');
      this.results.failed++;
      this.results.failures.push(`Validation error: ${error.message}`);
    }

    this.printSummary();
    return this.results.failed === 0;
  }
}

// CLI handling
async function main() {
  const args = process.argv.slice(2);
  
  if (args.includes('--help') || args.includes('-h')) {
    console.log(`
ICN Frontend-Backend Integration Validator

Usage: node validate-frontend-integration.js [options]

Options:
  --endpoint <url>    Backend node endpoint (default: http://localhost:8080)
  --offline          Skip backend availability tests  
  --help, -h         Show this help message

Environment Variables:
  ICN_TEST_ENDPOINT   Backend node endpoint
  ICN_TEST_NETWORK    Network type (default: devnet)

Examples:
  node validate-frontend-integration.js
  node validate-frontend-integration.js --endpoint http://localhost:8080
  node validate-frontend-integration.js --offline
`);
    process.exit(0);
  }

  // Parse command line arguments
  const endpointIndex = args.indexOf('--endpoint');
  if (endpointIndex !== -1 && args[endpointIndex + 1]) {
    CONFIG.nodeEndpoint = args[endpointIndex + 1];
  }

  const validator = new FrontendValidator();
  const success = await validator.run();
  
  process.exit(success ? 0 : 1);
}

// Handle being called as module vs script
if (require.main === module) {
  main().catch(error => {
    console.error('❌ Validation script failed:', error);
    process.exit(1);
  });
}

module.exports = { FrontendValidator, CONFIG };
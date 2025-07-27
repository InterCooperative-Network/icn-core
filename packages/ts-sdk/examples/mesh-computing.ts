/**
 * Mesh Computing Example
 * 
 * This example demonstrates:
 * - Job submission with various resource requirements
 * - Job status monitoring and progress tracking
 * - Complex job workflows and dependencies
 * - Resource optimization and cost management
 * - Executor queue introspection
 */

import { 
  ICNClient, 
  createStorage, 
  ICNMeshError,
  ICNValidationError,
  ErrorUtils,
  EnhancedUtils
} from '@icn/ts-sdk';

async function meshComputingExample() {
  console.log('⚙️  Starting Mesh Computing Example\n');

  const client = new ICNClient({
    nodeEndpoint: 'http://localhost:8080',
    network: 'devnet',
    storage: createStorage('@mesh-example:'),
  });

  try {
    await client.connect();
    console.log('✅ Connected to ICN node\n');

    const submitterDid = 'did:key:developer123';
    const executorDid = 'did:key:executor456';

    // 1. Simple Computation Job
    console.log('🚀 Submitting simple computation job...');
    
    const simpleJob = await client.mesh.submitJob({
      job_spec: {
        image: 'node:18-alpine',
        command: ['node', '-e', 'console.log("Hello ICN Mesh! Result:", Math.PI * 2)'],
        resources: {
          cpu_cores: 1,
          memory_mb: 256,
          storage_mb: 100
        },
        environment: {
          NODE_ENV: 'production',
          LOG_LEVEL: 'info'
        }
      },
      submitter_did: submitterDid,
      max_cost: 50,
      timeout_seconds: 300
    });
    
    console.log('✅ Simple job submitted successfully');
    console.log(`   Job ID: ${simpleJob.job_id}`);
    console.log('   Command: Node.js calculation');
    console.log('   Resources: 1 CPU, 256MB RAM, 100MB storage');
    console.log('   Max Cost: 50 mana');
    console.log('   Timeout: 5 minutes\n');

    // 2. Data Processing Job
    console.log('📊 Submitting data processing job...');
    
    const dataProcessingJob = await client.mesh.submitJob({
      job_spec: {
        image: 'python:3.11-slim',
        command: [
          'python', '-c', 
          'import json; import time; ' +
          'data = [{"id": i, "value": i**2} for i in range(1000)]; ' +
          'time.sleep(2); ' +
          'result = sum(item["value"] for item in data); ' +
          'print(f"Processed {len(data)} items, sum: {result}")'
        ],
        resources: {
          cpu_cores: 2,
          memory_mb: 512,
          storage_mb: 200
        },
        environment: {
          PYTHONUNBUFFERED: '1',
          DATA_SIZE: '1000'
        }
      },
      submitter_did: submitterDid,
      max_cost: 150,
      timeout_seconds: 600
    });
    
    console.log('✅ Data processing job submitted successfully');
    console.log(`   Job ID: ${dataProcessingJob.job_id}`);
    console.log('   Command: Python data processing');
    console.log('   Resources: 2 CPU, 512MB RAM, 200MB storage');
    console.log('   Max Cost: 150 mana');
    console.log('   Timeout: 10 minutes\n');

    // 3. Long-running Service Job
    console.log('🔄 Submitting long-running service job...');
    
    const serviceJob = await client.mesh.submitJob({
      job_spec: {
        image: 'nginx:alpine',
        command: ['sh', '-c', 'echo "Starting service..."; nginx -g "daemon off;"'],
        resources: {
          cpu_cores: 1,
          memory_mb: 128,
          storage_mb: 50
        },
        environment: {
          NGINX_PORT: '8080',
          WORKER_PROCESSES: '1'
        }
      },
      submitter_did: submitterDid,
      max_cost: 500, // Higher cost for longer running job
      timeout_seconds: 3600 // 1 hour
    });
    
    console.log('✅ Service job submitted successfully');
    console.log(`   Job ID: ${serviceJob.job_id}`);
    console.log('   Command: Nginx web server');
    console.log('   Resources: 1 CPU, 128MB RAM, 50MB storage');
    console.log('   Max Cost: 500 mana');
    console.log('   Timeout: 1 hour\n');

    // 4. Resource-Intensive Job
    console.log('💪 Submitting resource-intensive job...');
    
    const intensiveJob = await client.mesh.submitJob({
      job_spec: {
        image: 'ubuntu:22.04',
        command: [
          'bash', '-c',
          'apt-get update && apt-get install -y stress && ' +
          'echo "Starting CPU stress test..."; ' +
          'stress --cpu 4 --timeout 30s && ' +
          'echo "CPU stress test completed successfully"'
        ],
        resources: {
          cpu_cores: 4,
          memory_mb: 1024,
          storage_mb: 500
        }
      },
      submitter_did: submitterDid,
      max_cost: 300,
      timeout_seconds: 900 // 15 minutes
    });
    
    console.log('✅ Resource-intensive job submitted successfully');
    console.log(`   Job ID: ${intensiveJob.job_id}`);
    console.log('   Command: CPU stress test');
    console.log('   Resources: 4 CPU, 1GB RAM, 500MB storage');
    console.log('   Max Cost: 300 mana');
    console.log('   Timeout: 15 minutes\n');

    // 5. List All Jobs
    console.log('📋 Listing all mesh jobs...');
    
    const allJobs = await client.mesh.listJobs();
    
    console.log(`⚙️  Found ${allJobs.length} total jobs:`);
    allJobs.slice(0, 8).forEach((job, index) => {
      const duration = EnhancedUtils.formatJobDuration(job.started_at, job.completed_at);
      console.log(`   ${index + 1}. ${EnhancedUtils.formatJobId(job.id)}`);
      console.log(`      Status: ${job.status}`);
      console.log(`      Submitter: ${job.submitter.slice(0, 20)}...`);
      console.log(`      Cost: ${job.cost} mana`);
      console.log(`      Duration: ${duration}`);
      if (job.executor) {
        console.log(`      Executor: ${job.executor.slice(0, 20)}...`);
      }
      if (job.progress !== undefined) {
        console.log(`      Progress: ${job.progress}%`);
      }
    });
    
    if (allJobs.length > 8) {
      console.log(`   ... and ${allJobs.length - 8} more jobs`);
    }
    console.log();

    // 6. Monitor Job Status with Polling
    console.log('👀 Monitoring job status...');
    
    const jobsToMonitor = [simpleJob.job_id, dataProcessingJob.job_id];
    
    for (const jobId of jobsToMonitor) {
      console.log(`\n🔍 Monitoring job ${EnhancedUtils.formatJobId(jobId)}:`);
      
      let attempts = 0;
      const maxAttempts = 5;
      
      while (attempts < maxAttempts) {
        try {
          const jobStatus = await client.mesh.getJobStatus(jobId);
          const duration = EnhancedUtils.formatJobDuration(jobStatus.started_at, jobStatus.completed_at);
          
          console.log(`   Attempt ${attempts + 1}:`);
          console.log(`     Status: ${jobStatus.status}`);
          console.log(`     Duration: ${duration}`);
          console.log(`     Cost: ${jobStatus.cost} mana`);
          
          if (jobStatus.executor) {
            console.log(`     Executor: ${jobStatus.executor.slice(0, 20)}...`);
          }
          
          if (jobStatus.progress !== undefined) {
            console.log(`     Progress: ${jobStatus.progress}%`);
          }
          
          if (jobStatus.result) {
            console.log(`     Exit Code: ${jobStatus.result.exit_code}`);
            if (jobStatus.result.output && jobStatus.result.output.length > 0) {
              const output = jobStatus.result.output.slice(0, 200);
              console.log(`     Output: ${output}${jobStatus.result.output.length > 200 ? '...' : ''}`);
            }
          }
          
          if (jobStatus.error) {
            console.log(`     Error: ${jobStatus.error}`);
          }
          
          // Break if job is completed or failed
          if (['Completed', 'Failed', 'Cancelled'].includes(jobStatus.status)) {
            break;
          }
          
          // Wait before next check
          await new Promise(resolve => setTimeout(resolve, 2000));
          attempts++;
          
        } catch (error) {
          console.log(`     Error checking status: ${ErrorUtils.getErrorMessage(error)}`);
          break;
        }
      }
      
      if (attempts >= maxAttempts) {
        console.log(`     ⚠️  Reached maximum monitoring attempts`);
      }
    }
    console.log();

    // 7. Executor Queue Introspection
    console.log('🔍 Checking executor queue information...');
    
    try {
      const queueInfo = await client.executor.getExecutorQueue(executorDid);
      
      console.log('📊 Executor Queue Status:');
      console.log(`   Queued Jobs: ${queueInfo.queued}`);
      console.log(`   Capacity: ${queueInfo.capacity}`);
      console.log(`   Utilization: ${((queueInfo.queued / queueInfo.capacity) * 100).toFixed(1)}%`);
      console.log(`   Available Slots: ${queueInfo.capacity - queueInfo.queued}`);
      
      if (queueInfo.queued === queueInfo.capacity) {
        console.log('   ⚠️  Queue is at full capacity');
      } else if (queueInfo.queued / queueInfo.capacity > 0.8) {
        console.log('   ⚠️  Queue is near capacity');
      } else {
        console.log('   ✅ Queue has available capacity');
      }
    } catch (error) {
      console.log('⚠️  Executor queue information may not be available');
      console.log(`   Error: ${ErrorUtils.getErrorMessage(error)}`);
    }
    console.log();

    // 8. Job Analytics and Cost Analysis
    console.log('📊 Job analytics and cost analysis...');
    
    // Analyze job statuses
    const jobsByStatus = allJobs.reduce((acc, job) => {
      acc[job.status] = (acc[job.status] || 0) + 1;
      return acc;
    }, {} as Record<string, number>);
    
    console.log('📈 Job Status Distribution:');
    Object.entries(jobsByStatus).forEach(([status, count]) => {
      const percentage = ((count / allJobs.length) * 100).toFixed(1);
      console.log(`   ${status}: ${count} jobs (${percentage}%)`);
    });
    
    // Cost analysis
    const totalCost = allJobs.reduce((sum, job) => sum + job.cost, 0);
    const avgCost = allJobs.length > 0 ? totalCost / allJobs.length : 0;
    const costByStatus = allJobs.reduce((acc, job) => {
      acc[job.status] = (acc[job.status] || 0) + job.cost;
      return acc;
    }, {} as Record<string, number>);
    
    console.log('\n💰 Cost Analysis:');
    console.log(`   Total Cost: ${totalCost} mana`);
    console.log(`   Average Cost: ${avgCost.toFixed(1)} mana per job`);
    console.log('   Cost by Status:');
    Object.entries(costByStatus).forEach(([status, cost]) => {
      console.log(`     ${status}: ${cost} mana`);
    });
    
    // Resource utilization analysis
    const totalCPU = allJobs.reduce((sum, job) => {
      // Try to extract CPU from job spec if available
      return sum + 1; // Default assumption
    }, 0);
    
    const totalMemory = allJobs.reduce((sum, job) => {
      // Try to extract memory from job spec if available
      return sum + 256; // Default assumption in MB
    }, 0);
    
    console.log('\n🔧 Resource Utilization Estimates:');
    console.log(`   Total CPU Cores Used: ${totalCPU}`);
    console.log(`   Total Memory Used: ${EnhancedUtils.formatBytes(totalMemory * 1024 * 1024)}`);
    console.log(`   Average CPU per Job: ${(totalCPU / allJobs.length).toFixed(1)} cores`);
    console.log(`   Average Memory per Job: ${(totalMemory / allJobs.length).toFixed(0)} MB`);
    console.log();

    // 9. Job Performance Metrics
    console.log('⚡ Job performance metrics...');
    
    const completedJobs = allJobs.filter(job => job.status === 'Completed' && job.started_at && job.completed_at);
    
    if (completedJobs.length > 0) {
      const executionTimes = completedJobs.map(job => {
        const start = new Date(job.started_at!).getTime();
        const end = new Date(job.completed_at!).getTime();
        return (end - start) / 1000; // seconds
      });
      
      const avgExecutionTime = executionTimes.reduce((sum, time) => sum + time, 0) / executionTimes.length;
      const minExecutionTime = Math.min(...executionTimes);
      const maxExecutionTime = Math.max(...executionTimes);
      
      console.log('⏱️  Execution Time Statistics:');
      console.log(`   Completed Jobs: ${completedJobs.length}`);
      console.log(`   Average Execution Time: ${EnhancedUtils.formatJobDuration('', new Date(Date.now() + avgExecutionTime * 1000).toISOString())}`);
      console.log(`   Fastest Job: ${EnhancedUtils.formatJobDuration('', new Date(Date.now() + minExecutionTime * 1000).toISOString())}`);
      console.log(`   Slowest Job: ${EnhancedUtils.formatJobDuration('', new Date(Date.now() + maxExecutionTime * 1000).toISOString())}`);
      
      // Efficiency analysis
      const efficiencyMetrics = completedJobs.map(job => {
        const start = new Date(job.started_at!).getTime();
        const end = new Date(job.completed_at!).getTime();
        const executionTime = (end - start) / 1000;
        const costEfficiency = job.cost / executionTime; // mana per second
        return { jobId: job.id, executionTime, cost: job.cost, efficiency: costEfficiency };
      });
      
      const avgEfficiency = efficiencyMetrics.reduce((sum, metric) => sum + metric.efficiency, 0) / efficiencyMetrics.length;
      
      console.log('\n💡 Efficiency Metrics:');
      console.log(`   Average Cost Efficiency: ${avgEfficiency.toFixed(3)} mana/second`);
      
      const topEfficient = efficiencyMetrics.sort((a, b) => a.efficiency - b.efficiency).slice(0, 3);
      console.log('   Most Efficient Jobs:');
      topEfficient.forEach((metric, index) => {
        console.log(`     ${index + 1}. ${EnhancedUtils.formatJobId(metric.jobId)}: ${metric.efficiency.toFixed(3)} mana/sec`);
      });
    } else {
      console.log('⚠️  No completed jobs available for performance analysis');
    }
    console.log();

    // 10. Resource Optimization Recommendations
    console.log('🎯 Resource optimization recommendations...');
    
    const pendingJobs = allJobs.filter(job => job.status === 'Pending');
    const runningJobs = allJobs.filter(job => job.status === 'Running');
    const failedJobs = allJobs.filter(job => job.status === 'Failed');
    
    console.log('💡 Optimization Insights:');
    
    if (pendingJobs.length > 5) {
      console.log('   ⚠️  High number of pending jobs detected');
      console.log('   💡 Consider: Adding more executors or increasing executor capacity');
    }
    
    if (failedJobs.length / allJobs.length > 0.1) {
      console.log('   ⚠️  High failure rate detected');
      console.log('   💡 Consider: Reviewing job specifications and resource requirements');
    }
    
    if (runningJobs.length > 10) {
      console.log('   ⚠️  Many concurrent jobs running');
      console.log('   💡 Consider: Monitor resource utilization and executor performance');
    }
    
    const highCostJobs = allJobs.filter(job => job.cost > 200);
    if (highCostJobs.length > 0) {
      console.log(`   💰 ${highCostJobs.length} high-cost jobs detected`);
      console.log('   💡 Consider: Optimizing resource requirements for cost efficiency');
    }
    
    console.log('\n📋 Best Practices:');
    console.log('   • Use appropriate resource allocations for job requirements');
    console.log('   • Set reasonable timeouts to prevent resource waste');
    console.log('   • Monitor job success rates and optimize failing patterns');
    console.log('   • Consider job dependencies and workflow optimization');
    console.log('   • Use executor queue information for load balancing');
    console.log('   • Implement retry logic for transient failures');
    console.log('   • Monitor cost efficiency and optimize expensive operations');

    console.log('\n🎉 Mesh Computing example completed successfully!');
    console.log('\n💡 Key Features Demonstrated:');
    console.log('   • Various job types (simple, data processing, services, intensive)');
    console.log('   • Comprehensive job monitoring and status tracking');
    console.log('   • Resource requirement specification and optimization');
    console.log('   • Cost analysis and efficiency metrics');
    console.log('   • Executor queue introspection and load monitoring');
    console.log('   • Performance analytics and optimization recommendations');
    
    console.log('\n⚙️  Mesh Computing Benefits:');
    console.log('   • Distributed computing power with resource flexibility');
    console.log('   • Cost-effective job execution with transparent pricing');
    console.log('   • Scalable infrastructure with automatic load balancing');
    console.log('   • Comprehensive monitoring and analytics');
    console.log('   • Fault tolerance and retry mechanisms');

  } catch (error) {
    console.error('❌ Error during mesh computing example:');
    
    if (ErrorUtils.isErrorType(error, ICNMeshError)) {
      console.error('⚙️  Mesh Error:', error.message);
      console.error('💡 Tip: Check job specifications and resource availability');
    } else if (ErrorUtils.isErrorType(error, ICNValidationError)) {
      console.error('📝 Validation Error:', error.message);
      if (error.field) {
        console.error(`   Field: ${error.field}`);
      }
    } else {
      console.error('🔍 Unexpected Error:', ErrorUtils.getErrorMessage(error));
    }
  } finally {
    await client.disconnect();
    console.log('\n🔌 Disconnected from ICN node');
  }
}

// Run the example
if (require.main === module) {
  meshComputingExample().catch(console.error);
}

export { meshComputingExample };
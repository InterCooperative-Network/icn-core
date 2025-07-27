/**
 * Mutual Aid Example
 * 
 * This example demonstrates:
 * - Resource registration and management
 * - Resource discovery and matching
 * - Mutual aid coordination workflows
 * - Community resource sharing
 * - Resource utilization analytics
 */

import { 
  ICNClient, 
  createStorage, 
  ICNValidationError,
  ErrorUtils,
  EnhancedUtils
} from '@icn/ts-sdk';

async function mutualAidExample() {
  console.log('🤝 Starting Mutual Aid Resource Sharing Example\n');

  const client = new ICNClient({
    nodeEndpoint: 'http://localhost:8080',
    network: 'devnet',
    storage: createStorage('@mutual-aid-example:'),
  });

  try {
    await client.connect();
    console.log('✅ Connected to ICN node\n');

    // Example participants
    const cooperativeA = 'did:key:coop-a-123';
    const cooperativeB = 'did:key:coop-b-456';
    const memberAlice = 'did:key:alice789';
    const memberBob = 'did:key:bob012';
    const memberCarol = 'did:key:carol345';

    // 1. Register Technical Skills Resources
    console.log('💻 Registering technical skills resources...');
    
    await client.mutualAid.registerResource({
      id: 'skill_web_dev_alice',
      name: 'Full-Stack Web Development',
      description: 'Experienced developer offering React, Node.js, TypeScript, and PostgreSQL development services. Available for mentoring, code reviews, and project development.',
      category: 'technical_skills',
      provider_did: memberAlice,
      availability: 'available',
      location: 'Remote / San Francisco Bay Area',
      contact_info: 'alice@cooperative-a.org',
      metadata: {
        skills: 'React,TypeScript,Node.js,PostgreSQL,Docker,Kubernetes',
        experience_years: '8',
        hourly_rate: 'volunteer',
        availability_hours: 'evenings_weekends',
        languages: 'English,Spanish',
        timezone: 'PST',
        mentoring_available: 'true',
        project_types: 'web_apps,apis,databases,devops'
      },
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString()
    });
    
    console.log('✅ Alice\'s web development skills registered');
    
    await client.mutualAid.registerResource({
      id: 'skill_data_science_bob',
      name: 'Data Science and Machine Learning',
      description: 'Data scientist specializing in machine learning, statistical analysis, and data visualization. Offering consultation, model development, and training services.',
      category: 'technical_skills',
      provider_did: memberBob,
      availability: 'available',
      location: 'Remote / Austin, TX',
      contact_info: 'bob@cooperative-b.org',
      metadata: {
        skills: 'Python,R,TensorFlow,PyTorch,Pandas,Jupyter,SQL',
        experience_years: '6',
        hourly_rate: 'volunteer',
        specializations: 'nlp,computer_vision,predictive_modeling',
        availability_hours: 'flexible',
        languages: 'English',
        timezone: 'CST',
        consultation_available: 'true',
        training_available: 'true'
      },
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString()
    });
    
    console.log('✅ Bob\'s data science skills registered\n');

    // 2. Register Equipment and Tools
    console.log('🔧 Registering equipment and tools...');
    
    await client.mutualAid.registerResource({
      id: 'equipment_3d_printer',
      name: 'Professional 3D Printer (Prusa i3 MK3S+)',
      description: 'High-quality 3D printer available for community projects. Includes various filaments (PLA, PETG, ABS) and post-processing tools.',
      category: 'equipment',
      provider_did: cooperativeA,
      availability: 'available',
      location: 'San Francisco, CA - Cooperative A Makerspace',
      contact_info: 'makerspace@cooperative-a.org',
      metadata: {
        equipment_type: '3d_printer',
        brand: 'Prusa',
        model: 'i3_MK3S+',
        build_volume: '250x210x210mm',
        materials: 'PLA,PETG,ABS,FLEX',
        hourly_rate: '$5',
        daily_rate: '$25',
        booking_required: 'true',
        training_required: 'true',
        safety_certification: 'required',
        availability_schedule: 'weekdays_9_17,weekends_10_16'
      },
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString()
    });
    
    console.log('✅ 3D printer registered');
    
    await client.mutualAid.registerResource({
      id: 'equipment_electronics_lab',
      name: 'Electronics Lab and Testing Equipment',
      description: 'Fully equipped electronics lab with oscilloscopes, signal generators, power supplies, and component inventory for prototyping and testing.',
      category: 'equipment',
      provider_did: cooperativeB,
      availability: 'available',
      location: 'Austin, TX - Cooperative B Lab',
      contact_info: 'lab@cooperative-b.org',
      metadata: {
        equipment_type: 'electronics_lab',
        equipment_list: 'oscilloscope,signal_generator,power_supply,multimeter,soldering_station',
        component_inventory: 'resistors,capacitors,ics,microcontrollers,sensors',
        hourly_rate: '$10',
        daily_rate: '$40',
        booking_required: 'true',
        supervision_available: 'true',
        training_provided: 'true',
        safety_training_required: 'true'
      },
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString()
    });
    
    console.log('✅ Electronics lab registered\n');

    // 3. Register Educational Resources
    console.log('📚 Registering educational resources...');
    
    await client.mutualAid.registerResource({
      id: 'education_blockchain_workshop',
      name: 'Blockchain and Cooperative Economics Workshop',
      description: 'Comprehensive workshop covering blockchain technology, cryptocurrency, and cooperative economic principles. Includes hands-on exercises and case studies.',
      category: 'education',
      provider_did: memberCarol,
      availability: 'limited',
      location: 'Online / Regional Cooperative Centers',
      contact_info: 'carol@cooperative-network.org',
      metadata: {
        education_type: 'workshop',
        duration: '2_days',
        format: 'hybrid_online_offline',
        max_participants: '25',
        prerequisites: 'basic_economics,computer_literacy',
        materials_provided: 'true',
        certification_available: 'true',
        fee: 'sliding_scale',
        languages: 'English,Portuguese',
        next_session: '2024-08-15',
        frequency: 'monthly'
      },
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString()
    });
    
    console.log('✅ Blockchain workshop registered');
    
    await client.mutualAid.registerResource({
      id: 'education_permaculture_course',
      name: 'Permaculture Design Course',
      description: 'Permaculture Design Certificate (PDC) course focusing on sustainable agriculture, ecological design, and community resilience.',
      category: 'education',
      provider_did: cooperativeA,
      availability: 'available',
      location: 'Various locations + Online components',
      contact_info: 'permaculture@cooperative-a.org',
      metadata: {
        education_type: 'certification_course',
        duration: '72_hours',
        format: 'blended',
        certification: 'pdc_certificate',
        instructors: '3_certified_teachers',
        practical_components: 'garden_design,soil_building,water_systems',
        fee: '$800_sliding_scale',
        scholarships_available: 'true',
        next_session: '2024-09-01',
        frequency: 'quarterly'
      },
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString()
    });
    
    console.log('✅ Permaculture course registered\n');

    // 4. Register Financial Assistance
    console.log('💰 Registering financial assistance resources...');
    
    await client.mutualAid.registerResource({
      id: 'finance_emergency_fund',
      name: 'Emergency Mutual Aid Fund',
      description: 'Community emergency fund providing financial assistance for unexpected expenses, medical costs, housing emergencies, and basic needs.',
      category: 'financial_assistance',
      provider_did: cooperativeA,
      availability: 'available',
      location: 'Distributed - Application-based',
      contact_info: 'mutualaid@cooperative-a.org',
      metadata: {
        fund_type: 'emergency_assistance',
        max_amount: '$2000',
        typical_amount: '$200_500',
        application_required: 'true',
        review_process: 'community_committee',
        response_time: '48_hours',
        repayment_required: 'false',
        eligibility: 'community_member,verified_need',
        categories: 'medical,housing,food,utilities,transportation',
        monthly_budget: '$10000'
      },
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString()
    });
    
    console.log('✅ Emergency fund registered');
    
    await client.mutualAid.registerResource({
      id: 'finance_microcredit',
      name: 'Cooperative Microcredit Program',
      description: 'Low-interest loans for cooperative members starting businesses, pursuing education, or developing community projects.',
      category: 'financial_assistance',
      provider_did: cooperativeB,
      availability: 'available',
      location: 'Regional Cooperative Network',
      contact_info: 'microcredit@cooperative-b.org',
      metadata: {
        fund_type: 'microcredit',
        loan_range: '$500_25000',
        interest_rate: '2_5_percent',
        terms: '6_60_months',
        application_required: 'true',
        business_plan_required: 'true',
        collateral_required: 'false',
        guarantor_required: 'true',
        approval_time: '2_weeks',
        categories: 'business,education,community_projects,equipment'
      },
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString()
    });
    
    console.log('✅ Microcredit program registered\n');

    // 5. Register Housing and Space Resources
    console.log('🏠 Registering housing and space resources...');
    
    await client.mutualAid.registerResource({
      id: 'housing_cohousing_guest',
      name: 'Cohousing Guest Accommodation',
      description: 'Temporary accommodation in cooperative cohousing community for visiting members, emergency housing, or transitional stays.',
      category: 'housing',
      provider_did: cooperativeA,
      availability: 'limited',
      location: 'Berkeley, CA - Cooperative Cohousing',
      contact_info: 'housing@cooperative-a.org',
      metadata: {
        housing_type: 'guest_room',
        capacity: '2_people',
        amenities: 'private_room,shared_kitchen,shared_bathroom,laundry,parking',
        daily_rate: '$50',
        weekly_rate: '$300',
        monthly_rate: '$1000',
        max_stay: '3_months',
        booking_advance: '1_week',
        community_integration: 'encouraged',
        accessibility: 'wheelchair_accessible'
      },
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString()
    });
    
    console.log('✅ Cohousing guest accommodation registered');
    
    await client.mutualAid.registerResource({
      id: 'space_meeting_room',
      name: 'Community Meeting and Event Space',
      description: 'Flexible meeting space with presentation equipment, suitable for workshops, meetings, community events, and collaborative work sessions.',
      category: 'space',
      provider_did: cooperativeB,
      availability: 'available',
      location: 'Austin, TX - Cooperative Center',
      contact_info: 'events@cooperative-b.org',
      metadata: {
        space_type: 'meeting_room',
        capacity: '50_people',
        equipment: 'projector,sound_system,whiteboards,wifi,kitchen_access',
        hourly_rate: '$25',
        daily_rate: '$150',
        booking_required: 'true',
        setup_assistance: 'available',
        catering_allowed: 'true',
        accessibility: 'ada_compliant'
      },
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString()
    });
    
    console.log('✅ Meeting space registered\n');

    // 6. List All Resources
    console.log('📋 Listing all mutual aid resources...');
    
    const allResources = await client.mutualAid.listResources();
    
    console.log(`🤝 Found ${allResources.length} mutual aid resources:`);
    allResources.forEach((resource, index) => {
      const availabilityFormatted = EnhancedUtils.formatResourceAvailability(resource.availability);
      console.log(`   ${index + 1}. ${resource.name}`);
      console.log(`      Category: ${resource.category}`);
      console.log(`      Provider: ${resource.provider_did.slice(0, 20)}...`);
      console.log(`      Availability: ${availabilityFormatted}`);
      console.log(`      Location: ${resource.location || 'Not specified'}`);
      console.log(`      Contact: ${resource.contact_info || 'See provider'}`);
      if (resource.metadata?.hourly_rate || resource.metadata?.daily_rate) {
        const rates = [];
        if (resource.metadata.hourly_rate) rates.push(`${resource.metadata.hourly_rate}/hour`);
        if (resource.metadata.daily_rate) rates.push(`${resource.metadata.daily_rate}/day`);
        console.log(`      Rates: ${rates.join(', ')}`);
      }
    });
    console.log();

    // 7. Resource Discovery by Category
    console.log('🔍 Resource discovery by category...');
    
    const categories = ['technical_skills', 'equipment', 'education', 'financial_assistance', 'housing', 'space'];
    
    for (const category of categories) {
      const categoryResources = allResources.filter(r => r.category === category);
      console.log(`\n📂 ${category.replace('_', ' ').toUpperCase()} (${categoryResources.length} resources):`);
      
      categoryResources.slice(0, 3).forEach((resource, index) => {
        console.log(`   ${index + 1}. ${resource.name}`);
        console.log(`      Provider: ${resource.provider_did.slice(0, 20)}...`);
        console.log(`      Location: ${resource.location || 'Various/Remote'}`);
        
        // Show relevant metadata based on category
        if (category === 'technical_skills' && resource.metadata?.skills) {
          console.log(`      Skills: ${resource.metadata.skills}`);
        } else if (category === 'equipment' && resource.metadata?.equipment_type) {
          console.log(`      Type: ${resource.metadata.equipment_type}`);
        } else if (category === 'education' && resource.metadata?.duration) {
          console.log(`      Duration: ${resource.metadata.duration}`);
        } else if (category === 'financial_assistance' && resource.metadata?.max_amount) {
          console.log(`      Max Amount: ${resource.metadata.max_amount}`);
        } else if (category === 'housing' && resource.metadata?.capacity) {
          console.log(`      Capacity: ${resource.metadata.capacity}`);
        } else if (category === 'space' && resource.metadata?.capacity) {
          console.log(`      Capacity: ${resource.metadata.capacity}`);
        }
      });
      
      if (categoryResources.length > 3) {
        console.log(`   ... and ${categoryResources.length - 3} more ${category} resources`);
      }
    }
    console.log();

    // 8. Resource Utilization Analytics
    console.log('📊 Resource utilization analytics...');
    
    // Analyze resource distribution
    const resourcesByCategory = categories.map(category => ({
      category,
      count: allResources.filter(r => r.category === category).length
    }));
    
    console.log('📈 Resource Distribution:');
    resourcesByCategory.forEach(({ category, count }) => {
      const percentage = ((count / allResources.length) * 100).toFixed(1);
      console.log(`   ${category.replace('_', ' ')}: ${count} resources (${percentage}%)`);
    });
    
    // Analyze availability
    const availabilityStats = allResources.reduce((acc, resource) => {
      acc[resource.availability] = (acc[resource.availability] || 0) + 1;
      return acc;
    }, {} as Record<string, number>);
    
    console.log('\n📊 Availability Status:');
    Object.entries(availabilityStats).forEach(([status, count]) => {
      const percentage = ((count / allResources.length) * 100).toFixed(1);
      console.log(`   ${EnhancedUtils.formatResourceAvailability(status)}: ${count} (${percentage}%)`);
    });
    
    // Provider analysis
    const resourcesByProvider = allResources.reduce((acc, resource) => {
      acc[resource.provider_did] = (acc[resource.provider_did] || 0) + 1;
      return acc;
    }, {} as Record<string, number>);
    
    console.log('\n👥 Resources by Provider:');
    Object.entries(resourcesByProvider).forEach(([provider, count]) => {
      console.log(`   ${provider.slice(0, 25)}...: ${count} resource(s)`);
    });
    console.log();

    // 9. Resource Matching and Recommendations
    console.log('🎯 Resource matching and recommendations...');
    
    // Simulate resource matching scenarios
    const scenarios = [
      {
        need: 'Web development help for cooperative website',
        category: 'technical_skills',
        location: 'Remote',
        budget: 'Volunteer/Exchange'
      },
      {
        need: 'Emergency housing for displaced member',
        category: 'housing',
        location: 'San Francisco Bay Area',
        urgency: 'High'
      },
      {
        need: 'Training space for blockchain workshop',
        category: 'space',
        location: 'Austin, TX',
        date: '2024-08-15'
      },
      {
        need: 'Prototype development equipment access',
        category: 'equipment',
        location: 'Any',
        skills_required: 'Electronics'
      }
    ];
    
    scenarios.forEach((scenario, index) => {
      console.log(`\n🔍 Scenario ${index + 1}: ${scenario.need}`);
      
      const matchingResources = allResources.filter(resource => {
        // Basic category matching
        if (resource.category !== scenario.category) return false;
        
        // Location matching (simplified)
        if (scenario.location !== 'Any' && scenario.location !== 'Remote') {
          if (!resource.location?.toLowerCase().includes(scenario.location.toLowerCase()) &&
              !resource.location?.toLowerCase().includes('remote')) {
            return false;
          }
        }
        
        // Availability check
        if (resource.availability === 'unavailable') return false;
        
        return true;
      });
      
      console.log(`   📋 Found ${matchingResources.length} matching resources:`);
      matchingResources.slice(0, 2).forEach((resource, idx) => {
        console.log(`     ${idx + 1}. ${resource.name}`);
        console.log(`        Provider: ${resource.provider_did.slice(0, 20)}...`);
        console.log(`        Contact: ${resource.contact_info}`);
        console.log(`        Location: ${resource.location}`);
        
        // Show match quality
        let matchScore = 80; // Base score
        if (resource.location?.toLowerCase().includes('remote') && scenario.location === 'Remote') {
          matchScore += 10;
        }
        if (resource.availability === 'available') {
          matchScore += 10;
        }
        console.log(`        Match Score: ${Math.min(matchScore, 100)}%`);
      });
      
      if (matchingResources.length === 0) {
        console.log('     ⚠️  No matching resources found');
        console.log('     💡 Consider: Expanding search criteria or posting a request');
      }
    });
    console.log();

    // 10. Update Resource Availability
    console.log('🔄 Updating resource availability...');
    
    // Simulate resource usage
    await client.mutualAid.updateResource('equipment_3d_printer', {
      availability: 'unavailable',
      metadata: {
        ...allResources.find(r => r.id === 'equipment_3d_printer')?.metadata,
        current_user: 'did:key:project-team-456',
        estimated_completion: '2024-08-10',
        status_reason: 'In use for community solar panel housing project'
      },
      updated_at: new Date().toISOString()
    });
    
    console.log('✅ 3D printer marked as unavailable (in use)');
    
    await client.mutualAid.updateResource('education_blockchain_workshop', {
      availability: 'limited',
      metadata: {
        ...allResources.find(r => r.id === 'education_blockchain_workshop')?.metadata,
        current_enrollment: '18',
        max_participants: '25',
        spots_remaining: '7',
        registration_deadline: '2024-08-10'
      },
      updated_at: new Date().toISOString()
    });
    
    console.log('✅ Blockchain workshop enrollment updated');
    console.log();

    // 11. Community Impact Analysis
    console.log('🌟 Community impact analysis...');
    
    console.log('💡 Mutual Aid Network Insights:');
    console.log(`   Total Resources Registered: ${allResources.length}`);
    console.log(`   Categories Covered: ${categories.length}`);
    console.log(`   Active Providers: ${Object.keys(resourcesByProvider).length}`);
    console.log(`   Available Resources: ${availabilityStats.available || 0}`);
    console.log(`   Limited Availability: ${availabilityStats.limited || 0}`);
    
    // Calculate potential value
    const resourcesWithRates = allResources.filter(r => 
      r.metadata?.hourly_rate || r.metadata?.daily_rate || r.metadata?.max_amount
    );
    
    console.log('\n💰 Economic Impact Potential:');
    console.log(`   Resources with defined value: ${resourcesWithRates.length}`);
    console.log(`   Skill-sharing resources: ${allResources.filter(r => r.category === 'technical_skills').length}`);
    console.log(`   Equipment sharing resources: ${allResources.filter(r => r.category === 'equipment').length}`);
    console.log(`   Educational opportunities: ${allResources.filter(r => r.category === 'education').length}`);
    
    console.log('\n🤝 Cooperation Benefits:');
    console.log('   • Reduced individual costs through resource sharing');
    console.log('   • Increased access to specialized skills and equipment');
    console.log('   • Community resilience through mutual support networks');
    console.log('   • Knowledge transfer and skill development opportunities');
    console.log('   • Economic solidarity and reduced inequality');
    console.log('   • Environmental benefits through resource efficiency');
    
    console.log('\n🎯 Network Growth Opportunities:');
    console.log('   • Encourage more skill-sharing registrations');
    console.log('   • Develop resource request/matching system');
    console.log('   • Implement resource usage tracking and feedback');
    console.log('   • Create incentive systems for active sharing');
    console.log('   • Expand to include more resource categories');
    console.log('   • Build inter-cooperative resource networks');

    console.log('\n🎉 Mutual Aid example completed successfully!');
    console.log('\n💡 Key Features Demonstrated:');
    console.log('   • Comprehensive resource registration across multiple categories');
    console.log('   • Resource discovery and filtering by category and location');
    console.log('   • Availability management and real-time updates');
    console.log('   • Resource matching algorithms and recommendations');
    console.log('   • Community analytics and impact measurement');
    console.log('   • Provider diversity and resource distribution analysis');
    
    console.log('\n🤝 Mutual Aid Network Benefits:');
    console.log('   • Democratic resource sharing within cooperative communities');
    console.log('   • Reduced costs through collective resource ownership');
    console.log('   • Enhanced community resilience and self-reliance');
    console.log('   • Skill sharing and knowledge transfer networks');
    console.log('   • Economic solidarity and reduced inequality');
    console.log('   • Sustainable resource utilization patterns');

  } catch (error) {
    console.error('❌ Error during mutual aid example:');
    
    if (ErrorUtils.isErrorType(error, ICNValidationError)) {
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
  mutualAidExample().catch(console.error);
}

export { mutualAidExample };
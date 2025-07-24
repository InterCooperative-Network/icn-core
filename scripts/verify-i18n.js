#!/usr/bin/env node

/**
 * Simple verification script to check i18n implementation
 * This tests that language files load correctly and translations work
 */

const fs = require('fs');
const path = require('path');

console.log('🌍 Testing ICN i18n Implementation...\n');

// Test 1: Check that language files exist
console.log('📁 Checking language files...');
const languages = ['en', 'es'];
const namespaces = ['common', 'navigation', 'dashboard', 'accessibility'];

let missingFiles = 0;

for (const lang of languages) {
  for (const ns of namespaces) {
    const filePath = path.join(__dirname, '..', 'packages', 'i18n', 'locales', lang, `${ns}.json`);
    if (fs.existsSync(filePath)) {
      console.log(`✓ ${lang}/${ns}.json exists`);
    } else {
      console.log(`✗ ${lang}/${ns}.json missing`);
      missingFiles++;
    }
  }
}

// Test 2: Check that translation files are valid JSON
console.log('\n📋 Validating JSON syntax...');
let invalidFiles = 0;

for (const lang of languages) {
  for (const ns of namespaces) {
    const filePath = path.join(__dirname, '..', 'packages', 'i18n', 'locales', lang, `${ns}.json`);
    if (fs.existsSync(filePath)) {
      try {
        const content = fs.readFileSync(filePath, 'utf8');
        JSON.parse(content);
        console.log(`✓ ${lang}/${ns}.json is valid JSON`);
      } catch (error) {
        console.log(`✗ ${lang}/${ns}.json has invalid JSON: ${error.message}`);
        invalidFiles++;
      }
    }
  }
}

// Test 3: Check for translation key consistency
console.log('\n🔄 Checking translation key consistency...');
let inconsistentKeys = 0;

const enTranslations = {};
const esTranslations = {};

// Load English translations (reference)
for (const ns of namespaces) {
  const filePath = path.join(__dirname, '..', 'packages', 'i18n', 'locales', 'en', `${ns}.json`);
  if (fs.existsSync(filePath)) {
    try {
      enTranslations[ns] = JSON.parse(fs.readFileSync(filePath, 'utf8'));
    } catch (error) {
      console.log(`✗ Could not load en/${ns}.json`);
    }
  }
}

// Load Spanish translations
for (const ns of namespaces) {
  const filePath = path.join(__dirname, '..', 'packages', 'i18n', 'locales', 'es', `${ns}.json`);
  if (fs.existsSync(filePath)) {
    try {
      esTranslations[ns] = JSON.parse(fs.readFileSync(filePath, 'utf8'));
    } catch (error) {
      console.log(`✗ Could not load es/${ns}.json`);
    }
  }
}

// Check key consistency
function checkKeys(obj1, obj2, path = '') {
  for (const key in obj1) {
    const currentPath = path ? `${path}.${key}` : key;
    if (typeof obj1[key] === 'object' && obj1[key] !== null) {
      if (typeof obj2[key] === 'object' && obj2[key] !== null) {
        checkKeys(obj1[key], obj2[key], currentPath);
      } else {
        console.log(`✗ Missing nested key in Spanish: ${currentPath}`);
        inconsistentKeys++;
      }
    } else {
      if (!(key in obj2)) {
        console.log(`✗ Missing key in Spanish: ${currentPath}`);
        inconsistentKeys++;
      }
    }
  }
}

for (const ns of namespaces) {
  if (enTranslations[ns] && esTranslations[ns]) {
    console.log(`Checking keys for namespace: ${ns}`);
    checkKeys(enTranslations[ns], esTranslations[ns]);
  }
}

// Test 4: Check package.json configurations
console.log('\n📦 Checking package configurations...');
let configIssues = 0;

const i18nPackagePath = path.join(__dirname, '..', 'packages', 'i18n', 'package.json');
const webUiPackagePath = path.join(__dirname, '..', 'apps', 'web-ui', 'package.json');

try {
  const i18nPkg = JSON.parse(fs.readFileSync(i18nPackagePath, 'utf8'));
  
  if (i18nPkg.name === '@icn/i18n') {
    console.log('✓ i18n package name is correct');
  } else {
    console.log('✗ i18n package name incorrect');
    configIssues++;
  }

  if (i18nPkg.peerDependencies && i18nPkg.peerDependencies['react-i18next']) {
    console.log('✓ react-i18next is listed as peer dependency');
  } else {
    console.log('✗ react-i18next missing from peer dependencies');
    configIssues++;
  }
} catch (error) {
  console.log('✗ Could not read i18n package.json');
  configIssues++;
}

try {
  const webUiPkg = JSON.parse(fs.readFileSync(webUiPackagePath, 'utf8'));
  
  if (webUiPkg.dependencies && webUiPkg.dependencies['@icn/i18n']) {
    console.log('✓ web-ui depends on @icn/i18n');
  } else {
    console.log('✗ web-ui missing @icn/i18n dependency');
    configIssues++;
  }

  if (webUiPkg.dependencies && webUiPkg.dependencies['react-i18next']) {
    console.log('✓ web-ui has react-i18next dependency');
  } else {
    console.log('✗ web-ui missing react-i18next dependency');
    configIssues++;
  }
} catch (error) {
  console.log('✗ Could not read web-ui package.json');
  configIssues++;
}

// Summary
console.log('\n📊 Test Summary:');
console.log(`Missing files: ${missingFiles}`);
console.log(`Invalid JSON files: ${invalidFiles}`);
console.log(`Inconsistent translation keys: ${inconsistentKeys}`);
console.log(`Configuration issues: ${configIssues}`);

const totalIssues = missingFiles + invalidFiles + inconsistentKeys + configIssues;

if (totalIssues === 0) {
  console.log('\n🎉 All tests passed! i18n implementation looks good.');
  process.exit(0);
} else {
  console.log(`\n⚠️  Found ${totalIssues} issues that should be addressed.`);
  process.exit(1);
}
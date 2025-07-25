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
const languages = ['en', 'es', 'fr'];
const namespaces = ['common', 'navigation', 'dashboard', 'accessibility', 'explorer', 'wallet'];

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

// Test 3: Check for translation key consistency across all languages
console.log('\n🔄 Checking translation key consistency...');
let inconsistentKeys = 0;

const translations = {};

// Load all translations
for (const lang of languages) {
  translations[lang] = {};
  for (const ns of namespaces) {
    const filePath = path.join(__dirname, '..', 'packages', 'i18n', 'locales', lang, `${ns}.json`);
    if (fs.existsSync(filePath)) {
      try {
        translations[lang][ns] = JSON.parse(fs.readFileSync(filePath, 'utf8'));
      } catch (error) {
        console.log(`✗ Could not load ${lang}/${ns}.json`);
      }
    }
  }
}

// Check key consistency against English (reference language)
function checkKeys(obj1, obj2, path = '', lang = '') {
  for (const key in obj1) {
    const currentPath = path ? `${path}.${key}` : key;
    if (typeof obj1[key] === 'object' && obj1[key] !== null) {
      if (typeof obj2[key] === 'object' && obj2[key] !== null) {
        checkKeys(obj1[key], obj2[key], currentPath, lang);
      } else {
        console.log(`✗ Missing nested key in ${lang}: ${currentPath}`);
        inconsistentKeys++;
      }
    } else {
      if (!(key in obj2)) {
        console.log(`✗ Missing key in ${lang}: ${currentPath}`);
        inconsistentKeys++;
      }
    }
  }
}

for (const ns of namespaces) {
  if (translations['en'][ns]) {
    console.log(`Checking keys for namespace: ${ns}`);
    for (const lang of languages) {
      if (lang !== 'en' && translations[lang][ns]) {
        checkKeys(translations['en'][ns], translations[lang][ns], '', lang);
      }
    }
  }
}

// Test 4: Check package.json configurations
console.log('\n📦 Checking package configurations...');
let configIssues = 0;

const packagePaths = {
  'i18n': path.join(__dirname, '..', 'packages', 'i18n', 'package.json'),
  'web-ui': path.join(__dirname, '..', 'apps', 'web-ui', 'package.json'),
  'explorer': path.join(__dirname, '..', 'apps', 'explorer', 'package.json'),
  'wallet-ui': path.join(__dirname, '..', 'apps', 'wallet-ui', 'package.json')
};

// Check i18n package
try {
  const i18nPkg = JSON.parse(fs.readFileSync(packagePaths['i18n'], 'utf8'));
  
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

// Check app packages
const apps = ['web-ui', 'explorer', 'wallet-ui'];
for (const app of apps) {
  try {
    const appPkg = JSON.parse(fs.readFileSync(packagePaths[app], 'utf8'));
    
    if (appPkg.dependencies && appPkg.dependencies['@icn/i18n']) {
      console.log(`✓ ${app} depends on @icn/i18n`);
    } else {
      console.log(`✗ ${app} missing @icn/i18n dependency`);
      configIssues++;
    }

    if (appPkg.dependencies && appPkg.dependencies['react-i18next']) {
      console.log(`✓ ${app} has react-i18next dependency`);
    } else {
      console.log(`✗ ${app} missing react-i18next dependency`);
      configIssues++;
    }
  } catch (error) {
    console.log(`✗ Could not read ${app} package.json`);
    configIssues++;
  }
}

// Test 5: Check documentation exists
console.log('\n📚 Checking documentation files...');
let docIssues = 0;

const requiredDocs = [
  'docs/I18N_AND_ACCESSIBILITY.md',
  'docs/ACCESSIBILITY_AUDIT.md', 
  'docs/LANGUAGE_ADDITION_GUIDE.md'
];

for (const docPath of requiredDocs) {
  const fullPath = path.join(__dirname, '..', docPath);
  if (fs.existsSync(fullPath)) {
    console.log(`✓ ${docPath} exists`);
  } else {
    console.log(`✗ ${docPath} missing`);
    docIssues++;
  }
}

// Summary
console.log('\n📊 Test Summary:');
console.log(`Missing files: ${missingFiles}`);
console.log(`Invalid JSON files: ${invalidFiles}`);
console.log(`Inconsistent translation keys: ${inconsistentKeys}`);
console.log(`Configuration issues: ${configIssues}`);
console.log(`Documentation issues: ${docIssues}`);

const totalIssues = missingFiles + invalidFiles + inconsistentKeys + configIssues + docIssues;

if (totalIssues === 0) {
  console.log('\n🎉 All tests passed! i18n implementation looks good.');
  process.exit(0);
} else {
  console.log(`\n⚠️  Found ${totalIssues} issues that should be addressed.`);
  process.exit(1);
}
const { getDefaultConfig } = require('expo/metro-config');

const config = getDefaultConfig(__dirname);

// Add Tamagui support
config.resolver.assetExts.push('css');

module.exports = config;
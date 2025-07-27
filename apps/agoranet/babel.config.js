module.exports = function (api) {
  api.cache(true);
  return {
    presets: ['babel-preset-expo'],
    plugins: [
      [
        '@tamagui/babel-plugin',
        {
          components: ['@tamagui/core'],
          config: './src/tamagui.config.ts',
        },
      ],
      'expo-router/babel',
    ],
  };
};
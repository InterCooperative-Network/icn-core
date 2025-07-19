import { defineConfig } from 'tsup'

export default defineConfig({
  entry: ['src/index.ts'],
  format: ['cjs', 'esm'],
  dts: true,
  splitting: false,
  sourcemap: true,
  clean: true,
  external: [
    '@react-native-async-storage/async-storage'
  ],
  platform: 'neutral',
  target: 'es2020',
}) 
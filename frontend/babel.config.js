module.exports = {
  presets: [
    ['@babel/preset-env', {
      targets: {
        node: 'current'
      }
    }],
    '@babel/preset-react',
    '@babel/preset-typescript'
  ],
  plugins: [
    // Add any additional plugins here
  ],
  env: {
    test: {
      plugins: [
        // Plugin for handling dynamic imports in tests
        '@babel/plugin-syntax-dynamic-import',
      ]
    }
  }
};
const path = require("path");
const { getDefaultConfig, mergeConfig } = require("@react-native/metro-config");

const workspaceRoot = path.resolve(__dirname, "..");
const projectRoot = __dirname;

module.exports = mergeConfig(getDefaultConfig(projectRoot), {
  watchFolders: [workspaceRoot],
  resolver: {
    nodeModulesPaths: [
      path.resolve(projectRoot, "node_modules"),
      path.resolve(workspaceRoot, "node_modules"),
    ],
  },
});

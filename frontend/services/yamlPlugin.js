import { plugin } from "bun";
import { parseYAML } from "./yamlParser.js";

await plugin({
  name: "YAML",
  async setup(build) {
    build.onLoad({ filter: /\.(yaml|yml)$/ }, async (args) => {
      const text = await Bun.file(args.path).text();
      const exports = parseYAML(text);

      return {
        exports,
        loader: "object",
      };
    });
  },
});
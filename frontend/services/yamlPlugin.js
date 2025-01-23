import { plugin } from "bun";

await plugin({
  name: "YAML",
  async setup(build) {
    const { load } = await import("js-yaml");

    build.onLoad({ filter: /\.(yaml|yml)$/ }, async (args) => {
      const text = await Bun.file(args.path).text();
      const exports = load(text);

      return {
        exports,
        loader: "object",
      };
    });
  },
});
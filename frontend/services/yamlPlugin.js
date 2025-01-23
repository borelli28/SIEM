import { plugin } from "bun";

await plugin({
  name: "YAML",
  async setup(build) {
    build.onLoad({ filter: /\.(yaml|yml)$/ }, async (args) => {
      const text = await Bun.file(args.path).text();
      
      // Use the same YAML parser we created for the frontend
      function parseYAML(yamlString) {
        const lines = yamlString.split('\n');
        const result = {};
        let currentContext = result;
        let currentKey = '';
        let indentLevel = 0;
        let parentContexts = [];

        for (let line of lines) {
            if (!line.trim() || line.trim().startsWith('#')) continue;

            const indent = line.search(/\S/);
            const trimmedLine = line.trim();

            if (indent < indentLevel) {
                while (indent < indentLevel && parentContexts.length > 0) {
                    currentContext = parentContexts.pop();
                    indentLevel -= 2;
                }
            }

            if (trimmedLine.includes(':')) {
                const [key, value] = trimmedLine.split(':').map(s => s.trim());
                
                if (value && value !== '') {
                    currentContext[key] = value;
                } else {
                    currentKey = key;
                    currentContext[currentKey] = {};
                    parentContexts.push(currentContext);
                    currentContext = currentContext[currentKey];
                    indentLevel = indent;
                }
            } else if (trimmedLine.startsWith('-')) {
                const value = trimmedLine.substring(1).trim();
                if (!Array.isArray(currentContext)) {
                    currentContext = [];
                    parentContexts[parentContexts.length - 1][currentKey] = currentContext;
                }
                currentContext.push(value);
            }
        }

        return result;
      }

      const exports = parseYAML(text);

      return {
        exports,
        loader: "object",
      };
    });
  },
});
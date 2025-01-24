export function parseYAML(yamlString) {
    const lines = yamlString.split('\n');
    const result = {};
    const stack = [{ indent: -1, object: result }];
    let lastIndent = -1;

    for (let line of lines) {
        // Skip empty lines and comments
        if (!line.trim() || line.trim().startsWith('#')) continue;

        // Calculate indent level (count spaces)
        const indent = line.search(/\S/);
        const trimmedLine = line.trim();

        // Handle arrays and key-value pairs
        if (trimmedLine.startsWith('-')) {
            // Array item
            const value = trimmedLine.substring(1).trim();
            const parent = findParentInStack(stack, indent);
            const key = Object.keys(parent.object).pop();
            
            if (!Array.isArray(parent.object[key])) {
                // Initialize array if it doesn't exist
                parent.object[key] = [];
            }
            parent.object[key].push(value);
        } else if (trimmedLine.includes(':')) {
            // Key-value pair
            const [key, value] = trimmedLine.split(':').map(s => s.trim());
            
            if (indent <= lastIndent) {
                // Going back up in the hierarchy
                while (stack.length > 0 && stack[stack.length - 1].indent >= indent) {
                    stack.pop();
                }
            }

            const parent = stack[stack.length - 1].object;
            if (value) {
                // Direct key-value
                parent[key] = value.replace(/['"]/g, ''); // Remove quotes if present
            } else {
                // New nested object
                parent[key] = {};
                stack.push({ indent, object: parent[key] });
            }
        }

        lastIndent = indent;
    }

    return result;
}

function findParentInStack(stack, indent) {
    // Find appropriate parent for current indent level
    for (let i = stack.length - 1; i >= 0; i--) {
        if (stack[i].indent < indent) {
            return stack[i];
        }
    }
    return stack[0];
}
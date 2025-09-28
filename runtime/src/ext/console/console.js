globalThis.console = {
  log(...args) {
    const formatted = args.map((arg) => {
      if (typeof arg === "string") return arg;
      if (arg === null) return "null";
      if (arg === undefined) return "undefined";
      if (typeof arg === "number" || typeof arg === "boolean") {
        return String(arg);
      }
      if (typeof arg === "function") {
        return `[Function: ${arg.name || "anonymous"}]`;
      }
      if (typeof arg === "symbol") return arg.toString();

      if (Object.prototype.toString.call(arg) === "[object Date]") {
        return arg.toISOString();
      }

      if (Array.isArray(arg)) {
        if (arg.length >= 2) {
          const items = arg.map((item) => {
            if (typeof item === "string") return `  ${JSON.stringify(item)}`;
            if (item === null) return "  null";
            if (item === undefined) return "  undefined";
            return `  ${String(item)}`;
          }).join(",\n");
          return `[\n${items}\n]`;
        } else {
          const items = arg.map((item) => {
            if (typeof item === "string") return JSON.stringify(item);
            if (item === null) return "null";
            if (item === undefined) return "undefined";
            return String(item);
          }).join(", ");
          return `[ ${items} ]`;
        }
      }

      if (typeof arg === "object") {
        const constructorName = arg.constructor?.name;
        if (constructorName && constructorName !== "Object") {
          const ownKeys = Object.getOwnPropertyNames(arg);
          const protoKeys = arg.constructor.prototype
            ? Object.getOwnPropertyNames(arg.constructor.prototype).filter(
              (key) => key !== "constructor",
            )
            : [];
          const allKeys = [...new Set([...ownKeys, ...protoKeys])];

          const props = allKeys
            .filter((key) => !key.startsWith("_"))
            .map((key) => {
              try {
                const val = arg[key];
                if (typeof val === "function") return null;
                const valStr = typeof val === "string"
                  ? `"${val}"`
                  : String(val);
                return `  ${key}: ${valStr}`;
              } catch {
                return null;
              }
            })
            .filter((p) => p !== null)
            .join(",\n");
          return `${constructorName} {\n${props}\n}`;
        }
      }

      try {
        return JSON.stringify(arg, null, 2);
      } catch (_e) {
        return String(arg);
      }
    }).join(" ");
    __print(formatted);
  },
  error(...args) {
    const formatted = args.map((arg) => {
      if (typeof arg === "string") return arg;
      if (arg === null) return "null";
      if (arg === undefined) return "undefined";
      if (typeof arg === "number" || typeof arg === "boolean") {
        return String(arg);
      }
      if (typeof arg === "function") {
        return `[Function: ${arg.name || "anonymous"}]`;
      }
      if (typeof arg === "symbol") return arg.toString();

      if (Object.prototype.toString.call(arg) === "[object Date]") {
        return arg.toISOString();
      }

      if (Array.isArray(arg)) {
        if (arg.length >= 2) {
          const items = arg.map((item) => {
            if (typeof item === "string") return `  ${JSON.stringify(item)}`;
            if (item === null) return "  null";
            if (item === undefined) return "  undefined";
            return `  ${String(item)}`;
          }).join(",\n");
          return `[\n${items}\n]`;
        } else {
          const items = arg.map((item) => {
            if (typeof item === "string") return JSON.stringify(item);
            if (item === null) return "null";
            if (item === undefined) return "undefined";
            return String(item);
          }).join(", ");
          return `[ ${items} ]`;
        }
      }

      if (typeof arg === "object") {
        const constructorName = arg.constructor?.name;
        if (constructorName && constructorName !== "Object") {
          const ownKeys = Object.getOwnPropertyNames(arg);
          const protoKeys = arg.constructor.prototype
            ? Object.getOwnPropertyNames(arg.constructor.prototype).filter(
              (key) => key !== "constructor",
            )
            : [];
          const allKeys = [...new Set([...ownKeys, ...protoKeys])];

          const props = allKeys
            .filter((key) => !key.startsWith("_"))
            .map((key) => {
              try {
                const val = arg[key];
                if (typeof val === "function") return null;
                const valStr = typeof val === "string"
                  ? `"${val}"`
                  : String(val);
                return `  ${key}: ${valStr}`;
              } catch {
                return null;
              }
            })
            .filter((p) => p !== null)
            .join(",\n");
          return `${constructorName} {\n${props}\n}`;
        }
      }

      try {
        return JSON.stringify(arg, null, 2);
      } catch (_e) {
        return String(arg);
      }
    }).join(" ");
    __print(formatted);
  },
};

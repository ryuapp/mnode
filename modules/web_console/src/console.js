function formatValue(arg) {
  if (typeof arg === "string") return arg;
  if (arg === null) return "null";
  if (arg === undefined) return "undefined";
  if (typeof arg === "number" || typeof arg === "boolean") {
    return String(arg);
  }
  if (typeof arg === "function") {
    // Check if it's a class (constructor function)
    const fnStr = arg.toString();
    if (fnStr.startsWith("class ")) {
      const className = arg.name || "anonymous";
      // Check for parent class
      const parent = Object.getPrototypeOf(arg);
      if (parent && parent !== Function.prototype && parent.name) {
        return `[class ${className} extends ${parent.name}]`;
      }
      return `[class ${className}]`;
    }
    return `[Function: ${arg.name || "anonymous"}]`;
  }
  if (typeof arg === "symbol") return arg.toString();

  if (Object.prototype.toString.call(arg) === "[object Date]") {
    return arg.toISOString();
  }

  // Check for Promise
  if (arg instanceof Promise) {
    return "Promise { <pending> }";
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
            const valStr = typeof val === "string" ? `"${val}"` : String(val);
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
}

const print = globalThis[Symbol.for("mdeno.internal")].print;

globalThis.console = {
  log(...args) {
    const formatted = args.map(formatValue).join(" ");
    print(formatted);
  },
  error(...args) {
    const formatted = args.map(formatValue).join(" ");
    print(formatted);
  },
};

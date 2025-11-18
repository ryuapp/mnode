// Copyright 2018-2025 the Deno authors. MIT license.
globalThis.Deno ||= {};

// https://docs.deno.com/api/deno/~/Deno.exit
globalThis.Deno.exit = function (code) {
  __internal.exit(code);
};

// https://docs.deno.com/api/deno/~/Deno.Env
globalThis.Deno.env = {
  get: function (key) {
    return __internal.env.get(key);
  },
  set: function (key, value) {
    __internal.env.set(key, value);
  },
  delete: function (key) {
    __internal.env.delete(key);
  },
  has: function (key) {
    return __internal.env.has(key);
  },
  toObject: function () {
    return __internal.env.toObject();
  },
};

// https://docs.deno.com/api/deno/~/Deno.noColor
const noColorValue = globalThis.__mdeno_no_color;
Object.defineProperty(globalThis.Deno, "noColor", {
  get() {
    return noColorValue;
  },
});
delete globalThis.__mdeno_no_color;

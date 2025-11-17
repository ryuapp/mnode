// Copyright 2018-2025 the Deno authors. MIT license.
globalThis.Deno ||= {};

// https://docs.deno.com/api/deno/~/Deno.exit
globalThis.Deno.exit = function (code) {
  internal.exit(code);
};

// https://docs.deno.com/api/deno/~/Deno.Env
globalThis.Deno.env = {
  get: function (key) {
    return internal.env.get(key);
  },
  set: function (key, value) {
    internal.env.set(key, value);
  },
  delete: function (key) {
    internal.env.delete(key);
  },
  has: function (key) {
    return internal.env.has(key);
  },
  toObject: function () {
    return internal.env.toObject();
  },
};

// Copyright 2018-2025 the Deno authors. MIT license.
globalThis.Deno ||= {};

// https://docs.deno.com/api/deno/~/Deno.exit
globalThis.Deno.exit = function (code) {
  const internal = globalThis[Symbol.for("mdeno.internal")];
  internal.exit(code);
};

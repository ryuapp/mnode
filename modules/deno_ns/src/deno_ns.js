// Copyright 2018-2025 the Deno authors. MIT license.
// Deno namespace binding point - individual modules define their own APIs

const fs = globalThis.__mdeno__.fs;
const os = globalThis.__mdeno__.os;

const denoNs = {
  // File System APIs
  readFileSync: fs.readFileSync,
  readTextFileSync: fs.readTextFileSync,
  writeFileSync: fs.writeFileSync,
  writeTextFileSync: fs.writeTextFileSync,
  statSync: fs.statSync,
  mkdirSync: fs.mkdirSync,
  removeSync: fs.removeSync,
  copyFileSync: fs.copyFileSync,

  // OS APIs
  exit: os.exit,
  env: os.env,
};

// Add noColor as a getter
Object.defineProperty(denoNs, "noColor", {
  get() {
    return os.noColor;
  },
});

// Define globalThis.Deno
Object.defineProperty(globalThis, "Deno", {
  value: denoNs,
  enumerable: false,
  writable: false,
  configurable: false,
});

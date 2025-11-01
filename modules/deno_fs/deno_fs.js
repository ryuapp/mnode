// Copyright 2018-2025 the Deno authors. MIT license.
const internal = globalThis[Symbol.for("mdeno.internal")];

globalThis.Deno ||= {};

internal["fs.readFileSync"] = internal.fs.readFileSync;
internal["fs.readTextFileSync"] = internal.fs.readTextFileSync;
internal["fs.writeFileSync"] = internal.fs.writeFileSync;
internal["fs.writeTextFileSync"] = internal.fs.writeTextFileSync;
internal["fs.statSync"] = internal.fs.statSync;
internal["fs.mkdirSync"] = internal.fs.mkdirSync;
internal["fs.removeSync"] = internal.fs.removeSync;
internal["fs.copyFileSync"] = internal.fs.copyFileSync;

const PATHNAME_WIN_RE = /^\/*([A-Za-z]:)(\/|$)/;
const SLASH_WIN_RE = /\//g;
const PERCENT_RE = /%(?![0-9A-Fa-f]{2})/g;

// Convert Windows file URL to path (e.g., file:///C:/path → C:\path)
function pathFromURLWin32(url) {
  let p = url.pathname.replace(PATHNAME_WIN_RE, "$1/");
  p = p.replace(SLASH_WIN_RE, "\\");
  p = p.replace(PERCENT_RE, "%25");
  let path = decodeURIComponent(p);
  if (url.hostname !== "") {
    path = `\\\\${url.hostname}${path}`;
  }
  return path;
}

// Convert POSIX file URL to path (e.g., file:///home/user/path → /home/user/path)
function pathFromURLPosix(url) {
  if (url.hostname !== "") {
    throw new TypeError("Host must be empty");
  }
  return decodeURIComponent(
    url.pathname.replace(PERCENT_RE, "%25"),
  );
}

function pathFromURL(pathOrUrl) {
  if (pathOrUrl instanceof URL) {
    if (pathOrUrl.protocol !== "file:") {
      throw new TypeError("Must be a file URL");
    }

    return navigator.platform === "Win32"
      ? pathFromURLWin32(pathOrUrl)
      : pathFromURLPosix(pathOrUrl);
  }
  return String(pathOrUrl);
}

// https://docs.deno.com/api/deno/~/Deno.readFileSync
globalThis.Deno.readFileSync = function (path) {
  path = pathFromURL(path);
  return internal["fs.readFileSync"](path);
};

// https://docs.deno.com/api/deno/~/Deno.readTextFileSync
globalThis.Deno.readTextFileSync = function (path) {
  path = pathFromURL(path);
  return internal["fs.readTextFileSync"](path);
};

// https://docs.deno.com/api/deno/~/Deno.writeFileSync
globalThis.Deno.writeFileSync = function (path, data, options) {
  path = pathFromURL(path);

  // Convert data to Uint8Array if needed
  if (typeof data === "string") {
    data = new TextEncoder().encode(data);
  }

  const opts = options ? JSON.stringify(options) : null;
  return internal["fs.writeFileSync"](path, data, opts);
};

// https://docs.deno.com/api/deno/~/Deno.writeTextFileSync
globalThis.Deno.writeTextFileSync = function (path, text, options) {
  path = pathFromURL(path);

  const opts = options ? JSON.stringify(options) : null;
  return internal["fs.writeTextFileSync"](path, String(text), opts);
};

// https://docs.deno.com/api/deno/~/Deno.statSync
globalThis.Deno.statSync = function (path) {
  path = pathFromURL(path);

  const result = internal["fs.statSync"](path);
  return JSON.parse(result);
};

// https://docs.deno.com/api/deno/~/Deno.mkdirSync
globalThis.Deno.mkdirSync = function (path, options) {
  path = pathFromURL(path);

  const opts = options ? JSON.stringify(options) : null;
  return internal["fs.mkdirSync"](path, opts);
};

// https://docs.deno.com/api/deno/~/Deno.removeSync
globalThis.Deno.removeSync = function (path, options) {
  path = pathFromURL(path);

  const opts = options ? JSON.stringify(options) : null;
  return internal["fs.removeSync"](path, opts);
};

// https://docs.deno.com/api/deno/~/Deno.copyFileSync
globalThis.Deno.copyFileSync = function (fromPath, toPath) {
  fromPath = pathFromURL(fromPath);
  toPath = pathFromURL(toPath);

  return internal["fs.copyFileSync"](fromPath, toPath);
};

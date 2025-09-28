const internal = globalThis[Symbol.for("mnode.internal")];

function getEncoding(optOrCallback) {
  if (!optOrCallback || typeof optOrCallback === "function") {
    return null;
  }
  if (typeof optOrCallback === "string") {
    return optOrCallback;
  }
  return optOrCallback.encoding || null;
}

export function readFileSync(path, options) {
  const encoding = getEncoding(options);

  try {
    const content = internal.readFileSync(path);
    if (encoding && encoding !== "binary") {
      return content;
    }
    return content;
  } catch (err) {
    throw new Error(err);
  }
}

export function writeFileSync(path, data, _options) {
  try {
    internal.writeFileSync(path, data);
  } catch (err) {
    throw new Error(err);
  }
}

export function existsSync(path) {
  try {
    return internal.existsSync(path);
  } catch (_err) {
    return false;
  }
}

class Stats {
  constructor(
    dev,
    ino,
    mode,
    nlink,
    uid,
    gid,
    rdev,
    size,
    blksize,
    blocks,
    atimeMs,
    mtimeMs,
    ctimeMs,
    birthtimeMs,
    isFile,
    isDirectory,
    isSymbolicLink,
  ) {
    this.dev = dev;
    this.mode = mode;
    this.nlink = nlink;
    this.uid = uid;
    this.gid = gid;
    this.rdev = rdev;
    this.blksize = blksize;
    this.ino = ino;
    this.size = size;
    this.blocks = blocks;
    this.atimeMs = atimeMs;
    this.mtimeMs = mtimeMs;
    this.ctimeMs = ctimeMs;
    this.birthtimeMs = birthtimeMs;
    this._isFile = isFile;
    this._isDirectory = isDirectory;
    this._isSymbolicLink = isSymbolicLink;
  }

  isFile() {
    return this._isFile;
  }

  isDirectory() {
    return this._isDirectory;
  }

  isSymbolicLink() {
    return this._isSymbolicLink;
  }

  isBlockDevice() {
    return false;
  }

  isCharacterDevice() {
    return false;
  }

  isFIFO() {
    return false;
  }

  isSocket() {
    return false;
  }
}

export function statSync(path) {
  try {
    const result = JSON.parse(internal.statSync(path));
    return new Stats(
      result.dev,
      result.ino,
      result.mode,
      result.nlink,
      result.uid,
      result.gid,
      result.rdev,
      result.size,
      result.blksize,
      result.blocks,
      result.atimeMs,
      result.mtimeMs,
      result.ctimeMs,
      result.birthtimeMs,
      result.isFile,
      result.isDirectory,
      result.isSymbolicLink,
    );
  } catch (err) {
    throw new Error(err);
  }
}

export function readdirSync(path, _options) {
  try {
    return JSON.parse(internal.readdirSync(path));
  } catch (err) {
    throw new Error(err);
  }
}

export default {
  readFileSync,
  writeFileSync,
  existsSync,
  statSync,
  readdirSync,
};

// Base64 encoding/decoding
globalThis.btoa = function btoa(data) {
  if (typeof data !== "string") {
    throw new TypeError(
      "Failed to execute 'btoa': The string to be encoded contains characters outside of the Latin1 range.",
    );
  }

  const internal = globalThis[Symbol.for("mnode.internal")];
  return internal.encoding.btoa(data);
};

globalThis.atob = function atob(data) {
  if (typeof data !== "string") {
    throw new TypeError(
      "Failed to execute 'atob': 1 argument required, but only 0 present.",
    );
  }

  const internal = globalThis[Symbol.for("mnode.internal")];
  const result = internal.encoding.atob(data);

  // Check if result is an error (string starting with ERROR:)
  if (result.startsWith("ERROR: ")) {
    throw new DOMException(result.substring(7), "InvalidCharacterError");
  }

  return result;
};

// TextEncoder class
class TextEncoder {
  constructor() {
    this.encoding = "utf-8";
  }

  encode(input = "") {
    const str = String(input);
    const internal = globalThis[Symbol.for("mnode.internal")];
    const bytesJson = internal.encoding.encode(str);
    const bytes = JSON.parse(bytesJson);
    return new Uint8Array(bytes);
  }

  encodeInto(source, destination) {
    const str = String(source);
    const internal = globalThis[Symbol.for("mnode.internal")];
    const bytesJson = internal.encoding.encode(str);
    const bytes = JSON.parse(bytesJson);

    let written = 0;
    for (let i = 0; i < bytes.length && i < destination.length; i++) {
      destination[i] = bytes[i];
      written++;
    }

    return {
      read: str.length,
      written: written,
    };
  }
}

// TextDecoder class
class TextDecoder {
  #encoding;
  #fatal;
  #ignoreBOM;

  constructor(label = "utf-8", options = {}) {
    // Normalize encoding label
    const encoding = String(label).toLowerCase().replace(/[_-]/g, "");

    // Only support UTF-8 for now
    if (encoding !== "utf8" && encoding !== "unicode11utf8") {
      throw new RangeError(
        `The encoding label provided ('${label}') is invalid.`,
      );
    }

    this.#encoding = "utf-8";
    this.#fatal = options.fatal || false;
    this.#ignoreBOM = options.ignoreBOM || false;
  }

  get encoding() {
    return this.#encoding;
  }

  get fatal() {
    return this.#fatal;
  }

  get ignoreBOM() {
    return this.#ignoreBOM;
  }

  decode(input, _options = {}) {
    let bytes;

    if (input === undefined) {
      bytes = new Uint8Array(0);
    } else if (input instanceof ArrayBuffer) {
      bytes = new Uint8Array(input);
    } else if (ArrayBuffer.isView(input)) {
      bytes = new Uint8Array(input.buffer, input.byteOffset, input.byteLength);
    } else {
      throw new TypeError(
        "Failed to execute 'decode' on 'TextDecoder': The provided value is not of type '(ArrayBuffer or ArrayBufferView)'",
      );
    }

    const internal = globalThis[Symbol.for("mnode.internal")];
    const bytesArray = Array.from(bytes);
    const bytesJson = JSON.stringify(bytesArray);
    const result = internal.encoding.decode(bytesJson);

    if (result.startsWith("ERROR: ")) {
      if (this.#fatal) {
        throw new TypeError(result.substring(7));
      }
      // Non-fatal mode: return replacement characters (handled by Rust side)
      return "";
    }

    // Handle BOM (Byte Order Mark)
    if (!this.#ignoreBOM && result.charCodeAt(0) === 0xFEFF) {
      return result.slice(1);
    }

    return result;
  }
}

globalThis.TextEncoder = TextEncoder;
globalThis.TextDecoder = TextDecoder;

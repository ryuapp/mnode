class Response {
  #status;
  #statusText;
  #headers;
  #body;
  #bodyUsed = false;

  constructor(body, options = {}) {
    this.#status = options.status || 200;
    this.#statusText = options.statusText || "";
    this.#headers = new Headers(options.headers || {});
    this.#body = body;
  }

  get status() {
    return this.#status;
  }

  get statusText() {
    return this.#statusText;
  }

  get ok() {
    return this.#status >= 200 && this.#status < 300;
  }

  get headers() {
    return this.#headers;
  }

  get bodyUsed() {
    return this.#bodyUsed;
  }

  // deno-lint-ignore require-await
  async text() {
    if (this.#bodyUsed) {
      throw new TypeError("Body has already been consumed");
    }
    this.#bodyUsed = true;
    return this.#body;
  }

  async json() {
    const text = await this.text();
    return JSON.parse(text);
  }

  clone() {
    if (this.#bodyUsed) {
      throw new TypeError("Cannot clone a response that has been consumed");
    }
    return new Response(this.#body, {
      status: this.#status,
      statusText: this.#statusText,
      headers: this.#headers,
    });
  }
}

class Headers {
  #headers = {};

  constructor(init = {}) {
    if (init) {
      if (typeof init === "object") {
        for (const [key, value] of Object.entries(init)) {
          this.set(key, value);
        }
      }
    }
  }

  get(name) {
    return this.#headers[name.toLowerCase()] || null;
  }

  set(name, value) {
    this.#headers[name.toLowerCase()] = String(value);
  }

  has(name) {
    return name.toLowerCase() in this.#headers;
  }

  delete(name) {
    delete this.#headers[name.toLowerCase()];
  }

  forEach(callback) {
    for (const [key, value] of Object.entries(this.#headers)) {
      callback(value, key, this);
    }
  }

  entries() {
    return Object.entries(this.#headers)[Symbol.iterator]();
  }

  keys() {
    return Object.keys(this.#headers)[Symbol.iterator]();
  }

  values() {
    return Object.values(this.#headers)[Symbol.iterator]();
  }
}

async function fetch(url, options = {}) {
  const method = (options.method || "GET").toUpperCase();
  const headers = new Headers(options.headers || {});
  const body = options.body || "";

  // Convert headers to JSON string
  const headersObj = {};
  headers.forEach((value, key) => {
    headersObj[key] = value;
  });
  const headersJson = JSON.stringify(headersObj);

  // Start async fetch
  const internal = globalThis[Symbol.for("mnode.internal")];
  const taskId = internal.fetch.start(
    String(url),
    method,
    headersJson,
    String(body),
  );

  // Poll for result
  while (true) {
    const resultJson = internal.fetch.poll(taskId);
    if (resultJson) {
      const result = JSON.parse(resultJson);

      if (result.error) {
        throw new TypeError(`Failed to fetch: ${result.error}`);
      }

      // Create response
      return new Response(result.body, {
        status: result.status,
        statusText: "",
        headers: result.headers,
      });
    }

    // Yield to event loop
    await Promise.resolve();
  }
}

globalThis.fetch = fetch;
globalThis.Response = Response;
globalThis.Headers = Headers;

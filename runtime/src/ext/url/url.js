class URL {
  #href;
  #origin;
  #protocol;
  #username;
  #password;
  #host;
  #hostname;
  #port;
  #pathname;
  #search;
  #hash;

  constructor(url, base) {
    const internal = globalThis[Symbol.for("mnode.internal")];
    const parseURL = internal.url.parse;

    try {
      const result = JSON.parse(parseURL(url, base || ""));

      this.#href = result.href;
      this.#origin = result.origin;
      this.#protocol = result.protocol;
      this.#username = result.username;
      this.#password = result.password;
      this.#host = result.host;
      this.#hostname = result.hostname;
      this.#port = result.port;
      this.#pathname = result.pathname;
      this.#search = result.search;
      this.#hash = result.hash;
    } catch (_error) {
      throw new TypeError(`Invalid URL: ${url}`);
    }
  }

  get href() {
    return this.#href;
  }

  set href(value) {
    const internal = globalThis[Symbol.for("mnode.internal")];
    const parseURL = internal.url.parse;

    const result = JSON.parse(parseURL(String(value), ""));
    this.#href = result.href;
    this.#origin = result.origin;
    this.#protocol = result.protocol;
    this.#username = result.username;
    this.#password = result.password;
    this.#host = result.host;
    this.#hostname = result.hostname;
    this.#port = result.port;
    this.#pathname = result.pathname;
    this.#search = result.search;
    this.#hash = result.hash;
  }

  get origin() {
    return this.#origin;
  }

  get protocol() {
    return this.#protocol;
  }

  set protocol(value) {
    const internal = globalThis[Symbol.for("mnode.internal")];
    const setComponent = internal.url.setComponent;

    const result = JSON.parse(
      setComponent(this.#href, "protocol", String(value)),
    );
    this.#href = result.href;
    this.#protocol = result.protocol;
  }

  get username() {
    return this.#username;
  }

  set username(value) {
    const internal = globalThis[Symbol.for("mnode.internal")];
    const setComponent = internal.url.setComponent;

    const result = JSON.parse(
      setComponent(this.#href, "username", String(value)),
    );
    this.#href = result.href;
    this.#username = result.username;
  }

  get password() {
    return this.#password;
  }

  set password(value) {
    const internal = globalThis[Symbol.for("mnode.internal")];
    const setComponent = internal.url.setComponent;

    const result = JSON.parse(
      setComponent(this.#href, "password", String(value)),
    );
    this.#href = result.href;
    this.#password = result.password;
  }

  get host() {
    return this.#host;
  }

  set host(value) {
    const internal = globalThis[Symbol.for("mnode.internal")];
    const setComponent = internal.url.setComponent;

    const result = JSON.parse(
      setComponent(this.#href, "host", String(value)),
    );
    this.#href = result.href;
    this.#host = result.host;
    this.#hostname = result.hostname;
    this.#port = result.port;
  }

  get hostname() {
    return this.#hostname;
  }

  set hostname(value) {
    const internal = globalThis[Symbol.for("mnode.internal")];
    const setComponent = internal.url.setComponent;

    const result = JSON.parse(
      setComponent(this.#href, "hostname", String(value)),
    );
    this.#href = result.href;
    this.#host = result.host;
    this.#hostname = result.hostname;
  }

  get port() {
    return this.#port;
  }

  set port(value) {
    const internal = globalThis[Symbol.for("mnode.internal")];
    const setComponent = internal.url.setComponent;

    const result = JSON.parse(
      setComponent(this.#href, "port", String(value)),
    );
    this.#href = result.href;
    this.#host = result.host;
    this.#port = result.port;
  }

  get pathname() {
    return this.#pathname;
  }

  set pathname(value) {
    const internal = globalThis[Symbol.for("mnode.internal")];
    const setComponent = internal.url.setComponent;

    const result = JSON.parse(
      setComponent(this.#href, "pathname", String(value)),
    );
    this.#href = result.href;
    this.#pathname = result.pathname;
  }

  get search() {
    return this.#search;
  }

  set search(value) {
    const internal = globalThis[Symbol.for("mnode.internal")];
    const setComponent = internal.url.setComponent;

    const result = JSON.parse(
      setComponent(this.#href, "search", String(value)),
    );
    this.#href = result.href;
    this.#search = result.search;
  }

  get hash() {
    return this.#hash;
  }

  set hash(value) {
    const internal = globalThis[Symbol.for("mnode.internal")];
    const setComponent = internal.url.setComponent;

    const result = JSON.parse(
      setComponent(this.#href, "hash", String(value)),
    );
    this.#href = result.href;
    this.#hash = result.hash;
  }

  toString() {
    return this.#href;
  }

  toJSON() {
    return this.#href;
  }

  static parse(url, base) {
    try {
      return new URL(url, base);
    } catch {
      return null;
    }
  }

  static canParse(url, base) {
    try {
      new URL(url, base);
      return true;
    } catch {
      return false;
    }
  }
}

globalThis.URL = URL;

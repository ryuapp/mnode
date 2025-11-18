const __internal = globalThis[Symbol.for("mdeno.internal")];

class Navigator {
  constructor() {
    this.userAgent = "mdeno";
    this.platform = __internal.platform;
  }
}

globalThis.navigator = new Navigator();

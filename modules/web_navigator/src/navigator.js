class Navigator {
  constructor() {
    this.userAgent = "mdeno";
    this.platform = globalThis[Symbol.for("mdeno.internal")].platform;
  }
}

globalThis.navigator = new Navigator();

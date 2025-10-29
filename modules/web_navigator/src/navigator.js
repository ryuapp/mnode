class Navigator {
  constructor() {
    this.userAgent = "mnode";
    this.platform = globalThis[Symbol.for("mnode.internal")].platform;
  }
}

globalThis.navigator = new Navigator();

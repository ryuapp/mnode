class Navigator {
  constructor() {
    this.userAgent = "mdeno";
    this.platform = __internal.platform;
  }
}

globalThis.navigator = new Navigator();

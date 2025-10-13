import fs from "node:fs";
import process from "node:process";

// Navigator API
console.log("Navigator:", navigator.userAgent);

// URL API
const url = new URL("https://example.com/path?query=value#hash");
console.log("URL:", url.href);

// File System API
const isFile = fs.statSync("README.md").isFile();
console.log("README.md is file:", isFile);

// Process API
console.log("Process argv:", process.argv);
console.log(Response);
// Fetch API
console.log("\nTesting Fetch API...");
try {
  const promise = fetch("https://ryu.app");
  console.log("Fetch returns:", promise);

  const response = await promise;
  console.log("Response status:", response.status);
  console.log("Response ok:", response.ok);
  const text = await response.text();
  console.log("Response body length:", text.length, "bytes");
} catch (error) {
  console.error("Fetch error:", error.message);
}

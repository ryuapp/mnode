import fs from "node:fs";
import process from "node:process";

console.log(navigator);

const url = new URL("https://example.com");
console.log(url);
console.log(fs.statSync("README.md").isFile());
console.log(process.argv);

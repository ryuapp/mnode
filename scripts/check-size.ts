/**
 * Binary size checker script
 * Checks the size of compiled binaries using Deno.statSync
 *
 * Usage:
 *   deno scripts/check-size.ts
 */

/**
 * Format bytes to human-readable size
 */
function formatBytes(bytes: number): string {
  const units = ["B", "KB", "MB", "GB", "TB"];
  let size = bytes;
  let unitIndex = 0;

  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex++;
  }

  return `${size.toFixed(2)} ${units[unitIndex]}`;
}

/**
 * Check binary size using Deno.statSync
 */
function checkBinarySize(binaryPath: string): string {
  try {
    const stat = Deno.statSync(binaryPath);
    const sizeFormatted = formatBytes(stat.size);
    return sizeFormatted;
  } catch (_err) {
    throw new Error(`Failed to get size for binary at path: ${binaryPath}`);
  }
}

/**
 * Get platform-specific binary extension
 */
function getBinaryExt(): ".exe" | "" {
  const osType = Deno.build.os;
  return osType === "windows" ? ".exe" : "";
}

function main() {
  const target = Deno.args[0];
  const ext = getBinaryExt();
  const binaryPath = `target/${target ? `${target}/` : ""}release/mdeno${ext}`;
  const sizeFormatted = checkBinarySize(binaryPath);

  // Get the actual byte size for JSON output
  const stat = Deno.statSync(binaryPath);

  const output = {
    target: target || "unknown",
    size: sizeFormatted,
    size_bytes: stat.size,
  };

  console.log(JSON.stringify(output));
}

main();

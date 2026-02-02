export async function readFileAsUint8Array(file: Blob): Promise<Uint8Array> {
  const buffer = await file.arrayBuffer();
  return new Uint8Array(buffer);
}

export function bytesToHexPreview(bytes: Uint8Array, maxBytes: number): string {
  if (maxBytes <= 0) return "";
  const size = Math.min(bytes.length, maxBytes);
  const out: string[] = [];
  for (let i = 0; i < size; i += 1) {
    const value = bytes[i];
    if (value === undefined) break;
    out.push(value.toString(16).padStart(2, "0"));
  }
  return out.join(" ");
}

export function formatBytes(bytes: number): string {
  if (bytes < 0) return `${bytes} B`;
  if (bytes < 1024) return `${bytes} B`;
  const kib = bytes / 1024;
  if (kib < 1024) return `${kib.toFixed(2)} KiB`;
  const mib = kib / 1024;
  if (mib < 1024) return `${mib.toFixed(2)} MiB`;
  const gib = mib / 1024;
  return `${gib.toFixed(2)} GiB`;
}

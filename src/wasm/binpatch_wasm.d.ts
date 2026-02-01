declare module "binpatch-wasm" {
  export default function init(input?: unknown): Promise<void>;

  export function verify(profile_toml: string, firmware: Uint8Array): unknown;

  export function patch(
    profile_toml: string,
    firmware: Uint8Array,
    ssid: string,
    psk: string,
    select_index?: number,
  ): Uint8Array;
}

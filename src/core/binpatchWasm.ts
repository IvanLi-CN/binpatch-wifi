export type WasmVerifyOutput = {
  valid_offsets: number[];
};

type BinpatchWasmModule =
  typeof import("../../crates/binpatch-wasm/pkg/binpatch_wasm.js");

let modulePromise: Promise<BinpatchWasmModule> | null = null;

async function loadBinpatchWasmModule(): Promise<BinpatchWasmModule> {
  if (!modulePromise) {
    modulePromise = (async () => {
      const mod: BinpatchWasmModule = await import(
        "../../crates/binpatch-wasm/pkg/binpatch_wasm.js"
      );
      await mod.default();
      return mod;
    })();
  }
  return modulePromise;
}

export async function wasmVerify(
  profileToml: string,
  firmware: Uint8Array,
): Promise<WasmVerifyOutput> {
  const mod = await loadBinpatchWasmModule();
  return mod.verify(profileToml, firmware) as WasmVerifyOutput;
}

export async function wasmPatch(params: {
  profileToml: string;
  firmware: Uint8Array;
  ssid: string;
  psk: string;
  selectIndex?: number;
}): Promise<Uint8Array> {
  const mod = await loadBinpatchWasmModule();
  return mod.patch(
    params.profileToml,
    params.firmware,
    params.ssid,
    params.psk,
    params.selectIndex,
  );
}

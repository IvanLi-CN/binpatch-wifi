import { useEffect, useMemo, useState } from "react";
import {
  type WasmVerifyOutput,
  wasmPatch,
  wasmVerify,
} from "../core/binpatchWasm";
import {
  bytesToHexPreview,
  formatBytes,
  readFileAsUint8Array,
} from "../core/bytes";
import {
  DEFAULT_PROFILE_NAME,
  DEFAULT_PROFILE_TOML,
} from "../core/defaultProfile";
import "./app.css";

type FirmwareState =
  | { kind: "idle" }
  | { kind: "loading"; fileName: string }
  | { kind: "loaded"; fileName: string; bytes: Uint8Array }
  | { kind: "error"; fileName?: string; message: string };

type VerifyState =
  | { kind: "idle" }
  | { kind: "running" }
  | { kind: "done"; output: WasmVerifyOutput }
  | { kind: "error"; message: string };

type PatchState =
  | { kind: "idle" }
  | { kind: "running" }
  | { kind: "done"; bytes: Uint8Array }
  | { kind: "error"; message: string };

export function App() {
  const [firmware, setFirmware] = useState<FirmwareState>({ kind: "idle" });
  const [profileToml, setProfileToml] = useState<string>(DEFAULT_PROFILE_TOML);

  const [verify, setVerify] = useState<VerifyState>({ kind: "idle" });
  const [selectIndex, setSelectIndex] = useState<number | undefined>(undefined);

  const [ssid, setSsid] = useState("");
  const [psk, setPsk] = useState("");
  const [patch, setPatch] = useState<PatchState>({ kind: "idle" });

  const firmwareStats = useMemo(() => {
    if (firmware.kind !== "loaded") return null;
    return {
      size: firmware.bytes.byteLength,
      preview: bytesToHexPreview(firmware.bytes, 16),
    };
  }, [firmware]);

  const validOffsets = useMemo(() => {
    if (verify.kind !== "done") return null;
    return verify.output.valid_offsets;
  }, [verify]);

  const ssidBytes = useMemo(
    () => new TextEncoder().encode(ssid).byteLength,
    [ssid],
  );
  const pskBytes = useMemo(
    () => new TextEncoder().encode(psk).byteLength,
    [psk],
  );

  const download = useMemo(() => {
    if (patch.kind !== "done") return null;
    if (firmware.kind !== "loaded") return null;
    const url = URL.createObjectURL(
      new Blob([patch.bytes as unknown as BlobPart], {
        type: "application/octet-stream",
      }),
    );
    const name = firmware.fileName.replace(/(\.[^.]*)?$/, ".wifi$1");
    return { url, name };
  }, [patch, firmware]);

  useEffect(() => {
    if (!download) return;
    return () => URL.revokeObjectURL(download.url);
  }, [download]);

  return (
    <div className="app">
      <header className="app__header">
        <h1 className="app__title">binpatch-wifi</h1>
        <p className="app__subtitle">
          Browser-first Wi‑Fi config patcher (Rust core via WASM)
        </p>
      </header>

      <main className="app__main">
        <section className="card">
          <h2 className="card__title">Firmware</h2>
          <p className="card__hint">
            Upload a firmware binary; verification/patching is performed by Rust
            core (WASM).
          </p>
          <input
            type="file"
            className="file"
            accept=".bin,.elf,application/octet-stream"
            onChange={async (event) => {
              const file = event.currentTarget.files?.[0];
              if (!file) return;

              setFirmware({ kind: "loading", fileName: file.name });
              setVerify({ kind: "idle" });
              setPatch({ kind: "idle" });
              setSelectIndex(undefined);
              try {
                const bytes = await readFileAsUint8Array(file);
                setFirmware({ kind: "loaded", fileName: file.name, bytes });
              } catch (error) {
                const message =
                  error instanceof Error ? error.message : String(error);
                setFirmware({ kind: "error", fileName: file.name, message });
              }
            }}
          />

          {firmware.kind === "idle" && (
            <p className="status">No firmware loaded.</p>
          )}
          {firmware.kind === "loading" && (
            <p className="status">Loading: {firmware.fileName}</p>
          )}
          {firmware.kind === "error" && (
            <p className="status status--error">Failed: {firmware.message}</p>
          )}
          {firmware.kind === "loaded" && firmwareStats && (
            <div className="status">
              <div>
                <span className="label">File:</span> {firmware.fileName}
              </div>
              <div>
                <span className="label">Size:</span>{" "}
                {formatBytes(firmwareStats.size)} ({firmwareStats.size} bytes)
              </div>
              <div>
                <span className="label">Hex preview:</span>{" "}
                <code>{firmwareStats.preview}</code>
              </div>
            </div>
          )}
        </section>

        <section className="card">
          <h2 className="card__title">Profile</h2>
          <p className="card__hint">
            Default: <code>{DEFAULT_PROFILE_NAME}</code> (editable).
          </p>
          <textarea
            value={profileToml}
            spellCheck={false}
            rows={10}
            onChange={(event) => {
              setProfileToml(event.currentTarget.value);
              setVerify({ kind: "idle" });
              setPatch({ kind: "idle" });
              setSelectIndex(undefined);
            }}
          />
        </section>

        <section className="card">
          <h2 className="card__title">Verify</h2>
          <button
            type="button"
            disabled={firmware.kind !== "loaded" || verify.kind === "running"}
            onClick={async () => {
              if (firmware.kind !== "loaded") return;
              setVerify({ kind: "running" });
              setPatch({ kind: "idle" });
              try {
                const out = await wasmVerify(profileToml, firmware.bytes);
                setVerify({ kind: "done", output: out });
              } catch (error) {
                const message =
                  error instanceof Error ? error.message : String(error);
                setVerify({ kind: "error", message });
              }
            }}
          >
            {verify.kind === "running" ? "Verifying..." : "Verify"}
          </button>

          {verify.kind === "idle" && <p className="status">Not verified.</p>}
          {verify.kind === "error" && (
            <p className="status status--error">Failed: {verify.message}</p>
          )}
          {verify.kind === "done" && (
            <div className="status">
              <div>
                <span className="label">Valid matches:</span>{" "}
                {verify.output.valid_offsets.length}
              </div>
              {verify.output.valid_offsets.length > 0 && (
                <div>
                  <span className="label">Offsets:</span>{" "}
                  {verify.output.valid_offsets
                    .map((v) => `0x${v.toString(16).toUpperCase()}`)
                    .join(", ")}
                </div>
              )}
            </div>
          )}
        </section>

        <section className="card">
          <h2 className="card__title">Patch</h2>
          <p className="card__hint">
            PSK is never printed. Provide <code>select_index</code> when
            multiple matches exist.
          </p>

          <div className="row">
            <label className="row__label" htmlFor="ssid">
              SSID
            </label>
            <input
              id="ssid"
              className="row__input"
              value={ssid}
              onChange={(e) => setSsid(e.currentTarget.value)}
            />
          </div>

          <div className="row">
            <label className="row__label" htmlFor="psk">
              PSK
            </label>
            <input
              id="psk"
              className="row__input"
              value={psk}
              type="password"
              onChange={(e) => setPsk(e.currentTarget.value)}
            />
          </div>

          {validOffsets && validOffsets.length > 1 && (
            <div className="row">
              <label className="row__label" htmlFor="selectIndex">
                select_index
              </label>
              <select
                id="selectIndex"
                className="row__input"
                value={selectIndex ?? ""}
                onChange={(e) => {
                  const v = e.currentTarget.value;
                  if (v === "") setSelectIndex(undefined);
                  else setSelectIndex(Number(v));
                }}
              >
                <option value="">(required)</option>
                {validOffsets.map((offset, idx) => (
                  <option key={offset} value={idx}>
                    {idx} (0x{offset.toString(16).toUpperCase()})
                  </option>
                ))}
              </select>
            </div>
          )}

          <button
            type="button"
            disabled={
              firmware.kind !== "loaded" ||
              verify.kind !== "done" ||
              patch.kind === "running" ||
              ssid.length === 0 ||
              psk.length === 0 ||
              (validOffsets?.length ?? 0) === 0 ||
              ((validOffsets?.length ?? 0) > 1 && selectIndex === undefined)
            }
            onClick={async () => {
              if (firmware.kind !== "loaded") return;
              if (verify.kind !== "done") return;

              setPatch({ kind: "running" });
              try {
                const bytes = await wasmPatch({
                  profileToml,
                  firmware: firmware.bytes,
                  ssid,
                  psk,
                  selectIndex,
                });

                const postVerify = await wasmVerify(profileToml, bytes);
                if (postVerify.valid_offsets.length === 0) {
                  throw new Error("post-verify failed: no valid matches");
                }

                setPatch({ kind: "done", bytes });
              } catch (error) {
                const message =
                  error instanceof Error ? error.message : String(error);
                setPatch({ kind: "error", message });
              }
            }}
          >
            {patch.kind === "running" ? "Patching..." : "Patch"}
          </button>

          {patch.kind === "idle" && <p className="status">Not patched.</p>}
          {patch.kind === "error" && (
            <p className="status status--error">Failed: {patch.message}</p>
          )}
          {patch.kind === "done" && firmware.kind === "loaded" && (
            <div className="status">
              <div>
                <span className="label">Patched:</span> OK (ssid_bytes=
                {ssidBytes}, psk_bytes={pskBytes})
              </div>
              <div>
                {download && (
                  <a href={download.url} download={download.name}>
                    Download patched firmware
                  </a>
                )}
              </div>
            </div>
          )}
        </section>
      </main>
    </div>
  );
}

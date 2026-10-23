import React, { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

const ALGORITHMS = [
  { key: "md5", label: "MD5" },
  { key: "sha1", label: "SHA-1" },
  { key: "sha256", label: "SHA-256" },
  { key: "sha512", label: "SHA-512" },
];

function HashRow({ algo, filePath, onCopy, copiedField }) {
  const [hash, setHash] = useState(null);
  const [computing, setComputing] = useState(false);
  const [progress, setProgress] = useState(0);
  const [error, setError] = useState(null);
  const unlistenRef = useRef(null);

  const compute = useCallback(async () => {
    setComputing(true);
    setProgress(0);
    setError(null);
    setHash(null);

    const eventName = `checksum-progress-${algo.key}`;
    const unlisten = listen(eventName, (event) => {
      setProgress(event.payload);
    });
    unlistenRef.current = unlisten;

    try {
      const result = await invoke("compute_checksum", {
        filePath,
        algorithm: algo.key,
      });
      setHash(result.hash);
    } catch (err) {
      setError(err || "Failed to compute checksum");
    } finally {
      setComputing(false);
      const fn = await unlistenRef.current;
      if (fn) fn();
      unlistenRef.current = null;
    }
  }, [algo.key, filePath]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (unlistenRef.current) {
        unlistenRef.current.then((fn) => fn());
      }
    };
  }, []);

  return (
    <div
      className={`
        border rounded-xl p-3 sm:p-4 transition-all duration-300
        bg-surface-secondary border-separator
        ${computing ? "border-accent/50 bg-accent/10" : ""}
        ${error ? "border-danger/40 bg-danger/10" : ""}
      `}
    >
      <div className="flex items-center justify-between mb-2 sm:mb-3">
        <div className="flex items-center gap-1 sm:gap-2">
          <div className={`p-1 sm:p-1.5 rounded-lg ${hash ? 'bg-accent/10' : 'bg-surface-tertiary'} transition-colors`}>
            <i className={`ri-shield-check-line ${hash ? 'text-accent' : 'text-fg-secondary'} text-base sm:text-lg`}></i>
          </div>
          <span className="text-xs sm:text-sm font-bold text-fg uppercase tracking-wider">
            {algo.label}
          </span>
        </div>
        <div className="flex items-center gap-1 sm:gap-2">
          {hash && !computing && (
            <button
              onClick={() => onCopy(hash, algo.key)}
              className={`
                flex items-center gap-1 sm:gap-1.5 px-2 sm:px-3 py-1 sm:py-1.5 rounded-lg text-xs font-bold transition-all duration-300 border
                ${
                  copiedField === algo.key
                    ? "bg-success/10 text-success border-success/40"
                    : "bg-surface text-fg-secondary border-separator hover:text-fg hover:bg-surface-tertiary"
                }
              `}
              title="Copy to clipboard"
            >
              {copiedField === algo.key ? (
                <>
                  <i className="ri-check-line text-sm sm:text-base"></i>
                  <span className="hidden sm:inline">Copied</span>
                  <span className="sm:hidden">Copied</span>
                </>
              ) : (
                <>
                  <i className="ri-file-copy-line text-sm sm:text-base"></i>
                  <span className="hidden sm:inline">Copy</span>
                  <span className="sm:hidden">Copy</span>
                </>
              )}
            </button>
          )}
          <button
            onClick={compute}
            disabled={computing}
            className={`
              flex items-center gap-1 sm:gap-1.5 px-2 sm:px-3 py-1 sm:py-1.5 rounded-lg text-xs font-bold border transition-all duration-300
              ${
                computing
                  ? "bg-surface-tertiary text-fg-secondary border-separator cursor-wait"
                  : "bg-accent text-accent-fg border-accent hover:bg-accent-hover"
              }
            `}
            title={`Compute ${algo.label} checksum`}
          >
            {computing ? (
              <>
                <div className="w-2.5 h-2.5 sm:w-3 sm:h-3 border border-accent/40 border-t-accent rounded-full animate-spin"></div>
                <span className="font-semibold text-xs sm:text-xs">{progress}%</span>
              </>
            ) : (
              <>
                <i className="ri-play-fill text-sm sm:text-base"></i>
                <span className="hidden sm:inline">{hash ? "Recompute" : "Compute"}</span>
                <span className="sm:hidden">{hash ? "Re" : "Go"}</span>
              </>
            )}
          </button>
        </div>
      </div>

      {/* Progress bar while computing */}
      {computing && (
        <div className="w-full h-1.5 sm:h-2 bg-surface-tertiary rounded-full overflow-hidden mb-2 sm:mb-3">
          <div
            className="h-full bg-accent rounded-full transition-all duration-500 ease-out"
            style={{ width: `${progress}%` }}
          ></div>
        </div>
      )}

      {/* Hash value or placeholder */}
      {hash && !computing && (
        <div className="bg-surface rounded-lg p-2 sm:p-3 border border-separator overflow-hidden">
          <p
            className="text-xs sm:text-sm font-mono text-fg break-all leading-tight sm:leading-relaxed select-all"
            title={hash}
          >
            {hash}
          </p>
        </div>
      )}
      {error && !computing && (
        <div className="bg-danger/10 rounded-lg p-2 sm:p-3 border border-danger/40">
          <p className="text-xs sm:text-sm text-danger font-medium">{error}</p>
        </div>
      )}
      {!hash && !computing && !error && (
        <div className="text-center py-3 sm:py-4 bg-surface rounded-lg border border-dashed border-separator">
          <i className="ri-cursor-line text-xl sm:text-2xl text-fg-tertiary mb-1 sm:mb-2"></i>
          <p className="text-xs sm:text-sm text-fg-tertiary">
            Click Compute to generate {algo.label} hash
          </p>
        </div>
      )}
    </div>
  );
}

function ChecksumModal({ filePath, onClose }) {
  const [copiedField, setCopiedField] = useState(null);
  const fileName = filePath.split("/").filter(Boolean).pop() || filePath;

  const copyToClipboard = async (value, field) => {
    try {
      await navigator.clipboard.writeText(value);
      setCopiedField(field);
      setTimeout(() => setCopiedField(null), 2000);
    } catch {
      const textarea = document.createElement("textarea");
      textarea.value = value;
      textarea.style.position = "fixed";
      textarea.style.opacity = "0";
      document.body.appendChild(textarea);
      textarea.select();
      document.execCommand("copy");
      document.body.removeChild(textarea);
      setCopiedField(field);
      setTimeout(() => setCopiedField(null), 2000);
    }
  };

  return (
    <div
      className="fixed inset-0 z-[100] flex items-center justify-center bg-black/30 p-4"
      onClick={onClose}
    >
      <div
        className="bg-surface rounded-xl shadow-window border border-separator w-full max-w-md sm:max-w-lg mx-4 max-h-[90vh] flex flex-col overflow-hidden"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="shrink-0 px-4 sm:px-6 py-3 sm:py-5 bg-surface-secondary border-b border-separator">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2 sm:gap-4">
              <div className="w-8 h-8 sm:w-12 sm:h-12 bg-surface-tertiary rounded-xl flex items-center justify-center shadow-sm">
                <i className="ri-shield-check-line text-lg sm:text-2xl text-fg"></i>
              </div>
              <div className="min-w-0">
                <h2 className="text-base sm:text-xl font-bold text-fg">Checksum</h2>
                <p
                  className="text-xs sm:text-sm text-accent truncate max-w-[150px] sm:max-w-[300px]"
                  title={filePath}
                >
                  {fileName}
                </p>
              </div>
            </div>
            <button
              onClick={onClose}
              className="w-8 h-8 sm:w-10 sm:h-10 rounded-xl flex items-center justify-center text-fg-secondary hover:text-fg hover:bg-surface-tertiary transition-colors"
              title="Close"
            >
              <i className="ri-close-line text-base sm:text-xl"></i>
            </button>
          </div>
        </div>

        {/* Body */}
        <div className="flex-1 overflow-y-auto px-3 sm:px-6 py-3 sm:py-5 space-y-2 sm:space-y-4 bg-surface-secondary">
          {ALGORITHMS.map((algo) => (
            <HashRow
              key={algo.key}
              algo={algo}
              filePath={filePath}
              onCopy={copyToClipboard}
              copiedField={copiedField}
            />
          ))}
        </div>

        {/* Footer */}
        <div className="shrink-0 px-4 sm:px-6 py-3 sm:py-4 bg-accent/10 border-t border-accent/40">
          <div className="flex items-center justify-between gap-2 sm:gap-4">
            <p className="text-xs sm:text-sm text-fg-secondary flex-1">
              <i className="ri-information-line mr-1 sm:mr-1.5 text-accent"></i>
              <span className="hidden sm:inline">Click Compute on the algorithm you need</span>
              <span className="sm:hidden">Click Compute on algorithm</span>
            </p>
            <button
              onClick={onClose}
              className="px-4 sm:px-5 py-2 sm:py-2.5 bg-surface-secondary text-fg rounded-xl text-xs sm:text-sm font-bold hover:bg-surface-tertiary transition-all duration-300 shadow-sm hover:shadow-sm shrink-0"
            >
              Close
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

export default ChecksumModal;

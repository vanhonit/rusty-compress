import React, { useState, useEffect } from "react";
import { listen, emit } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

function ProgressWindow({ name }) {
  const [progress, setProgress] = useState(0);
  const [filesExtracted, setFilesExtracted] = useState(0);
  const [totalFiles, setTotalFiles] = useState(0);
  const [isComplete, setIsComplete] = useState(false);
  const [error, setError] = useState(null);
  const urlParams = new URLSearchParams(window.location.search);
  const fileName = urlParams.get("fileName");

  useEffect(() => {
    const unlistenProgress = listen("extract-progress", (event) => {
      const progressData = event.payload;
      if (
        typeof progressData === "object" &&
        progressData.files !== undefined
      ) {
        setProgress(progressData.percentage || 0);
        setFilesExtracted(progressData.files || 0);
        if (progressData.total !== undefined) {
          setTotalFiles(progressData.total);
        }
      } else {
        setProgress(progressData || 0);
      }
    });

    const unlistenComplete = listen("extract-complete", () => {
      setIsComplete(true);
      setProgress(100);
    });

    const unlistenError = listen("extract-error", (event) => {
      setError(event.payload);
    });

    emit(`${name}://ready`, null);

    return () => {
      Promise.all([unlistenProgress, unlistenComplete, unlistenError]).then(
        (unsubs) => {
          unsubs.forEach((unsub) => unsub?.());
        },
      );
    };
  }, [name]);

  return (
    <div className="w-full h-full flex flex-col bg-surface-secondary p-3 sm:p-4 overflow-hidden">
      {/* Header */}
      <div className="shrink-0 mb-3 sm:mb-4">
        <div className="flex items-center gap-2 sm:gap-3 mb-2 sm:mb-3">
          <div className="p-1.5 sm:p-2 bg-accent/10 rounded-xl">
            <i className="ri-download-cloud-line text-accent text-xl sm:text-2xl"></i>
          </div>
          <div className="flex-1 min-w-0">
            <h1 className="text-xs sm:text-sm font-bold text-fg mb-0.5 sm:mb-1">Extracting Archive</h1>
            <p className="text-xs text-fg-tertiary truncate">{fileName}</p>
          </div>
        </div>
      </div>

      {/* Error State */}
      {error && (
        <div className="shrink-0 mb-3 sm:mb-4 p-2 sm:p-3 bg-danger/10 border border-danger/30 rounded-xl">
          <div className="flex items-start gap-2">
            <div className="w-5 h-5 bg-danger/10 rounded-full flex items-center justify-center flex-shrink-0 mt-0.5">
              <i className="ri-error-warning-line text-sm text-danger"></i>
            </div>
            <div className="flex-1">
              <p className="text-danger text-xs sm:text-sm font-bold mb-1">Extraction Failed</p>
              <p className="text-danger text-xs sm:text-sm">{error}</p>
              {error.toLowerCase().includes("password") && (
                <p className="text-danger text-xs sm:text-sm mt-2">
                  Please try again with the correct password.
                </p>
              )}
            </div>
          </div>
        </div>
      )}

      {/* Progress Bar */}
      <div className="shrink-0 mb-3 sm:mb-4">
        <div className="flex items-center justify-between mb-1.5 sm:mb-2">
          <span className="text-xs sm:text-sm font-bold text-fg-tertiary">Progress</span>
          <span className="text-xs sm:text-sm font-bold text-accent">
            {Math.round(progress)}%
          </span>
        </div>
        <div className="w-full bg-surface-tertiary rounded-full h-2 sm:h-3 overflow-hidden">
          <div
            className="bg-accent h-full transition-all duration-500 ease-out rounded-full"
            style={{ width: `${progress}%` }}
          ></div>
        </div>
      </div>

      {/* Files Counter */}
      <div className="shrink-0 text-xs sm:text-sm text-fg-tertiary mb-3 sm:mb-4 flex items-center gap-1.5 sm:gap-2">
        <i className="ri-file-zip-line text-accent text-sm sm:text-base"></i>
        <span className="font-bold text-fg-tertiary">{filesExtracted}</span>
        <span>
          {totalFiles > 0 ? `/ ${totalFiles} ` : ''}files extracted
        </span>
      </div>

      {/* Status Messages */}
      <div className="flex-1 space-y-2 sm:space-y-3 overflow-y-auto">
        {!isComplete && !error && (
          <div className="flex items-start gap-2 sm:gap-3 p-2 sm:p-3 bg-accent/10 rounded-xl border border-accent/40">
            <div className="flex-shrink-0 mt-0.5">
              <div className="w-2.5 h-2.5 sm:w-3 sm:h-3 bg-accent rounded-full"></div>
            </div>
            <div>
              <p className="text-xs sm:text-sm text-accent font-medium">Extracting files...</p>
              <p className="text-xs text-accent mt-1">Please wait while we process your files</p>
            </div>
          </div>
        )}

        {isComplete && (
          <div className="flex items-start gap-2 sm:gap-3 p-3 sm:p-4 bg-success/15 rounded-xl border border-success/30 shadow-sm">
            <i className="ri-checkbox-circle-line text-success text-xl sm:text-2xl flex-shrink-0"></i>
            <div className="flex-1">
              <p className="text-xs sm:text-sm text-success font-bold mb-1">Extraction complete!</p>
              <p className="text-xs text-success">All files have been extracted successfully</p>
            </div>
          </div>
        )}

        {error && (
          <div className="flex items-start gap-2 sm:gap-3 p-2 sm:p-3 bg-danger/10 rounded-xl border border-danger/20">
            <i className="ri-error-warning-line text-danger text-xl sm:text-2xl flex-shrink-0"></i>
            <div>
              <p className="text-xs sm:text-sm text-danger font-bold">Extraction failed</p>
              <p className="text-xs text-danger mt-1">Please check the error message above</p>
            </div>
          </div>
        )}
      </div>

      {/* Footer Info */}
      <div className="shrink-0 mt-2 sm:mt-4 border-t border-separator pt-2 sm:pt-3">
        {!isComplete && !error && (
          <p className="text-xs text-fg-secondary">Extracting to your selected destination...</p>
        )}

        {(isComplete || error) && (
          <div className="flex items-center justify-between">
            <p className="text-xs text-fg-secondary">
              {isComplete ? 'Extraction completed successfully!' : 'Extraction failed.'}
            </p>
            <button
              onClick={() => getCurrentWindow().close()}
              className="px-3 py-1.5 bg-accent hover:bg-accent-hover text-accent-fg text-xs font-medium rounded-mac transition-colors duration-200"
            >
              Close
            </button>
          </div>
        )}
      </div>
    </div>
  );
}

export default ProgressWindow;

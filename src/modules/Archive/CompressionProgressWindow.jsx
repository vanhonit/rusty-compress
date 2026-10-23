import React, { useState, useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";

const CompressionProgressWindow = () => {
  const [progress, setProgress] = useState({
    percentage: 0,
    files_processed: 0,
    current_file: "",
    compression_ratio: 0.0
  });
  const [isComplete, setIsComplete] = useState(false);
  const [error, setError] = useState(null);
  const [estimatedTime, setEstimatedTime] = useState(null);

  useEffect(() => {
    // Listen for compression progress events
    const unlistenProgress = window.__TAURI__.event.listen(
      "compress-progress",
      (event) => {
        const data = event.payload;
        setProgress({
          percentage: data.percentage || 0,
          files_processed: data.files_processed || 0,
          current_file: data.current_file || "",
          compression_ratio: data.compression_ratio || 0.0
        });
      }
    );

    // Listen for completion event
    const unlistenComplete = window.__TAURI__.event.listen(
      "compress-complete",
      () => {
        setIsComplete(true);
        // Auto-close after 2 seconds
        setTimeout(() => {
          getCurrentWindow().close();
        }, 2000);
      }
    );

    // Listen for error event
    const unlistenError = window.__TAURI__.event.listen(
      "compress-error",
      (event) => {
        const errorMsg = event.payload || "Compression failed";
        let userMessage = "An error occurred during compression";

        // Provide user-friendly error messages
        if (typeof errorMsg === 'string') {
          if (errorMsg.includes("not found") || errorMsg.includes("File not found")) {
            userMessage = "One or more source files could not be found. Please check file paths.";
          } else if (errorMsg.includes("permission") || errorMsg.includes("access")) {
            userMessage = "Permission denied. Please check file permissions.";
          } else if (errorMsg.includes("disk") || errorMsg.includes("space")) {
            userMessage = "Insufficient disk space. Please free up disk space.";
          } else if (errorMsg.includes("7z")) {
            userMessage = "7-Zip tool is not available. Please install 7-Zip (p7zip) to continue.";
          } else if (errorMsg.includes("cancel")) {
            userMessage = "Compression was cancelled by the user.";
          } else {
            userMessage = errorMsg; // Use the error message directly
          }
        }

        setError(userMessage);
      }
    );

    return () => {
      unlistenProgress.then((fn) => fn());
      unlistenComplete.then((fn) => fn());
      unlistenError.then((fn) => fn());
    };
  }, []);

  return (
    <div className="min-h-screen bg-surface-secondary flex items-center justify-center p-4">
      <div className="bg-surface rounded-xl shadow-window border border-separator max-w-md w-full overflow-hidden">
        {/* Header */}
        <div className="bg-surface-secondary px-6 py-4 border-b border-separator">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 bg-surface-tertiary rounded-lg flex items-center justify-center">
              <i className="ri-archive-add-line text-2xl text-accent"></i>
            </div>
            <div>
              <h1 className="text-[13px] font-semibold text-fg">Creating Archive</h1>
              <p className="text-xs text-accent">Compressing files</p>
            </div>
          </div>
        </div>

        {/* Content */}
        <div className="p-6 space-y-6">
          {/* Error Display */}
          {error ? (
            <div className="p-4 bg-danger/10 border border-danger/40 rounded-xl">
              <div className="flex items-start gap-3">
                <div className="w-8 h-8 bg-danger/10 rounded-full flex items-center justify-center flex-shrink-0">
                  <i className="ri-close-circle-line text-xl text-danger"></i>
                </div>
                <div className="flex-1">
                  <h3 className="text-sm font-semibold text-danger mb-1">Error</h3>
                  <p className="text-sm text-danger">{error}</p>
                </div>
              </div>
            </div>
          ) : isComplete ? (
            /* Success State */
            <div className="p-4 bg-success/10 border border-success/40 rounded-xl">
              <div className="flex items-start gap-3">
                <div className="w-8 h-8 bg-success/10 rounded-full flex items-center justify-center flex-shrink-0">
                  <i className="ri-check-line text-xl text-success"></i>
                </div>
                <div className="flex-1">
                  <h3 className="text-sm font-semibold text-success mb-1">Complete</h3>
                  <p className="text-sm text-success">Archive created successfully!</p>
                </div>
              </div>
            </div>
          ) : (
            /* Progress State */
            <>
              {/* Progress Bar */}
              <div className="space-y-3">
                <div className="flex items-center justify-between">
                  <span className="text-sm font-medium text-fg">Progress</span>
                  <span className="text-sm font-bold text-accent">
                    {Math.round(progress.percentage)}%
                  </span>
                </div>
                <div className="w-full h-3 bg-surface-tertiary rounded-full overflow-hidden">
                  <div
                    className="h-full bg-accent rounded-full transition-all duration-300 ease-out"
                    style={{ width: `${progress.percentage}%` }}
                  ></div>
                </div>
              </div>

              {/* Current File */}
              {progress.current_file && (
                <div className="p-3 bg-accent/10 rounded-xl border border-accent/40">
                  <div className="flex items-center gap-2">
                    <i className="ri-file-line text-accent"></i>
                    <span className="text-sm font-medium text-fg truncate flex-1">
                      {progress.current_file}
                    </span>
                  </div>
                </div>
              )}

              {/* Stats Grid */}
              <div className="grid grid-cols-2 gap-3">
                <div className="p-3 bg-accent/10 rounded-xl border border-accent/40">
                  <div className="flex items-center gap-2 mb-1">
                    <i className="ri-file-list-3-line text-accent"></i>
                    <span className="text-xs font-semibold text-fg-secondary uppercase">
                      Files Processed
                    </span>
                  </div>
                  <p className="text-lg font-bold text-accent">
                    {progress.files_processed}
                  </p>
                </div>

                <div className="p-3 bg-accent/10 rounded-xl border border-accent/40">
                  <div className="flex items-center gap-2 mb-1">
                    <i className="ri-shrink-line text-accent"></i>
                    <span className="text-xs font-semibold text-fg-secondary uppercase">
                      Compression Ratio
                    </span>
                  </div>
                  <p className="text-lg font-bold text-accent">
                    {progress.compression_ratio > 0
                      ? `${progress.compression_ratio.toFixed(1)}%`
                      : 'N/A'}
                  </p>
                </div>
              </div>

              {/* Loading Animation */}
              <div className="flex items-center justify-center py-4">
                <div className="relative">
                  <div className="absolute inset-0 bg-accent/10 rounded-full opacity-20"></div>
                  <div className="relative w-12 h-12 bg-accent/10 rounded-full flex items-center justify-center">
                    <i className="ri-refresh-line text-xl text-accent animate-spin"></i>
                  </div>
                </div>
              </div>
            </>
          )}
        </div>

        {/* Footer */}
        <div className="bg-surface-secondary border-t border-separator px-6 py-4">
          <div className="flex justify-center">
            {!isComplete && !error && (
              <p className="text-xs text-fg-secondary text-center">
                Compressing files... Please wait
              </p>
            )}
            {isComplete && (
              <p className="text-xs text-fg-secondary text-center">
                Closing automatically...
              </p>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};

export default CompressionProgressWindow;
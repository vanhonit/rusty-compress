import React, { useState, useEffect } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";

// Validation utilities
const validateArchiveName = (name) => {
  if (!name || name.trim() === "") {
    return { valid: false, error: "Archive name is required" };
  }

  // Check for invalid characters
  const invalidChars = /[<>:"|?*]/;
  if (invalidChars.test(name)) {
    return { valid: false, error: "Archive name contains invalid characters: <>:\"|?*" };
  }

  // Check for reserved names
  const reservedNames = ["CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9", "LPT1", "LPT2", "LPT3"];
  const baseName = name.split('.')[0].toUpperCase();
  if (reservedNames.includes(baseName)) {
    return { valid: false, error: `"${baseName}" is a reserved name in Windows` };
  }

  // Check for reasonable length
  if (name.length > 255) {
    return { valid: false, error: "Archive name is too long (max 255 characters)" };
  }

  return { valid: true, error: null };
};

const CreateArchiveModal = ({
  selectedFiles,
  currentPath,
  onClose,
  onSuccess
}) => {
  const [archiveName, setArchiveName] = useState("");
  const [destinationFolder, setDestinationFolder] = useState(currentPath || "");
  const [archiveFormat, setArchiveFormat] = useState("zip");
  const [compressionLevel, setCompressionLevel] = useState(6);
  const [isCreating, setIsCreating] = useState(false);
  const [error, setError] = useState(null);

  // Format descriptions
  const formatOptions = [
    {
      value: "zip",
      label: "ZIP",
      description: "Universal format, good compression, widely supported",
      extension: ".zip"
    },
    {
      value: "tar",
      label: "TAR",
      description: "Unix format, no compression, preserves permissions",
      extension: ".tar"
    },
    {
      value: "tar.gz",
      label: "TAR.GZ",
      description: "Gzipped TAR, good compression, Linux standard",
      extension: ".tar.gz"
    },
    {
      value: "tar.bz2",
      label: "TAR.BZ2",
      description: "Bzip2-compressed TAR, better compression than gzip",
      extension: ".tar.bz2"
    },
    {
      value: "7z",
      label: "7-Zip",
      description: "High compression, multiple algorithms, Windows/Linux",
      extension: ".7z"
    }
  ];

  // Compression level presets
  const compressionPresets = [
    { value: 0, label: "None", description: "No compression, fastest" },
    { value: 1, label: "Fast", description: "Best speed, good compression" },
    { value: 6, label: "Normal", description: "Balanced speed and compression" },
    { value: 9, label: "Maximum", description: "Best compression, slowest" }
  ];

  // Set default archive name when modal opens
  useEffect(() => {
    if (selectedFiles.length === 1) {
      // Use selected file/folder name as default
      const name = selectedFiles[0].split('/').pop().replace(/\.[^.]+$/, '');
      setArchiveName(name);
    } else {
      // Use timestamp for multiple files
      const timestamp = new Date().toISOString().slice(0, 19).replace(/[:T]/g, '-');
      setArchiveName(`archive-${timestamp}`);
    }
  }, [selectedFiles]);

  // Auto-update archive name when format changes
  useEffect(() => {
    const currentFormat = formatOptions.find(f => f.value === archiveFormat);
    if (currentFormat && archiveName && !archiveName.endsWith(currentFormat.extension)) {
      // Remove any existing extension and add the correct one
      const baseName = archiveName.replace(/\.(zip|tar|tar\.gz|tar\.bz2|7z)$/i, '');
      setArchiveName(baseName + currentFormat.extension);
    }
  }, [archiveFormat]);

  const handleBrowseDestination = async () => {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Select Destination Folder"
    });

    if (selected) {
      setDestinationFolder(selected);
    }
  };

  const handleCreateArchive = async () => {
    // Validate archive name
    const nameValidation = validateArchiveName(archiveName);
    if (!nameValidation.valid) {
      setError(nameValidation.error);
      return;
    }

    // Validate destination folder
    if (!destinationFolder.trim()) {
      setError("Destination folder is required");
      return;
    }

    // Validate file selection
    if (selectedFiles.length === 0) {
      setError("No files selected for archiving");
      return;
    }

    // Check if files exist
    try {
      for (const file of selectedFiles) {
        const exists = await invoke("path_exists", { path: file });
        if (!exists) {
          setError(`File not found: ${file.split('/').pop()}`);
          return;
        }
      }
    } catch (err) {
      setError(`Failed to validate files: ${err.message || err}`);
      return;
    }

    setIsCreating(true);
    setError(null);

    try {
      // Map format to backend ArchiveFormat enum
      const formatMap = {
        "zip": "zip",
        "tar": "tar",
        "tar.gz": "tar",
        "tar.bz2": "tar",
        "7z": "7z"
      };

      const backendFormat = formatMap[archiveFormat] || "zip";
      const outputPath = `${destinationFolder}/${archiveName}`;

      // Check for overwriting existing file
      const archiveExists = await invoke("path_exists", { path: outputPath });
      if (archiveExists) {
        setError(`Archive "${archiveName}" already exists in the destination folder. Please choose a different name or delete the existing file.`);
        return;
      }

      await invoke("create_archive", {
        files: selectedFiles,
        outputPath,
        archiveFormat: backendFormat,
        compressionLevel
      });

      // Success callback
      if (onSuccess) {
        onSuccess();
      }

      // Close modal
      onClose();
    } catch (err) {
      console.error("Archive creation error:", err);

      // Extract detailed error information
      let errorMessage = "Failed to create archive";
      let technicalDetails = "";

      if (typeof err === 'string') {
        errorMessage = err;
        technicalDetails = err;
      } else if (err.message) {
        errorMessage = err.message;
        technicalDetails = err.message;
      } else if (typeof err === 'object') {
        technicalDetails = JSON.stringify(err, null, 2);
        errorMessage = "Unknown error occurred";
      }

      // Add technical details to the error message for debugging
      const detailedError = `${errorMessage}\n\nTechnical details:\n${technicalDetails}`;

      setError(detailedError);
    } finally {
      setIsCreating(false);
    }
  };

  const handleBackdropClick = (e) => {
    if (e.target === e.currentTarget && !isCreating) {
      onClose();
    }
  };

  const currentFormat = formatOptions.find(f => f.value === archiveFormat);
  const currentPreset = compressionPresets.find(p => p.value === compressionLevel);

  return (
    <div
      className="fixed inset-0 bg-black/30 flex items-center justify-center z-[100] p-4"
      onClick={handleBackdropClick}
    >
      <div className="bg-surface rounded-xl shadow-window border border-separator max-w-xl w-full max-h-[90vh] flex flex-col">
        {/* Header */}
        <div className="bg-surface-secondary px-4 py-3 flex-shrink-0 border-b border-separator">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <div className="w-8 h-8 bg-surface-tertiary rounded-lg flex items-center justify-center">
                <i className="ri-archive-add-line text-xl text-fg"></i>
              </div>
              <div>
                <h2 className="text-base font-bold text-fg">Create Archive</h2>
                <p className="text-xs text-accent">
                  {selectedFiles.length} file{selectedFiles.length !== 1 ? 's' : ''}
                </p>
              </div>
            </div>
            <button
              onClick={onClose}
              disabled={isCreating}
              className="text-fg-secondary hover:text-fg transition-colors p-1 hover:bg-surface-tertiary rounded-lg disabled:opacity-50 disabled:cursor-not-allowed"
              title="Close"
            >
              <i className="ri-close-line text-lg"></i>
            </button>
          </div>
        </div>

        {/* Content */}
        <div className="p-4 space-y-4 overflow-y-auto flex-1">
          {/* Error Display */}
          {error && (
            <div className="p-2.5 bg-danger/10 border border-danger/40 rounded-lg flex flex-col gap-2">
              <div className="flex items-start gap-2">
                <div className="w-5 h-5 bg-danger/10 rounded-full flex items-center justify-center flex-shrink-0 mt-0.5">
                  <i className="ri-alert-line text-xs text-danger"></i>
                </div>
                <div className="flex-1 overflow-auto max-h-32">
                  <p className="text-xs text-danger whitespace-pre-wrap font-mono">{error}</p>
                </div>
                <button
                  onClick={() => setError(null)}
                  className="text-danger hover:text-danger transition-colors p-1 hover:bg-danger/10 rounded"
                >
                  <i className="ri-close-line text-sm"></i>
                </button>
              </div>
              <div className="text-xs text-danger bg-danger/10 p-2 rounded">
                <strong>Tip:</strong> Check the browser console (F12) for more detailed error information
              </div>
            </div>
          )}

          {/* Archive Name */}
          <div>
            <label className="text-xs font-semibold text-fg-secondary uppercase tracking-wider mb-1 block">
              Archive Name
            </label>
            <input
              type="text"
              value={archiveName}
              onChange={(e) => setArchiveName(e.target.value)}
              disabled={isCreating}
              className="w-full px-3 py-2 rounded-lg border border-separator focus:border-accent focus:ring-2 focus:ring-accent/30 outline-none transition-all disabled:bg-surface-secondary disabled:cursor-not-allowed"
              placeholder="Enter archive name"
            />
          </div>

          {/* Format Selection */}
          <div>
            <label className="text-xs font-semibold text-fg-secondary uppercase tracking-wider mb-1 block">
              Archive Format
            </label>
            <div className="flex flex-wrap gap-2">
              {formatOptions.map((format) => (
                <button
                  key={format.value}
                  onClick={() => !isCreating && setArchiveFormat(format.value)}
                  disabled={isCreating}
                  className={`
                    relative group px-3 py-2 rounded-lg border transition-all duration-300 flex-1 min-w-[100px]
                    ${archiveFormat === format.value
                      ? 'border-accent bg-accent/10'
                      : 'border-separator bg-surface hover:bg-surface-tertiary'
                    }
                    disabled:opacity-50 disabled:cursor-not-allowed
                  `}
                >
                  <div className="flex items-center justify-center gap-1.5">
                    <i className={`ri-file-${format.value === '7z' ? 'zip' : format.value.replace('.', '-')}-line text-lg text-accent`}></i>
                    <span className="font-semibold text-sm text-fg">{format.label}</span>
                    {archiveFormat === format.value && (
                      <i className="ri-check-line text-accent text-sm"></i>
                    )}
                  </div>
                </button>
              ))}
            </div>
          </div>

          {/* Compression Level */}
          <div>
            <label className="text-xs font-semibold text-fg-secondary uppercase tracking-wider mb-1 block">
              Compression Level
            </label>
            <div className="flex items-center gap-3">
              <input
                type="range"
                min="0"
                max="9"
                value={compressionLevel}
                onChange={(e) => setCompressionLevel(parseInt(e.target.value))}
                disabled={isCreating}
                className="flex-1 h-2 bg-surface-tertiary rounded-lg appearance-none cursor-pointer accent-accent disabled:opacity-50 disabled:cursor-not-allowed"
              />
              <div className="w-10 h-10 bg-accent/10 rounded-lg flex items-center justify-center">
                <span className="text-base font-bold text-accent">{compressionLevel}</span>
              </div>
            </div>
            <div className="flex gap-1.5 mt-2">
              {compressionPresets.map((preset) => (
                <button
                  key={preset.value}
                  onClick={() => !isCreating && setCompressionLevel(preset.value)}
                  disabled={isCreating}
                  className={`
                    flex-1 px-2 py-1.5 rounded-lg border text-xs font-medium transition-all duration-300
                    ${compressionLevel === preset.value
                      ? 'border-accent bg-accent text-accent-fg'
                      : 'border-separator text-fg hover:bg-surface-tertiary'
                    }
                    disabled:opacity-50 disabled:cursor-not-allowed
                  `}
                  title={preset.description}
                >
                  {preset.label}
                </button>
              ))}
            </div>
          </div>

          {/* Destination Folder */}
          <div>
            <label className="text-xs font-semibold text-fg-secondary uppercase tracking-wider mb-1 block">
              Destination Folder
            </label>
            <div className="flex gap-2">
              <input
                type="text"
                value={destinationFolder}
                onChange={(e) => setDestinationFolder(e.target.value)}
                disabled={isCreating}
                className="flex-1 px-3 py-2 rounded-lg border border-separator focus:border-accent focus:ring-2 focus:ring-accent/30 outline-none transition-all disabled:bg-surface-secondary disabled:cursor-not-allowed"
                placeholder="Select destination folder"
              />
              <button
                onClick={handleBrowseDestination}
                disabled={isCreating}
                className="px-3 py-2 bg-surface-secondary hover:bg-surface-tertiary text-fg rounded-lg border border-separator transition-all disabled:opacity-50 disabled:cursor-not-allowed"
                title="Browse"
              >
                <i className="ri-folder-open-line text-base"></i>
              </button>
            </div>
          </div>

          {/* File List Preview */}
          <div>
            <label className="text-xs font-semibold text-fg-secondary uppercase tracking-wider mb-1 block">
              Files to Archive ({selectedFiles.length})
            </label>
            <div className="max-h-24 overflow-y-auto bg-surface-secondary rounded-lg border border-separator p-2 space-y-1">
              {selectedFiles.slice(0, 5).map((file, index) => (
                <div key={index} className="flex items-center gap-2 text-xs text-fg truncate">
                  <i className="ri-file-line text-fg-tertiary flex-shrink-0 text-sm"></i>
                  <span className="truncate">{file}</span>
                </div>
              ))}
              {selectedFiles.length > 5 && (
                <div className="text-xs text-fg-secondary text-center py-1">
                  ... and {selectedFiles.length - 5} more files
                </div>
              )}
            </div>
          </div>
        </div>

        {/* Footer */}
        <div className="bg-surface-secondary border-t border-separator px-4 py-3 flex-shrink-0">
          <div className="flex justify-end gap-2">
            <button
              onClick={onClose}
              disabled={isCreating}
              className="px-4 py-2 bg-surface border border-separator text-fg rounded-lg font-medium text-sm hover:bg-surface-secondary hover:border-separator transition-all disabled:opacity-50 disabled:cursor-not-allowed"
            >
              Cancel
            </button>
            <button
              onClick={handleCreateArchive}
              disabled={isCreating}
              className="px-4 py-2 bg-accent text-accent-fg rounded-lg font-medium text-sm hover:bg-accent/15 hover:shadow-sm transition-all disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1.5"
            >
              {isCreating ? (
                <>
                  <i className="ri-loader-4-line animate-spin text-sm"></i>
                  Creating...
                </>
              ) : (
                <>
                  <i className="ri-archive-add-line text-sm"></i>
                  Create Archive
                </>
              )}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};

export default CreateArchiveModal;
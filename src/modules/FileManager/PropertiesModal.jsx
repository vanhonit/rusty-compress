import React from "react";

// Utility to format file size
const formatFileSize = (bytes) => {
  if (!bytes || bytes === 0) return "0 bytes";

  const units = ["bytes", "KB", "MB", "GB", "TB"];
  let size = bytes;
  let unitIndex = 0;

  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex++;
  }

  return `${size.toFixed(2)} ${units[unitIndex]}`;
};

// Format date
const formatDate = (dateString) => {
  if (!dateString) return "Unknown";
  try {
    const date = new Date(dateString);
    return date.toLocaleString();
  } catch {
    return dateString;
  }
};

const PropertiesModal = ({ file, currentArchive, onClose }) => {
  const fileExtension = file.name.split(".").pop().toLowerCase();
  const isArchive = currentArchive && !file.is_dir;

  const handleBackdropClick = (e) => {
    if (e.target === e.currentTarget) {
      onClose();
    }
  };

  return (
    <div
      className="fixed inset-0 bg-black/30 flex items-center justify-center z-[100] p-4"
      onClick={handleBackdropClick}
    >
      <div className="bg-surface rounded-xl shadow-window border border-separator max-w-lg w-full overflow-hidden">
        {/* Header */}
        <div className="bg-surface-secondary px-6 py-4 border-b border-separator">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className="w-10 h-10 bg-surface-tertiary rounded-lg flex items-center justify-center">
                {file.is_dir ? (
                  <i className="ri-folder-fill text-2xl text-fg"></i>
                ) : (
                  <i className="ri-file-line text-2xl text-fg"></i>
                )}
              </div>
              <div>
                <h2 className="text-lg font-bold text-fg">File Properties</h2>
                <p className="text-xs text-accent">View file details</p>
              </div>
            </div>
            <button
              onClick={onClose}
              className="text-fg-secondary hover:text-fg transition-colors p-1 hover:bg-surface-tertiary rounded-lg"
              title="Close"
            >
              <i className="ri-close-line text-xl"></i>
            </button>
          </div>
        </div>

        {/* Content */}
        <div className="p-6">
          {/* File Name */}
          <div className="mb-4">
            <label className="text-xs font-semibold text-fg-secondary uppercase tracking-wider mb-1 block">
              Name
            </label>
            <div className="bg-accent/10 rounded-lg p-3 border border-accent/40">
              <p className="text-sm font-medium text-fg break-words">
                {file.name}
              </p>
            </div>
          </div>

          {/* Properties Grid */}
          <div className="grid grid-cols-2 gap-4">
            {/* Type */}
            <div>
              <label className="text-xs font-semibold text-fg-secondary uppercase tracking-wider mb-1 block">
                Type
              </label>
              <div className="bg-surface-secondary rounded-lg p-3 border border-separator">
                <p className="text-sm font-medium text-fg">
                  {file.is_dir ? "Folder" : fileExtension.toUpperCase() + " File"}
                </p>
              </div>
            </div>

            {/* Size */}
            <div>
              <label className="text-xs font-semibold text-fg-secondary uppercase tracking-wider mb-1 block">
                Size
              </label>
              <div className="bg-surface-secondary rounded-lg p-3 border border-separator">
                <p className="text-sm font-medium text-fg">
                  {file.is_dir ? "—" : formatFileSize(file.size)}
                </p>
              </div>
            </div>

            {/* Last Modified */}
            <div className="col-span-2">
              <label className="text-xs font-semibold text-fg-secondary uppercase tracking-wider mb-1 block">
                Last Modified
              </label>
              <div className="bg-surface-secondary rounded-lg p-3 border border-separator">
                <p className="text-sm font-medium text-fg">
                  {formatDate(file.last_modified)}
                </p>
              </div>
            </div>

            {/* Path */}
            <div className="col-span-2">
              <label className="text-xs font-semibold text-fg-secondary uppercase tracking-wider mb-1 block">
                Path
              </label>
              <div className="bg-surface-secondary rounded-lg p-3 border border-separator">
                <p className="text-sm font-medium text-fg break-all">
                  {file.path}
                </p>
              </div>
            </div>

            {/* MIME Type (if available) */}
            {file.mimetype && (
              <div className="col-span-2">
                <label className="text-xs font-semibold text-fg-secondary uppercase tracking-wider mb-1 block">
                  MIME Type
                </label>
                <div className="bg-surface-secondary rounded-lg p-3 border border-separator">
                  <p className="text-sm font-medium text-fg break-all">
                    {file.mimetype}
                  </p>
                </div>
              </div>
            )}

            {/* Archive Source (if in archive) */}
            {isArchive && (
              <div className="col-span-2">
                <label className="text-xs font-semibold text-fg-secondary uppercase tracking-wider mb-1 block">
                  Archive Source
                </label>
                <div className="bg-accent/10 rounded-lg p-3 border border-accent/40">
                  <p className="text-sm font-medium text-fg break-all">
                    {currentArchive}
                  </p>
                </div>
              </div>
            )}
          </div>

          {/* Additional Info */}
          {file.is_dir && (
            <div className="mt-4 p-3 bg-accent/10 rounded-lg border border-accent/40">
              <div className="flex items-start gap-2">
                <i className="ri-information-line text-accent text-lg mt-0.5"></i>
                <div>
                  <p className="text-sm font-medium text-accent">
                    Directory
                  </p>
                  <p className="text-xs text-accent mt-1">
                    This is a folder containing other files and subdirectories.
                  </p>
                </div>
              </div>
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="bg-surface-secondary border-t border-separator px-6 py-4">
          <div className="flex justify-end">
            <button
              onClick={onClose}
              className="px-6 py-2.5 bg-accent text-accent-fg rounded-xl font-medium text-sm hover:bg-accent/15 hover:shadow-sm transition-all duration-300"
            >
              Close
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};

export default PropertiesModal;
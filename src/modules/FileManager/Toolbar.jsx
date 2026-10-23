import React, { useState } from "react";

// ── Reusable toolbar button ────────────────────────────────────────────────
function ToolbarButton({
  onClick,
  disabled,
  title,
  icon,
  label,
  badge,
  shortcut,
}) {
  return (
    <button
      onClick={onClick}
      disabled={disabled}
      title={title}
      className={`
        relative flex flex-col items-center justify-center gap-0.5 px-2 sm:px-2.5 py-1.5 sm:py-2 rounded-lg min-w-[40px] transition-all duration-150 select-none
        ${
          disabled
            ? "text-fg-tertiary cursor-not-allowed"
            : "text-fg-secondary hover:bg-surface-tertiary hover:text-fg rounded-mac cursor-pointer"
        }
      `}
    >
      <div className="relative">
        <i className={`${icon} text-lg sm:text-xl leading-none`}></i>
        {badge != null && badge > 0 && (
          <span className="absolute -top-1.5 -right-2 min-w-[14px] px-1 bg-accent text-accent-fg text-[9px] font-bold rounded-full leading-[14px] text-center shadow-sm">
            {badge > 99 ? "99+" : badge}
          </span>
        )}
      </div>
      <span className="text-[9px] sm:text-[10px] font-medium leading-none whitespace-nowrap">
        {label}
      </span>
      {shortcut && !disabled && (
        <kbd className="absolute -top-1 -right-1 px-1 bg-surface-tertiary text-fg text-[8px] font-bold rounded shadow-sm leading-[11px]">
          {shortcut}
        </kbd>
      )}
    </button>
  );
}

// ── Vertical divider between button groups ────────────────────────────────
function ToolbarDivider() {
  return (
    <div className="w-px h-7 sm:h-8 bg-separator mx-0.5 sm:mx-1 flex-shrink-0" />
  );
}

// ── Toolbar ───────────────────────────────────────────────────────────────
function Toolbar({
  onExtract,
  onExtractHere,

  onOpen,
  onChecksum,
  onCreateArchive,
  onSelectAll,
  onDeselectAll,
  selectedCount = 0,
  totalCount = 0,
  isInArchive = false,
  hasArchivePath = false,
  disabled = false,
  onSearchChange,
  searchQuery = "",
}) {
  const [localSearchQuery, setLocalSearchQuery] = useState(searchQuery);

  const allSelected = totalCount > 0 && selectedCount === totalCount;
  const hasSelection = selectedCount > 0;
  const hasSingleSelection = selectedCount === 1;
  const canExtract = isInArchive && !disabled;
  const canExtractHere = isInArchive && !disabled;

  const canOpen = hasSingleSelection && !disabled;
  const canChecksum = hasArchivePath && !disabled;
  const canCreateArchive = hasSelection && !isInArchive && !disabled;
  const canSelectAll = totalCount > 0 && !allSelected && !disabled;
  const canDeselectAll = hasSelection && !disabled;
  const hasSearch = searchQuery && searchQuery.trim().length > 0;

  return (
    <div className="bg-surface-secondary px-2 sm:px-3 py-1 sm:py-1.5 border-b border-separator">
      <div className="flex items-center gap-0.5 sm:gap-1">
        {/* ── Group 1: Open ──────────────────────────────────────── */}
        <ToolbarButton
          onClick={onOpen}
          disabled={!canOpen}
          title={
            !hasSingleSelection
              ? "Select a single file or folder to open"
              : "Open selected item"
          }
          icon="ri-arrow-right-s-line"
          label="Open"
          shortcut={hasSingleSelection ? "↵" : null}
        />

        <ToolbarDivider />

        {/* ── Group 2: Extract ───────────────────────────────────── */}
        <ToolbarButton
          onClick={onExtract}
          disabled={!canExtract}
          title={
            !isInArchive
              ? "Available only when browsing archives"
              : hasSelection
                ? "Extract selected files — choose destination folder"
                : "Extract all files — choose destination folder"
          }
          icon="ri-file-zip-fill"
          label="Extract…"
          badge={canExtract && hasSelection ? selectedCount : null}
        />

        <ToolbarButton
          onClick={onExtractHere}
          disabled={!canExtractHere}
          title={
            !isInArchive
              ? "Available only when browsing archives"
              : hasSelection
                ? "Extract selected files to the archive's own folder"
                : "Extract all files to the archive's own folder"
          }
          icon="ri-folder-download-fill"
          label="Extract Here"
          badge={canExtractHere && hasSelection ? selectedCount : null}
        />

        <ToolbarDivider />

        {/* ── Group 4: Archive utilities ─────────────────────────── */}
        <ToolbarButton
          onClick={onChecksum}
          disabled={!canChecksum}
          title={
            !hasArchivePath
              ? "Open an archive file to compute checksum"
              : "Compute checksum for this archive"
          }
          icon="ri-shield-check-fill"
          label="Checksum"
        />

        <ToolbarButton
          onClick={onCreateArchive}
          disabled={!canCreateArchive}
          title={
            !hasSelection
              ? "Select files to create archive"
              : isInArchive
                ? "Cannot create archive from archive contents"
                : "Create archive from selected files (⌘N)"
          }
          icon="ri-archive-2-fill"
          label="Archive"
          badge={canCreateArchive ? selectedCount : null}
          shortcut={canCreateArchive ? "⌘N" : null}
        />

        {/* ── Search — takes remaining space ─────────────────────── */}
        <div className="flex-1 mx-1.5 sm:mx-2 min-w-0">
          <div className="relative group">
            <div className="absolute inset-y-0 left-0 pl-2 sm:pl-2.5 flex items-center pointer-events-none">
              <i className="ri-search-line text-fg-secondary text-sm group-focus-within:text-fg transition-colors"></i>
            </div>
            <input
              type="text"
              placeholder="Search files…"
              value={localSearchQuery}
              onChange={(e) => {
                const v = e.target.value;
                setLocalSearchQuery(v);
                onSearchChange && onSearchChange(v);
              }}
              className="w-full pl-7 sm:pl-8 pr-7 sm:pr-8 py-1.5 rounded-lg bg-surface-tertiary border border-separator text-fg text-xs sm:text-sm placeholder:text-fg-tertiary focus:outline-none focus:bg-surface-tertiary focus:border-separator transition-all"
            />
            {hasSearch && (
              <button
                onClick={() => {
                  setLocalSearchQuery("");
                  onSearchChange && onSearchChange("");
                }}
                className="absolute inset-y-0 right-0 pr-2 flex items-center text-fg-secondary hover:text-fg transition-colors"
                title="Clear search"
              >
                <i className="ri-close-line text-sm"></i>
              </button>
            )}
          </div>
        </div>

        {/* ── Selection control ──────────────────────────────────── */}
        {!hasSelection ? (
          <ToolbarButton
            onClick={onSelectAll}
            disabled={!canSelectAll}
            title={
              allSelected
                ? "All files already selected"
                : totalCount === 0
                  ? "No files to select"
                  : "Select all files"
            }
            icon="ri-checkbox-multiple-line"
            label="Select All"
          />
        ) : (
          <ToolbarButton
            onClick={onDeselectAll}
            disabled={!canDeselectAll}
            title="Clear selection"
            icon="ri-close-circle-line"
            label="Deselect"
          />
        )}
      </div>
    </div>
  );
}

export default Toolbar;

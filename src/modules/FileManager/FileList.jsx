import React from "react";
import FileItem from "./FileItem";

const FileList = ({
  files,
  selectedFiles,
  onOpenFile,
  onSelect,
  isLoading,
  currentArchive,
  archivePassword,
  onShowProperties,
  onCreateArchive,
}) => {
  const selectedCount = selectedFiles?.length || 0;
  const totalCount = files?.length || 0;

  // Loading skeleton
  const LoadingSkeleton = () => (
    <div className="bg-surface-secondary">
      {[...Array(8)].map((_, index) => (
        <div
          key={index}
          className="flex items-center px-4 sm:px-6 py-2 sm:py-4 border-b border-separator"
        >
          <div className="w-4 h-4 sm:w-5 sm:h-5 bg-surface-tertiary rounded mr-3 sm:mr-4"></div>
          <div className="w-5 h-5 sm:w-6 sm:h-6 bg-surface-tertiary rounded mr-3 sm:mr-4"></div>
          <div className="flex-1 flex gap-2 sm:gap-4">
            <div className="w-1/3 h-3 sm:h-4 bg-surface-tertiary rounded"></div>
            <div className="w-20 sm:w-24 h-3 sm:h-4 bg-surface-tertiary rounded"></div>
            <div className="w-16 sm:w-20 h-3 sm:h-4 bg-surface-tertiary rounded"></div>
            <div className="w-12 sm:w-16 h-3 sm:h-4 bg-surface-tertiary rounded"></div>
          </div>
        </div>
      ))}
    </div>
  );

  // Empty state
  const EmptyState = () => (
    <div className="flex flex-col items-center justify-center py-12 sm:py-20 px-4 bg-surface-secondary">
      <div className="mb-4 sm:mb-6 relative">
        <div className="absolute inset-0 bg-accent/10 rounded-full opacity-20"></div>
        <div className="relative w-16 h-16 sm:w-20 sm:h-20 bg-accent/10 rounded-full flex items-center justify-center shadow-sm">
          <i className="ri-folder-open-line text-3xl sm:text-4xl text-accent"></i>
        </div>
      </div>
      <h3 className="text-lg sm:text-xl font-bold text-fg mb-2 sm:mb-3">
        No Files Found
      </h3>
      <p className="text-xs sm:text-sm text-fg-secondary text-center max-w-sm sm:max-w-md leading-relaxed">
        This directory is empty. Navigate to a different location or open an
        archive to view its contents.
      </p>
      <div className="mt-4 sm:mt-6 flex flex-col sm:flex-row gap-2 sm:gap-3">
        <div className="flex items-center gap-2 px-3 sm:px-4 py-1.5 sm:py-2 bg-accent/10 text-accent rounded-lg text-xs sm:text-sm">
          <i className="ri-folder-5-line text-sm sm:text-base"></i>
          <span>Browse directories</span>
        </div>
        <div className="flex items-center gap-2 px-3 sm:px-4 py-1.5 sm:py-2 bg-accent/10 text-accent rounded-lg text-xs sm:text-sm">
          <i className="ri-archive-2-line text-sm sm:text-base"></i>
          <span>Open archive</span>
        </div>
      </div>
    </div>
  );

  return (
    <div className="bg-surface flex flex-col h-full overflow-hidden">
      {/* Table Header */}
      <div className="shrink-0 sticky top-0 z-10 bg-accent/10 border-b-2 border-accent/40 shadow-sm">
        <div className="flex items-center px-4 sm:px-6 py-2 sm:py-4">
          {/* Checkbox Column */}
          <div className="w-5 mr-3 sm:mr-4">
            <div className="w-5 h-5"></div>
          </div>

          {/* Icon Column */}
          <div className="w-5 sm:w-6 mr-3 sm:mr-4"></div>

          {/* Name Column */}
          <div className="flex-1 flex items-center gap-1 sm:gap-2 min-w-0">
            <span className="text-xs font-bold text-fg uppercase tracking-wider">
              Name
            </span>
            <i className="ri-arrow-up-down-line text-accent text-sm cursor-pointer hover:text-accent transition-colors"></i>
          </div>

          {/* Date Modified Column */}
          <div className="w-28 sm:w-40 hidden md:block">
            <span className="text-xs font-bold text-fg uppercase tracking-wider">
              Date Modified
            </span>
          </div>

          {/* Type Column */}
          <div className="w-24 sm:w-32 hidden lg:block">
            <span className="text-xs font-bold text-fg uppercase tracking-wider">
              Type
            </span>
          </div>

          {/* Size Column */}
          <div className="w-20 sm:w-28 text-right">
            <span className="text-xs font-bold text-fg uppercase tracking-wider">
              Size
            </span>
          </div>
        </div>
      </div>

      {/* File List Container with Custom Scrollbar */}
      <div
        className="flex-1 overflow-y-auto overflow-x-hidden"
        style={{
          scrollbarWidth: "thin",
          scrollbarColor: "rgb(var(--fg-2) / 0.35) transparent",
        }}
      >
        {/* Loading State */}
        {isLoading && <LoadingSkeleton />}

        {/* File Items */}
        {!isLoading && totalCount > 0 && (
          <div>
            {files.map((file, index) => (
              <FileItem
                key={`${file.path}-${index}`}
                file={file}
                index={index}
                selected={selectedFiles.some(
                  (selectedFile) => selectedFile.path === file.path,
                )}
                onOpenFile={onOpenFile}
                onSelect={onSelect}
                currentArchive={currentArchive}
                archivePassword={archivePassword}
                onShowProperties={onShowProperties}
                onCreateArchive={onCreateArchive}
              />
            ))}
          </div>
        )}

        {/* Empty State */}
        {!isLoading && totalCount === 0 && <EmptyState />}
      </div>
    </div>
  );
};

export default FileList;

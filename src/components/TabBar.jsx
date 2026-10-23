import React from 'react';
import Tab from './Tab';
import { useTabContext } from '../contexts/TabContext';
import { homeDir } from '@tauri-apps/api/path';

/**
 * TabBar Component
 * Displays a horizontal bar with all tabs and a new tab button
 */
const TabBar = () => {
  const { tabs, activeTabId, switchTab, closeTab, addTab, updateTab } = useTabContext();

  /**
   * Handle creating a new tab
   * Creates a new home directory tab
   */
  const handleNewTab = async () => {
    // Create a new tab with home directory
    const newTabId = addTab({
      title: 'Home',
      type: 'filesystem',
      currentPath: '',
      currentArchive: '',
      archivePath: '',
      files: [],
      selectedFiles: [],
      searchQuery: '',
      isLoading: true,
      error: null,
    });

    // Load home directory for the new tab
    try {
      const homeDirPath = await homeDir();
      // Import invoke here to avoid circular dependencies
      const { invoke } = await import('@tauri-apps/api/core');
      const result = await invoke('read_directory', { path: homeDirPath });

      updateTab(newTabId, {
        files: result,
        currentPath: homeDirPath,
        currentArchive: '',
        archivePath: '',
        selectedFiles: [],
        searchQuery: '',
        title: 'Home',
        type: 'filesystem',
        isLoading: false,
        error: null,
      });
    } catch (err) {
      console.error('Failed to load home directory for new tab:', err);
      updateTab(newTabId, {
        error: `Failed to load home directory: ${err.message || err}`,
        isLoading: false,
      });
    }
  };

  /**
   * Handle tab switching
   */
  const handleTabClick = (tabId) => {
    switchTab(tabId);
  };

  /**
   * Handle tab closing
   */
  const handleTabClose = (tabId) => {
    closeTab(tabId);
  };

  /**
   * Determine if a tab can be closed
   * Don't allow closing the last tab
   */
  const canCloseTab = (tabId) => {
    return tabs.length > 1;
  };

  return (
    <div className="tab-bar">
      <div className="flex items-center gap-0.5 overflow-x-auto px-1 pt-1">
        {/* Render all tabs */}
        {tabs.map((tab) => (
          <Tab
            key={tab.id}
            id={tab.id}
            title={tab.title}
            type={tab.type}
            isActive={tab.id === activeTabId}
            onClick={handleTabClick}
            onClose={handleTabClose}
            canClose={canCloseTab(tab.id)}
          />
        ))}

        {/* New Tab Button */}
        <button
          className="
            flex items-center justify-center w-8 h-8 rounded-lg ml-1 mr-1
            text-fg-secondary hover:text-fg hover:bg-surface-tertiary
            select-none
          "
          onClick={handleNewTab}
          title="New tab (Cmd+T)"
          aria-label="New tab"
        >
          <i className="ri-add-line text-lg"></i>
        </button>
      </div>
    </div>
  );
};

export default TabBar;
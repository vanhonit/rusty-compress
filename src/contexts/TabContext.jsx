import React, { createContext, useContext, useState, useCallback } from 'react';

// Create the TabContext
const TabContext = createContext(null);

/**
 * Tab Provider Component
 * Manages global tab state and provides tab manipulation functions
 */
export const TabProvider = ({ children }) => {
  const [tabs, setTabs] = useState([]);
  const [activeTabId, setActiveTabId] = useState(null);

  /**
   * Generate a unique tab ID
   */
  const generateTabId = useCallback(() => {
    return `tab-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  }, []);

  /**
   * Create a new tab and add it to the tabs array
   * @param {Object} tabData - Initial tab data (title, type, path, etc.)
   * @returns {string} The ID of the newly created tab
   */
  const addTab = useCallback((tabData) => {
    const newTab = {
      id: generateTabId(),
      title: tabData.title || 'New Tab',
      type: tabData.type || 'filesystem',
      currentPath: tabData.currentPath || '',
      currentArchive: tabData.currentArchive || '',
      archivePath: tabData.archivePath || '',
      archivePassword: tabData.archivePassword || null,
      files: tabData.files || [],
      selectedFiles: tabData.selectedFiles || [],
      searchQuery: tabData.searchQuery || '',
      isLoading: tabData.isLoading !== undefined ? tabData.isLoading : false,
      error: tabData.error || null,
    };

    setTabs(prevTabs => [...prevTabs, newTab]);
    setActiveTabId(newTab.id);

    return newTab.id;
  }, [generateTabId]);

  /**
   * Close a tab and switch to an adjacent tab
   * @param {string} tabId - The ID of the tab to close
   */
  const closeTab = useCallback((tabId) => {
    setTabs(prevTabs => {
      // Don't allow closing the last tab
      if (prevTabs.length <= 1) {
        return prevTabs;
      }

      const tabToDelete = prevTabs.find(t => t.id === tabId);
      if (!tabToDelete) return prevTabs;

      const newTabs = prevTabs.filter(t => t.id !== tabId);

      // If we're closing the active tab, switch to another tab
      if (activeTabId === tabId) {
        const deletedIndex = prevTabs.findIndex(t => t.id === tabId);
        // Prefer the tab to the left, or the tab to the right if there's no left tab
        const newIndex = Math.max(0, deletedIndex - 1);
        setActiveTabId(newTabs[newIndex].id);
      }

      return newTabs;
    });
  }, [activeTabId]);

  /**
   * Switch to a different tab
   * @param {string} tabId - The ID of the tab to switch to
   */
  const switchTab = useCallback((tabId) => {
    const tabExists = tabs.some(t => t.id === tabId);
    if (tabExists) {
      setActiveTabId(tabId);
    }
  }, [tabs]);

  /**
   * Update a specific tab's state
   * @param {string} tabId - The ID of the tab to update
   * @param {Object} updates - Partial tab data to update
   */
  const updateTab = useCallback((tabId, updates) => {
    setTabs(prevTabs =>
      prevTabs.map(tab =>
        tab.id === tabId
          ? { ...tab, ...updates }
          : tab
      )
    );
  }, []);

  /**
   * Get the currently active tab object
   */
  const activeTab = tabs.find(t => t.id === activeTabId) || null;

  const value = {
    tabs,
    activeTabId,
    activeTab,
    addTab,
    closeTab,
    switchTab,
    updateTab,
  };

  return (
    <TabContext.Provider value={value}>
      {children}
    </TabContext.Provider>
  );
};

/**
 * Custom hook to use the TabContext
 * @throws {Error} If used outside of TabProvider
 */
export const useTabContext = () => {
  const context = useContext(TabContext);
  if (!context) {
    throw new Error('useTabContext must be used within a TabProvider');
  }
  return context;
};

export default TabContext;
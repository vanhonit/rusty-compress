import React, { useState, useEffect, useRef } from "react";

const PasswordModal = ({
  isOpen,
  onClose,
  onSubmit,
  error: initialError = null,
  isLoading = false,
  subtitle = null,
  buttonText = null,
}) => {
  const [password, setPassword] = useState("");
  const [error, setError] = useState(initialError);
  const [showPassword, setShowPassword] = useState(false);
  const inputRef = useRef(null);

  // Auto-focus input when modal opens
  useEffect(() => {
    if (isOpen && inputRef.current) {
      inputRef.current.focus();
    }
    // Reset password when modal opens/closes
    if (!isOpen) {
      setPassword("");
      setError(null);
    }
  }, [isOpen]);

  // Update error when initialError changes
  useEffect(() => {
    setError(initialError);
  }, [initialError]);

  const handleSubmit = (e) => {
    e.preventDefault();
    if (password.trim()) {
      onSubmit(password.trim());
    }
  };

  const handleCancel = () => {
    setPassword("");
    setError(null);
    onClose();
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/30 flex items-center justify-center z-[100] p-4">
      <div className="bg-surface rounded-xl shadow-window max-w-md w-full border border-separator overflow-hidden">
        {/* Header */}
        <div className="bg-surface-secondary px-6 py-4 border-b border-separator">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className="w-10 h-10 bg-surface-tertiary rounded-lg flex items-center justify-center">
                <i className="ri-lock-2-line text-xl text-fg"></i>
              </div>
              <div>
                <h2 className="text-[13px] font-semibold text-fg">Archive Password</h2>
                <p className="text-xs text-fg-secondary">{subtitle || "Enter password to continue"}</p>
              </div>
            </div>
            <button
              onClick={handleCancel}
              className="text-fg-secondary hover:text-fg hover:bg-surface-tertiary rounded-lg p-2 transition-colors"
              title="Close"
            >
              <i className="ri-close-line text-xl"></i>
            </button>
          </div>
        </div>

        {/* Content */}
        <form onSubmit={handleSubmit} className="p-6">
          {/* Error Message */}
          {error && (
            <div className="mb-4 p-3 bg-danger/10 border border-danger/40 rounded-xl flex items-start gap-2">
              <div className="w-5 h-5 bg-danger/10 rounded-full flex items-center justify-center flex-shrink-0 mt-0.5">
                <i className="ri-error-warning-line text-sm text-danger"></i>
              </div>
              <div className="flex-1 min-w-0">
                <p className="text-sm text-danger font-medium">Password Error</p>
                <p className="text-xs text-danger mt-0.5">{error}</p>
              </div>
            </div>
          )}

          {/* Password Input */}
          <div className="mb-4">
            <label htmlFor="password" className="block text-sm font-medium text-fg mb-2">
              Password
            </label>
            <div className="relative">
              <input
                ref={inputRef}
                id="password"
                type={showPassword ? "text" : "password"}
                value={password}
                onChange={(e) => {
                  setPassword(e.target.value);
                  setError(null); // Clear error when user starts typing
                }}
                placeholder="Enter archive password"
                disabled={isLoading}
                className="w-full px-4 py-3 pr-12 border border-separator rounded-xl focus:outline-none focus:ring-2 focus:ring-accent/30 focus:border-transparent disabled:bg-surface-secondary disabled:cursor-not-allowed transition-all"
                autoComplete="current-password"
              />
              <button
                type="button"
                onClick={() => setShowPassword(!showPassword)}
                className="absolute right-3 top-1/2 transform -translate-y-1/2 text-fg-tertiary hover:text-fg-secondary transition-colors"
                title={showPassword ? "Hide password" : "Show password"}
              >
                <i className={`text-xl ${showPassword ? "ri-eye-off-line" : "ri-eye-line"}`}></i>
              </button>
            </div>
          </div>

          {/* Info Message */}
          <div className="mb-6 p-3 bg-accent/10 border border-accent/40 rounded-xl flex items-start gap-2">
            <div className="w-5 h-5 bg-accent/10 rounded-full flex items-center justify-center flex-shrink-0 mt-0.5">
              <i className="ri-information-line text-sm text-accent"></i>
            </div>
            <div className="flex-1 min-w-0">
              <p className="text-xs text-accent">
                This archive is password-protected. Please enter the correct password to extract the files.
              </p>
            </div>
          </div>

          {/* Action Buttons */}
          <div className="flex gap-3">
            <button
              type="button"
              onClick={handleCancel}
              disabled={isLoading}
              className="flex-1 px-6 py-3 bg-surface-secondary hover:bg-surface-tertiary text-fg font-semibold rounded-xl transition-colors disabled:bg-surface-secondary disabled:cursor-not-allowed border border-separator"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={!password.trim() || isLoading}
              className="flex-1 px-6 py-3 bg-accent hover:bg-accent-hover text-accent-fg font-semibold rounded-xl transition-all disabled:bg-surface-tertiary disabled:cursor-not-allowed shadow-sm border border-transparent"
            >
              {isLoading ? (
                <span className="flex items-center justify-center gap-2">
                  <i className="ri-refresh-line animate-spin"></i>
                  <span>Verifying...</span>
                </span>
              ) : (
                <span className="flex items-center justify-center gap-2">
                  <i className="ri-unlock-line"></i>
                  <span>{buttonText || "Extract"}</span>
                </span>
              )}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};

export default PasswordModal;
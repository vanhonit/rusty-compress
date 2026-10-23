import Cocoa
import FinderSync

/// Archive suffixes offered in the context menu. Mirrors `detect_archive_type`
/// in src-tauri/src/utils.rs, minus 7z (the app cannot extract it).
/// Keep this list in sync with that function.
private let archiveSuffixes: [String] = [
    ".zip", ".jar", ".apk", ".xapk", ".ipa",
    ".tar", ".tar.gz", ".tgz", ".tar.bz2", ".tbz2", ".gz", ".bz2",
    ".rar",
]

/// URL scheme registered by the main app (src-tauri/Info.plist).
private let appURLScheme = "rustycompress"

/// Unreserved characters only. Everything else is percent-encoded, so `+`, `&`
/// and `=` in file names survive the trip (the Rust side decodes `+` as a space).
private let strictQueryAllowed: CharacterSet = CharacterSet(
    charactersIn: "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~"
)

class FinderSync: FIFinderSync {

    override init() {
        super.init()
        FIFinderSyncController.default().directoryURLs = [URL(fileURLWithPath: "/")]
    }

    // MARK: - Menu

    override func menu(for menuKind: FIMenuKind) -> NSMenu? {
        let menu = NSMenu(title: "")
        guard menuKind == .contextualMenuForItems, allSelectionIsArchive() else {
            return menu
        }

        let here = NSMenuItem(title: "Extract Here with Rusty", action: #selector(extractHere(_:)), keyEquivalent: "")
        here.target = self
        here.image = symbol("tray.and.arrow.down", "Extract here")
        let to = NSMenuItem(title: "Rusty Extract To…", action: #selector(extractTo(_:)), keyEquivalent: "")
        to.target = self
        to.image = symbol("folder.badge.plus", "Extract to folder")

        // Top-level items (no submenu) so they appear directly in Finder's context menu.
        menu.addItem(here)
        menu.addItem(to)
        return menu
    }

    /// SF Symbol drawn in solid white. Finder copies extension menu items into its
    /// own process and drops the template flag, so template images render black;
    /// tint explicitly instead.
    private func symbol(_ name: String, _ description: String) -> NSImage? {
        guard let base = NSImage(systemSymbolName: name, accessibilityDescription: description) else {
            return nil
        }
        let size = base.size
        let tinted = NSImage(size: size)
        tinted.lockFocus()
        base.draw(in: NSRect(origin: .zero, size: size))
        NSColor.white.set()
        NSRect(origin: .zero, size: size).fill(using: .sourceAtop)
        tinted.unlockFocus()
        tinted.isTemplate = false
        tinted.accessibilityDescription = description
        return tinted
    }

    /// True only when at least one item is selected and every selected item is a
    /// regular file whose name ends with a supported archive suffix.
    private func allSelectionIsArchive() -> Bool {
        guard let urls = FIFinderSyncController.default().selectedItemURLs(), !urls.isEmpty else {
            return false
        }
        return urls.allSatisfy { isSupportedArchive($0) }
    }

    private func isSupportedArchive(_ url: URL) -> Bool {
        guard url.isFileURL else { return false }
        // Unknown directory status is treated as "not a file" so folders are never offered.
        let isDirectory = (try? url.resourceValues(forKeys: [.isDirectoryKey]))?.isDirectory ?? true
        if isDirectory { return false }
        let name = url.lastPathComponent.lowercased()
        return archiveSuffixes.contains { name.hasSuffix($0) }
    }

    // MARK: - Actions

    @objc func extractHere(_ sender: AnyObject?) {
        openRequest(mode: "here")
    }

    @objc func extractTo(_ sender: AnyObject?) {
        openRequest(mode: "to")
    }

    /// Hands the selection to the main app as `rustycompress://extract?mode=...&path=...`.
    /// The extension never touches the files itself; it is sandboxed.
    private func openRequest(mode: String) {
        let urls = FIFinderSyncController.default().selectedItemURLs() ?? []
        let paths = urls.filter { $0.isFileURL }.map { $0.path }
        guard !paths.isEmpty else { return }

        var components = URLComponents()
        components.scheme = appURLScheme
        components.host = "extract"
        components.percentEncodedQueryItems =
            [URLQueryItem(name: "mode", value: encode(mode))]
            + paths.map { URLQueryItem(name: "path", value: encode($0)) }

        guard let url = components.url else { return }
        NSWorkspace.shared.open(url)
    }

    private func encode(_ value: String) -> String {
        value.addingPercentEncoding(withAllowedCharacters: strictQueryAllowed) ?? value
    }
}

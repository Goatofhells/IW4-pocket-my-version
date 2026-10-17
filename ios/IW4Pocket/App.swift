import SwiftUI
import UniformTypeIdentifiers
import UIKit

struct ServiceReply: Decodable { let ok: Bool; let message: String }
func reply(_ pointer: UnsafeMutablePointer<CChar>?) throws -> ServiceReply {
    guard let pointer = pointer else { throw CocoaError(.coderInvalidValue) }
    defer { iw4_string_free(pointer) }
    return try JSONDecoder().decode(ServiceReply.self, from: Data(String(cString: pointer).utf8))
}
@MainActor final class Installation: ObservableObject {
    @Published var status = "No verification performed"
    @Published var busy = false
    var root: URL { FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0].appendingPathComponent("Installation", isDirectory: true) }
    var hasFiles: Bool { FileManager.default.fileExists(atPath: root.path) }
    func run(_ operation: @escaping () throws -> String) {
        guard !busy else { return }; busy = true
        Task {
            do { status = try await Task.detached(priority: .userInitiated) { try operation() }.value }
            catch { status = error.localizedDescription }
            busy = false
        }
    }
    func verify() {
        let path = root.path
        run {
            guard let url = Bundle.main.url(forResource: "manifests-v1", withExtension: "json") else { throw CocoaError(.fileNoSuchFile) }
            let catalog = try String(contentsOf: url, encoding: .utf8)
            return try path.withCString { p in try catalog.withCString { c in try reply(iw4_verify(p, c)).message } }
        }
    }
    func importFiles(_ urls: [URL]) {
        let destination = root
        run {
            let fm = FileManager.default
            try fm.createDirectory(at: destination.deletingLastPathComponent(), withIntermediateDirectories: true)
            let staging = destination.deletingLastPathComponent().appendingPathComponent(UUID().uuidString)
            defer { try? fm.removeItem(at: staging) }
            guard !urls.isEmpty else { return "No files selected" }
            if urls.count == 1, urls[0].pathExtension.lowercased() == "zip" {
                let url = urls[0]; let access = url.startAccessingSecurityScopedResource()
                defer { if access { url.stopAccessingSecurityScopedResource() } }
                let result = try url.path.withCString { a in try staging.path.withCString { d in try reply(iw4_extract(a,d)) } }
                guard result.ok else { return result.message }
            } else {
                try fm.createDirectory(at: staging, withIntermediateDirectories: false)
                var total: Int64 = 0; var names = Set<String>()
                guard urls.count <= 20_000 else { return "Too many files" }
                for url in urls {
                    let access = url.startAccessingSecurityScopedResource()
                    defer { if access { url.stopAccessingSecurityScopedResource() } }
                    let v = try url.resourceValues(forKeys: [.isRegularFileKey,.isSymbolicLinkKey,.fileSizeKey])
                    guard v.isRegularFile == true, v.isSymbolicLink != true, names.insert(url.lastPathComponent.lowercased()).inserted else { return "Select unique regular files, or use a ZIP to preserve directories" }
                    total += Int64(v.fileSize ?? 0)
                    guard total <= 16 * 1024 * 1024 * 1024 else { return "Import exceeds 16 GiB limit" }
                    try fm.copyItem(at: url, to: staging.appendingPathComponent(url.lastPathComponent))
                }
            }
            if fm.fileExists(atPath: destination.path) {
                _ = try fm.replaceItemAt(destination, withItemAt: staging)
            } else { try fm.moveItem(at: staging, to: destination) }
            return "Files imported into sandbox. Verification required; runtime unavailable."
        }
    }
}
struct DocumentPicker: UIViewControllerRepresentable {
    let completion: ([URL]) -> Void
    func makeCoordinator() -> Coordinator { Coordinator(completion) }
    func makeUIViewController(context: Context) -> UIDocumentPickerViewController {
        let picker = UIDocumentPickerViewController(forOpeningContentTypes: [.zip, .data], asCopy: false)
        picker.allowsMultipleSelection = true; picker.delegate = context.coordinator; return picker
    }
    func updateUIViewController(_ uiViewController: UIDocumentPickerViewController, context: Context) {}
    class Coordinator: NSObject, UIDocumentPickerDelegate {
        let completion: ([URL]) -> Void
        init(_ completion: @escaping ([URL]) -> Void) { self.completion = completion }
        func documentPicker(_ controller: UIDocumentPickerViewController, didPickDocumentsAt urls: [URL]) { completion(urls) }
    }
}
@main struct IW4PocketApp: App {
    var body: some Scene { WindowGroup { LauncherView().preferredColorScheme(.dark) } }
}
struct LauncherView: View {
    @StateObject private var installation = Installation()
    @State private var picker = false
    @AppStorage("renderScale") private var renderScale = 0.75
    @AppStorage("frameRate") private var frameRate = 30
    @AppStorage("quality") private var quality = "Low"
    @AppStorage("cacheMB") private var cacheMB = 128
    var body: some View {
        NavigationView {
            List {
                Section {
                    VStack(alignment: .leading, spacing: 12) {
                        Label("IW4 POCKET", systemImage: "scope").font(.largeTitle.bold()).foregroundColor(.mint)
                        Text("Launcher foundation • 0.1.0").foregroundColor(.secondary)
                        Text(installation.status).font(.callout)
                        Label(installation.hasFiles ? "Imported files present" : "No imported installation", systemImage: "folder")
                        Label("IW4 runtime unavailable", systemImage: "lock.fill").foregroundColor(.orange)
                    }.padding(.vertical)
                }
                Section("Installation") {
                    Button { picker = true } label: { Label("Import Game Files", systemImage: "square.and.arrow.down") }
                    Button { installation.verify() } label: { Label("Verify Installation", systemImage: "checkmark.shield") }
                    if installation.busy { ProgressView("Working…") }
                    Text("ZIP preserves installation paths. Individual files import at the root. No verified MW2 catalog is bundled. Hash matches never prove Steam ownership.").font(.caption).foregroundColor(.secondary)
                }.disabled(installation.busy)
                Section("Play") {
                    NavigationLink("Offline Practice") { unavailable("Offline Practice", "The independent arena, rendering, touch controls and bots have not been implemented.") }
                    NavigationLink("Multiplayer") { unavailable("Multiplayer", "No network client or server protocol is implemented. MW2 server compatibility is unverified.") }
                }
                Section {
                    NavigationLink("Settings") {
                        Form {
                            Section("Future runtime preferences — no gameplay effect yet") {
                                Picker("Graphics", selection: $quality) { ForEach(["Low","Medium"], id: \.self) { Text($0) } }
                                Picker("Frame-rate target", selection: $frameRate) { Text("30").tag(30); Text("60").tag(60) }
                                Slider(value: $renderScale, in: 0.5...1, step: 0.05) { Text("Resolution scale") }
                                Text("Resolution scale: \(Int(renderScale * 100))%")
                                Stepper("Cache request: \(cacheMB) MiB", value: $cacheMB, in: 16...512, step: 16)
                                Text("No device frame-rate benchmarks available.")
                            }
                        }.navigationTitle("Settings")
                    }
                    NavigationLink("Diagnostics") {
                        List {
                            Text("Physical memory: \(ProcessInfo.processInfo.physicalMemory / 1_048_576) MiB")
                            Text("Recommended cache cap: \(iw4_cache_limit(UInt32(cacheMB), ProcessInfo.processInfo.physicalMemory / 1_048_576)) MiB")
                            Text("OS: \(ProcessInfo.processInfo.operatingSystemVersionString)")
                            Text("Rust FFI: linked • manifest schema: 1")
                            Text("Renderer: not implemented • network: disconnected")
                            Text("Cache cap is advisory until a resource cache exists.")
                        }.navigationTitle("Diagnostics")
                    }
                }
            }.listStyle(.insetGrouped).navigationTitle("Home")
        }.navigationViewStyle(.stack).accentColor(.mint)
        .sheet(isPresented: $picker) { DocumentPicker { urls in picker = false; installation.importFiles(urls) } }
    }
    func unavailable(_ title: String, _ message: String) -> some View {
        VStack(spacing: 20) { Image(systemName: "hammer").font(.largeTitle).foregroundColor(.mint); Text("Not playable yet").font(.title2.bold()); Text(message).multilineTextAlignment(.center) }.padding(32).navigationTitle(title)
    }
}

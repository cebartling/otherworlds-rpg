import Foundation

/// View model for the character list screen.
@Observable
@MainActor
final class CharacterListViewModel {
    private(set) var characters: [CharacterSummary] = []
    private(set) var isLoading = false
    private(set) var error: APIError?

    private let endpoint: CharacterEndpoint

    init(endpoint: CharacterEndpoint) {
        self.endpoint = endpoint
    }

    func loadCharacters() async {
        AppLogger.ui.info("Loading characters...")
        isLoading = true
        error = nil
        do {
            characters = try await endpoint.listCharacters()
            AppLogger.ui.info("Loaded \(self.characters.count) characters")
        } catch let apiError as APIError {
            AppLogger.ui.error("Failed to load characters: \(apiError.localizedDescription)")
            error = apiError
        } catch {
            AppLogger.ui.error("Failed to load characters: \(error.localizedDescription)")
            self.error = .network(error.localizedDescription)
        }
        isLoading = false
    }

    func archiveCharacter(id: UUID) async {
        error = nil
        do {
            _ = try await endpoint.archiveCharacter(id: id)
            await loadCharacters()
        } catch let apiError as APIError {
            AppLogger.ui.error("Failed to archive character \(id): \(apiError.localizedDescription)")
            error = apiError
        } catch {
            AppLogger.ui.error("Failed to archive character \(id): \(error.localizedDescription)")
            self.error = .network(error.localizedDescription)
        }
    }

    func dismissError() {
        error = nil
    }
}

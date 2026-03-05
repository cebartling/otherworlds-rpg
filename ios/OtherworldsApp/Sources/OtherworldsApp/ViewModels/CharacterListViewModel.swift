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
        isLoading = true
        error = nil
        do {
            characters = try await endpoint.listCharacters()
        } catch let apiError as APIError {
            error = apiError
        } catch {
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
            error = apiError
        } catch {
            self.error = .network(error.localizedDescription)
        }
    }

    func dismissError() {
        error = nil
    }
}

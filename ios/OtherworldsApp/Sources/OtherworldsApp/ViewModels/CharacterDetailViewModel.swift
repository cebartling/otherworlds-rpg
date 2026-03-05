import Foundation

/// View model for the character detail screen.
@Observable
@MainActor
final class CharacterDetailViewModel {
    private(set) var character: CharacterDetail?
    private(set) var isLoading = false
    private(set) var error: APIError?

    let characterId: UUID
    private let endpoint: CharacterEndpoint

    init(characterId: UUID, endpoint: CharacterEndpoint) {
        self.characterId = characterId
        self.endpoint = endpoint
    }

    func loadCharacter() async {
        AppLogger.ui.info("Loading character \(self.characterId)...")
        isLoading = true
        error = nil
        do {
            character = try await endpoint.getCharacter(id: characterId)
        } catch let apiError as APIError {
            AppLogger.ui.error("Failed to load character \(self.characterId): \(apiError.localizedDescription)")
            error = apiError
        } catch {
            AppLogger.ui.error("Failed to load character \(self.characterId): \(error.localizedDescription)")
            self.error = .network(error.localizedDescription)
        }
        isLoading = false
    }

    func modifyAttribute(attribute: String, newValue: Int32) async {
        error = nil
        do {
            let request = ModifyAttributeRequest(
                characterId: characterId,
                attribute: attribute,
                newValue: newValue
            )
            _ = try await endpoint.modifyAttribute(request: request)
            await loadCharacter()
        } catch let apiError as APIError {
            error = apiError
        } catch {
            self.error = .network(error.localizedDescription)
        }
    }

    func awardExperience(amount: UInt32) async {
        error = nil
        do {
            let request = AwardExperienceRequest(characterId: characterId, amount: amount)
            _ = try await endpoint.awardExperience(request: request)
            await loadCharacter()
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

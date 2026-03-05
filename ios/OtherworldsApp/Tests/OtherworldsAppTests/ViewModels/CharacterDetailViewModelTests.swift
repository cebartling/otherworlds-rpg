import Foundation
import Testing

@testable import OtherworldsApp

@Suite("CharacterDetailViewModel — loading, success, error states")
@MainActor
struct CharacterDetailViewModelTests {

    private func makeViewModel(characterId: UUID = UUID()) -> (CharacterDetailViewModel, MockHTTPClient) {
        let mock = MockHTTPClient()
        let endpoint = CharacterEndpoint(client: mock)
        let vm = CharacterDetailViewModel(characterId: characterId, endpoint: endpoint)
        return (vm, mock)
    }

    private func characterJSON(id: UUID) -> Data {
        """
        {
            "character_id":"\(id)",
            "name":"Alaric",
            "attributes":{"strength":18,"dexterity":14},
            "experience":250,
            "version":3
        }
        """.data(using: .utf8)!
    }

    @Test func loadCharacter_success_populatesCharacter() async {
        let characterId = UUID()
        let (vm, mock) = makeViewModel(characterId: characterId)
        mock.nextResult = .success(characterJSON(id: characterId))

        await vm.loadCharacter()

        #expect(vm.character != nil)
        #expect(vm.character?.characterId == characterId)
        #expect(vm.character?.name == "Alaric")
        #expect(vm.character?.attributes["strength"] == 18)
        #expect(vm.isLoading == false)
        #expect(vm.error == nil)
    }

    @Test func loadCharacter_error_setsError() async {
        let (vm, mock) = makeViewModel()
        mock.nextResult = .failure(.server(statusCode: 404, error: "not_found", message: "Not found"))

        await vm.loadCharacter()

        #expect(vm.character == nil)
        #expect(vm.error == .server(statusCode: 404, error: "not_found", message: "Not found"))
    }

    @Test func modifyAttribute_callsEndpointAndReloads() async {
        let characterId = UUID()
        let (vm, mock) = makeViewModel(characterId: characterId)
        mock.nextResult = .success("""
            {"event_ids":[]}
            """.data(using: .utf8)!)

        await vm.modifyAttribute(attribute: "strength", newValue: 20)

        #expect(mock.calls.count >= 1)
        #expect(mock.calls[0].method == "POST")
        #expect(mock.calls[0].path.contains("modify-attribute"))
    }

    @Test func awardExperience_callsEndpointAndReloads() async {
        let characterId = UUID()
        let (vm, mock) = makeViewModel(characterId: characterId)
        mock.nextResult = .success("""
            {"event_ids":[]}
            """.data(using: .utf8)!)

        await vm.awardExperience(amount: 100)

        #expect(mock.calls.count >= 1)
        #expect(mock.calls[0].method == "POST")
        #expect(mock.calls[0].path.contains("award-experience"))
    }

    @Test func dismissError_clearsError() async {
        let (vm, mock) = makeViewModel()
        mock.nextResult = .failure(.httpError(statusCode: 500))

        await vm.loadCharacter()
        #expect(vm.error != nil)

        vm.dismissError()
        #expect(vm.error == nil)
    }
}

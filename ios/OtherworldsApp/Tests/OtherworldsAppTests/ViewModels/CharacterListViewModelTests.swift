import Foundation
import Testing

@testable import OtherworldsApp

@Suite("CharacterListViewModel — loading, success, error states")
@MainActor
struct CharacterListViewModelTests {

    private func makeViewModel() -> (CharacterListViewModel, MockHTTPClient) {
        let mock = MockHTTPClient()
        let endpoint = CharacterEndpoint(client: mock)
        let vm = CharacterListViewModel(endpoint: endpoint)
        return (vm, mock)
    }

    @Test func loadCharacters_success_populatesCharacters() async {
        let (vm, mock) = makeViewModel()
        let characterId = UUID()
        let json = """
            [{"character_id":"\(characterId)","name":"Alaric","experience":250,"version":3}]
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        await vm.loadCharacters()

        #expect(vm.characters.count == 1)
        #expect(vm.characters[0].characterId == characterId)
        #expect(vm.characters[0].name == "Alaric")
        #expect(vm.isLoading == false)
        #expect(vm.error == nil)
    }

    @Test func loadCharacters_error_setsError() async {
        let (vm, mock) = makeViewModel()
        mock.nextResult = .failure(.httpError(statusCode: 500))

        await vm.loadCharacters()

        #expect(vm.characters.isEmpty)
        #expect(vm.isLoading == false)
        #expect(vm.error == .httpError(statusCode: 500))
    }

    @Test func archiveCharacter_success_reloads() async {
        let (vm, mock) = makeViewModel()
        let eventId = UUID()
        mock.nextResult = .success("""
            {"event_ids":["\(eventId)"]}
            """.data(using: .utf8)!)

        await vm.archiveCharacter(id: UUID())

        #expect(mock.calls.count >= 1)
        #expect(mock.calls[0].method == "DELETE")
    }

    @Test func dismissError_clearsError() async {
        let (vm, mock) = makeViewModel()
        mock.nextResult = .failure(.httpError(statusCode: 404))

        await vm.loadCharacters()
        #expect(vm.error != nil)

        vm.dismissError()
        #expect(vm.error == nil)
    }
}

Feature: Sync file compatibility with the Go app
  Fixtures: tests/fixtures/nunc-sync.json, tests/fixtures/webdav/nunc-sync.json

  @integration @REQ-compat @wip
  Scenario: A sync file written by the Go app is read

  @integration @REQ-compat @wip
  Scenario: A sync file written by nunc is read by the Go format definition

  @integration @REQ-compat @wip
  Scenario: The roundtrip Go to nunc to Go keeps all todos and sessions

  @integration @REQ-compat @wip
  Scenario: The WebDAV file of another device is merged with the local database

  @e2e @US-Sync-02-WebDAV @desktop @wip
  Scenario: Go app and nunc sync alternately against a WebDAV server

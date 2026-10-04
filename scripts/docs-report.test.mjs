import { describe, expect, it } from "vitest";
import {
  layerOf,
  parseFeature,
  problems,
  statusOf,
  storiesOf,
  traceability,
} from "./docs-report.mjs";

const text = `@REQ-1
Feature: Demo

  @domain @US-A @wip
  Scenario: Open
    Given x

  # Kommentar
  @integration @manual
  Scenario Outline: Manual
    Given y
`;

describe("docs-report", () => {
  const feature = parseFeature(text, "features/demo.feature");

  it("liest Titel, Tags und Szenarien (Feature-Tags erben)", () => {
    expect(feature.title).toBe("Demo");
    expect(feature.scenarios.map((s) => s.title)).toEqual(["Open", "Manual"]);
    expect(storiesOf(feature.scenarios[0])).toEqual(["REQ-1", "US-A"]);
    expect(layerOf(feature.scenarios[1])).toBe("integration");
    expect(statusOf(feature.scenarios[0])).toBe("wip");
    expect(statusOf(feature.scenarios[1])).toBe("manuell");
  });

  it("meldet @wip und fehlende Ebene", () => {
    expect(problems([feature])).toEqual(["features/demo.feature: Open – @wip"]);
  });

  it("gruppiert die Rückverfolgbarkeit nach Story", () => {
    const report = traceability([feature]);
    expect(report).toContain("## US-A");
    expect(report).toContain(
      "| Open | `features/demo.feature` | domain | wip |",
    );
  });
});

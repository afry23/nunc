import { addDays, toIsoDate } from "../lib/format";

/**
 * Vorschau im normalen Browser (`npm run dev`, ohne Tauri): ein Fake-Backend mit Beispieldaten ersetzt die
 * IPC-Schicht, damit sich die Oberfläche ohne App ansehen lässt. Wird nur im Dev-Build geladen.
 */
export async function installPreviewBackend(): Promise<void> {
  const { createFakeBackend } = await import("../../tests/helpers/fakeBackend");
  const fake = createFakeBackend();

  fake.addTodo("(A) Systemarchitektur finalisieren +projekt_alpha @buero", {
    pomodoroCount: 4,
  });
  fake.addTodo("(A) Folien für das Review vorbereiten +stakeholder", {
    pomodoroCount: 2,
  });
  fake.addTodo("(B) Pull Requests prüfen +redesign @coding", {
    pomodoroCount: 1,
  });
  fake.addTodo("(B) Abstimmung mit dem Team @call");
  fake.addTodo("Abhängigkeiten aktualisieren +projekt_alpha");
  fake.addTodo("Neues Equipment bestellen @zuhause");
  fake.addTodo("x Steuererklärung abgeben +privat", { completed: true });

  const today = new Date();
  const counts = [
    3, 5, 0, 4, 6, 2, 0, 4, 5, 3, 0, 1, 4, 6, 5, 2, 0, 3, 4, 4, 5, 0, 2, 3, 6,
    4, 5, 3, 4, 2,
  ];
  const daily = counts.map((count, index) => ({
    date: toIsoDate(addDays(today, index - counts.length + 1)),
    pomodoroCount: count,
    workMinutes: count * 25,
    breakMinutes: count * 5,
  }));
  fake.stats = {
    ...fake.stats,
    daily,
    projects: [
      { project: "+projekt_alpha", workMinutes: 1250 },
      { project: "+redesign", workMinutes: 640 },
      { project: "+stakeholder", workMinutes: 300 },
    ],
    contexts: [
      { context: "@buero", workMinutes: 900 },
      { context: "@coding", workMinutes: 700 },
      { context: "@zuhause", workMinutes: 210 },
    ],
    topTodos: [
      {
        todoId: 1,
        description: "Systemarchitektur finalisieren",
        pomodoroCount: 4,
        workMinutes: 100,
      },
      {
        todoId: 2,
        description: "Folien für das Review vorbereiten",
        pomodoroCount: 2,
        workMinutes: 50,
      },
    ],
    totalPomodoros: counts.reduce((a, b) => a + b, 0),
    totalWorkMin: counts.reduce((a, b) => a + b, 0) * 25,
    averagePomodoros7: 3.6,
    averagePomodoros30: 3.1,
    currentStreak: 4,
    longestStreak: 9,
  };
}

/** Mobile Plattform (Touch-Geräte); Desktop-Funktionen wie Tastenkürzel sind dort aus. */
export function isMobile(userAgent: string = navigator.userAgent): boolean {
  return /Android|iPhone|iPad|iPod/i.test(userAgent);
}

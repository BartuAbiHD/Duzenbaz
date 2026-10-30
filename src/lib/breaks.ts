export type NightSettings = { enabled: boolean; start: string; end: string; warmth: number };

export type BreakSettings = {
  enabled: boolean;
  intervalMin: number;
  breakSecs: number;
  night: NightSettings;
};

export type BreakState = {
  phase: "working" | "break";
  workSecs: number;
  intervalSecs: number;
  breakRemaining: number;
  breakSecs: number;
  enabled: boolean;
};

export type Report = {
  hours: number[];
  days: { day: string; seconds: number }[];
  todayIndex: number;
  breaksTaken: number;
  breaksSkipped: number;
};

export function formatDuration(secs: number): string {
  const m = Math.round(secs / 60);
  if (m < 60) return `${m} dk`;
  return `${Math.floor(m / 60)} sa ${m % 60} dk`;
}

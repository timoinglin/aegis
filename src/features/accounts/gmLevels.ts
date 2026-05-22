// GM levels for this repack (auth.account_access.gmlevel, 0–9). Labels follow the
// verified command-security tiers; the worldserver applies the change after a relog.
export const GM_LEVELS: { value: number; label: string }[] = [
  { value: 0, label: "0 — Player (no GM access)" },
  { value: 1, label: "1 — Moderator (low)" },
  { value: 2, label: "2 — Moderator" },
  { value: 3, label: "3 — Chat moderation" },
  { value: 4, label: "4 — Basic GM (appear, go, kick, ban)" },
  { value: 5, label: "5 — Content GM (items, mail, modify)" },
  { value: 6, label: "6 — Senior GM (achievements, cheats)" },
  { value: 7, label: "7 — Admin" },
  { value: 8, label: "8 — Admin" },
  { value: 9, label: "9 — Full admin (everything)" },
];

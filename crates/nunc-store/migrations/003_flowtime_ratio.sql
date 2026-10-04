-- Add ratio-based break calculation settings.
-- flowtime_ratio:            work-to-rest ratio (e.g. 0.2 = 1:0.2); range 0.1–0.5.
-- flowtime_min_flow_state_min: minimum worked minutes before a break is suggested.
INSERT OR IGNORE INTO settings (key, value) VALUES
  ('flowtime_ratio', '0.2'),
  ('flowtime_min_flow_state_min', '15');

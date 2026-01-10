// SF Symbols-style SVG icon paths
// Designed for 24x24 viewBox with 1.5px stroke weight

export interface IconPath {
  outline: string;
  filled: string;
}

export const iconPaths: Record<string, IconPath> = {
  // Dashboard - square.grid.2x2
  dashboard: {
    outline: 'M3 3h8v8H3V3zm10 0h8v8h-8V3zM3 13h8v8H3v-8zm10 0h8v8h-8v-8z',
    filled: 'M3 3h8v8H3V3zm10 0h8v8h-8V3zM3 13h8v8H3v-8zm10 0h8v8h-8v-8z',
  },

  // Diagnose - stethoscope / heart.text.square
  diagnose: {
    outline: 'M9 2v3M15 2v3M9 5a3 3 0 0 0-3 3v3a6 6 0 0 0 12 0V8a3 3 0 0 0-3-3M12 17v4M10 21h4',
    filled: 'M9 2v3M15 2v3M9 5a3 3 0 0 0-3 3v3a6 6 0 0 0 12 0V8a3 3 0 0 0-3-3M12 17v4M10 21h4',
  },

  // WiFi - wifi
  wifi: {
    outline: 'M12 19.5a1.5 1.5 0 1 0 0-3 1.5 1.5 0 0 0 0 3zM8.5 15.5a5 5 0 0 1 7 0M5 12a9 9 0 0 1 14 0M1.5 8.5a13 13 0 0 1 21 0',
    filled: 'M12 19.5a1.5 1.5 0 1 0 0-3 1.5 1.5 0 0 0 0 3zM8.5 15.5a5 5 0 0 1 7 0M5 12a9 9 0 0 1 14 0M1.5 8.5a13 13 0 0 1 21 0',
  },

  // Speed - speedometer / gauge
  speed: {
    outline: 'M12 22c5.523 0 10-4.477 10-10S17.523 2 12 2 2 6.477 2 12s4.477 10 10 10zM12 6v4M12 12l4 4',
    filled: 'M12 22c5.523 0 10-4.477 10-10S17.523 2 12 2 2 6.477 2 12s4.477 10 10 10zM12 6v4M12 12l4 4',
  },

  // More - ellipsis.circle
  more: {
    outline: 'M12 22c5.523 0 10-4.477 10-10S17.523 2 12 2 2 6.477 2 12s4.477 10 10 10zM8 12h.01M12 12h.01M16 12h.01',
    filled: 'M12 22c5.523 0 10-4.477 10-10S17.523 2 12 2 2 6.477 2 12s4.477 10 10 10zM8 12h.01M12 12h.01M16 12h.01',
  },

  // Interfaces - network / server
  interfaces: {
    outline: 'M4 6h16M4 6v12a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V6M4 6l2-4h12l2 4M12 10v4M8 10v4M16 10v4',
    filled: 'M4 6h16M4 6v12a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V6M4 6l2-4h12l2 4M12 10v4M8 10v4M16 10v4',
  },

  // Ping - antenna.radiowaves.left.and.right
  ping: {
    outline: 'M12 12v8M8.5 6.5a5 5 0 0 1 7 0M5.5 3.5a9 9 0 0 1 13 0M6 16c-1.5 1-3 2.5-3 4M18 16c1.5 1 3 2.5 3 4',
    filled: 'M12 12v8M8.5 6.5a5 5 0 0 1 7 0M5.5 3.5a9 9 0 0 1 13 0M6 16c-1.5 1-3 2.5-3 4M18 16c1.5 1 3 2.5 3 4',
  },

  // Traceroute - point.topleft.down.curvedto.point.bottomright.up / map-pin with path
  traceroute: {
    outline: 'M3 6a3 3 0 1 0 0-6 3 3 0 0 0 0 6zM21 24a3 3 0 1 0 0-6 3 3 0 0 0 0 6zM6 3c4 0 6 4 9 9s5 9 9 9',
    filled: 'M3 6a3 3 0 1 0 0-6 3 3 0 0 0 0 6zM21 24a3 3 0 1 0 0-6 3 3 0 0 0 0 6zM6 3c4 0 6 4 9 9s5 9 9 9',
  },

  // DNS - magnifyingglass / globe with magnifier
  dns: {
    outline: 'M11 19a8 8 0 1 0 0-16 8 8 0 0 0 0 16zM21 21l-4.35-4.35',
    filled: 'M11 19a8 8 0 1 0 0-16 8 8 0 0 0 0 16zM21 21l-4.35-4.35',
  },

  // Capture - record.circle / eye
  capture: {
    outline: 'M12 22c5.523 0 10-4.477 10-10S17.523 2 12 2 2 6.477 2 12s4.477 10 10 10zM12 16a4 4 0 1 0 0-8 4 4 0 0 0 0 8z',
    filled: 'M12 22c5.523 0 10-4.477 10-10S17.523 2 12 2 2 6.477 2 12s4.477 10 10 10zM12 16a4 4 0 1 0 0-8 4 4 0 0 0 0 8z',
  },

  // Report - doc.text
  report: {
    outline: 'M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8l-6-6zM14 2v6h6M8 13h8M8 17h8M8 9h2',
    filled: 'M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8l-6-6zM14 2v6h6M8 13h8M8 17h8M8 9h2',
  },

  // Fix - wrench.and.screwdriver
  fix: {
    outline: 'M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z',
    filled: 'M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z',
  },

  // Chevron right - chevron.right
  chevron: {
    outline: 'M9 18l6-6-6-6',
    filled: 'M9 18l6-6-6-6',
  },

  // Check - checkmark.circle
  check: {
    outline: 'M12 22c5.523 0 10-4.477 10-10S17.523 2 12 2 2 6.477 2 12s4.477 10 10 10zM9 12l2 2 4-4',
    filled: 'M12 22c5.523 0 10-4.477 10-10S17.523 2 12 2 2 6.477 2 12s4.477 10 10 10zM9 12l2 2 4-4',
  },

  // X mark - xmark.circle
  xmark: {
    outline: 'M12 22c5.523 0 10-4.477 10-10S17.523 2 12 2 2 6.477 2 12s4.477 10 10 10zM15 9l-6 6M9 9l6 6',
    filled: 'M12 22c5.523 0 10-4.477 10-10S17.523 2 12 2 2 6.477 2 12s4.477 10 10 10zM15 9l-6 6M9 9l6 6',
  },

  // Close / X - xmark
  close: {
    outline: 'M18 6L6 18M6 6l12 12',
    filled: 'M18 6L6 18M6 6l12 12',
  },

  // Refresh - arrow.clockwise
  refresh: {
    outline: 'M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74L21 8V3M21 3v5h-5',
    filled: 'M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74L21 8V3M21 3v5h-5',
  },

  // Warning - exclamationmark.triangle
  warning: {
    outline: 'M12 9v4M12 17h.01M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z',
    filled: 'M12 9v4M12 17h.01M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z',
  },

  // Info - info.circle
  info: {
    outline: 'M12 22c5.523 0 10-4.477 10-10S17.523 2 12 2 2 6.477 2 12s4.477 10 10 10zM12 16v-4M12 8h.01',
    filled: 'M12 22c5.523 0 10-4.477 10-10S17.523 2 12 2 2 6.477 2 12s4.477 10 10 10zM12 16v-4M12 8h.01',
  },

  // Play - play.fill
  play: {
    outline: 'M5 3l14 9-14 9V3z',
    filled: 'M5 3l14 9-14 9V3z',
  },

  // Stop - stop.fill
  stop: {
    outline: 'M6 4h12v16H6z',
    filled: 'M6 4h12v16H6z',
  },

  // Settings - gearshape
  settings: {
    outline: 'M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6zM19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z',
    filled: 'M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6zM19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z',
  },
};

export type IconName = keyof typeof iconPaths;

import { detectPlatform, type PlatformInfo } from './detect';

let platformInfo = $state<PlatformInfo | null>(null);
let isInitialized = $state(false);

export async function initializePlatform(): Promise<PlatformInfo> {
  if (platformInfo) {
    return platformInfo;
  }

  platformInfo = await detectPlatform();
  applyPlatformClasses(platformInfo);
  isInitialized = true;

  // Also set up dark mode class listener
  setupDarkModeListener();

  return platformInfo;
}

function applyPlatformClasses(info: PlatformInfo): void {
  const body = document.body;

  // Remove existing platform classes
  body.classList.remove(
    'platform-macos', 'platform-windows', 'platform-linux',
    'platform-ios', 'platform-android', 'platform-unknown',
    'device-desktop', 'device-mobile'
  );

  // Apply new classes
  body.classList.add(`platform-${info.platform}`);
  body.classList.add(info.isDesktop ? 'device-desktop' : 'device-mobile');

  // Apply dark mode class based on system preference
  if (window.matchMedia('(prefers-color-scheme: dark)').matches) {
    body.classList.add('dark');
  }
}

function setupDarkModeListener(): void {
  const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
  mediaQuery.addEventListener('change', (e) => {
    if (e.matches) {
      document.body.classList.add('dark');
    } else {
      document.body.classList.remove('dark');
    }
  });
}

export function getPlatform(): PlatformInfo | null {
  return platformInfo;
}

export function isPlatformReady(): boolean {
  return isInitialized;
}

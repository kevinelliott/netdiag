import { platform } from '@tauri-apps/plugin-os';

export type Platform = 'macos' | 'windows' | 'linux' | 'ios' | 'android' | 'unknown';

export interface PlatformInfo {
  platform: Platform;
  isDesktop: boolean;
  isMobile: boolean;
  supportsVibrancy: boolean;
  supportsAcrylic: boolean;
}

export async function detectPlatform(): Promise<PlatformInfo> {
  try {
    const os = await platform();
    const platformMap: Record<string, Platform> = {
      'macos': 'macos',
      'darwin': 'macos',
      'windows': 'windows',
      'linux': 'linux',
      'ios': 'ios',
      'android': 'android',
    };

    const detectedPlatform = platformMap[os] || 'unknown';
    const isMobile = ['ios', 'android'].includes(detectedPlatform);

    return {
      platform: detectedPlatform,
      isDesktop: !isMobile,
      isMobile,
      supportsVibrancy: detectedPlatform === 'macos',
      supportsAcrylic: detectedPlatform === 'windows',
    };
  } catch {
    // Fallback for web/dev mode - detect from user agent
    return detectFromUserAgent();
  }
}

function detectFromUserAgent(): PlatformInfo {
  const ua = navigator.userAgent.toLowerCase();

  let detectedPlatform: Platform = 'unknown';
  if (ua.includes('iphone') || ua.includes('ipad')) {
    detectedPlatform = 'ios';
  } else if (ua.includes('android')) {
    detectedPlatform = 'android';
  } else if (ua.includes('mac')) {
    detectedPlatform = 'macos';
  } else if (ua.includes('win')) {
    detectedPlatform = 'windows';
  } else if (ua.includes('linux')) {
    detectedPlatform = 'linux';
  }

  const isMobile = ['ios', 'android'].includes(detectedPlatform);

  return {
    platform: detectedPlatform,
    isDesktop: !isMobile,
    isMobile,
    supportsVibrancy: detectedPlatform === 'macos',
    supportsAcrylic: detectedPlatform === 'windows',
  };
}

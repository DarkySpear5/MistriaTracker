import { describe, expect, it } from 'vitest';
import viteConfig from '../../vite.config';
import tauriConfig from '../../src-tauri/tauri.conf.json';
import packageConfig from '../../package.json';

describe('workspace runtime configuration', () => {
  it('keeps the Tauri dev URL and Vite server on port 1420', () => {
    expect(tauriConfig.build?.devUrl).toBe('http://localhost:1420');
    expect(viteConfig.server?.port).toBe(1420);
    expect(viteConfig.server?.strictPort).toBe(true);
  });

  it('enables production bundling for the native desktop app', () => {
    expect(tauriConfig.bundle?.active).toBe(true);
  });

  it('declares the verified Node runtime floor', () => {
    expect(packageConfig.engines?.node).toBe('>=24.19.0');
  });
});

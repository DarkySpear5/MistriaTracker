import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

const repositoryRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const companion = resolve(
  repositoryRoot,
  'companion/mistria_tracker_companion/gml/MistriaTrackerCompanion.gml',
);

describe('1.0.5 companion compatibility', () => {
  it('emits the installed game version in each live event', () => {
    const source = readFileSync(companion, 'utf8');

    expect(source).toContain('game_version: "1.0.5"');
  });
});

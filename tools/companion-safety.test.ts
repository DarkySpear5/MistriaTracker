import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

const REPOSITORY_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const COMPANION_GML = resolve(REPOSITORY_ROOT, 'companion/mistria_tracker_companion/gml/MistriaTrackerCompanion.gml');
const MANIFEST = resolve(REPOSITORY_ROOT, 'companion/mistria_tracker_companion/manifest.json');
const INSTALLER_SCRIPT = resolve(REPOSITORY_ROOT, 'tools/installer.nsi');

const FORBIDDEN_API_PATTERNS = [
  /\b(?:save|load)\s*(?:_|\.)\s*[A-Za-z0-9_]+\s*\(/i,
  /\b(?:file|directory|buffer)\s*_\s*[A-Za-z0-9_]+\s*\(/i,
  /\b(?:item|object)\s*_\s*(?:register|create|add)\s*\(/i,
  /\bmmapi\s*_\s*register(?:\s*_\s*[A-Za-z0-9_]+)+\s*\(/i,
  /\bmmapi\s*_\s*(?:config|modsave|guard|override)(?:\s*_\s*[A-Za-z0-9_]+)*\s*\(/i,
  /\b(?:give|add|remove|modify|set)\s*_\s*(?:item|gold|currency|heart|relationship|collection|museum|time|room|progression)[A-Za-z0-9_]*\s*\(/i,
];

const INPUT_MUTATION_PATTERNS = [
  /\b_(?:value|ctx)(?:\s*(?:\.[A-Za-z_][A-Za-z0-9_]*|\[[^\]]+\]))*\s*(?:=(?!=)|\+=|-=|\*=|\/=|\+\+|--)/i,
  /\bvariable_struct_set\s*\(\s*_(?:value|ctx)\b/i,
  /\barray_(?:push|set)\s*\(\s*_(?:value|ctx)\b/i,
];

const RANDOM_API_PATTERN = /\b(?:i?random)(?:\s*_\s*[A-Za-z0-9_]+)*\s*\(/i;

const SESSION_ID_ALLOWED_SYMBOLS = new Set([
  'mistria_tracker_companion_session_id',
  'string',
  'get_timer',
  'string_copy',
  'string_length',
]);

function forbiddenApiMatches(source: string): string[] {
  return FORBIDDEN_API_PATTERNS
    .map((pattern) => source.match(pattern)?.[0])
    .filter((match): match is string => match !== undefined);
}

function inputMutationMatches(source: string): string[] {
  return INPUT_MUTATION_PATTERNS
    .map((pattern) => source.match(pattern)?.[0])
    .filter((match): match is string => match !== undefined);
}

function randomApiMatches(source: string): string[] {
  return source.match(RANDOM_API_PATTERN) ?? [];
}

function stripGmlComments(source: string): string {
  let stripped = '';
  let index = 0;
  let quote: '"' | "'" | undefined;

  while (index < source.length) {
    const character = source[index];
    const nextCharacter = source[index + 1];

    if (quote !== undefined) {
      stripped += character;
      if (character === '\\' && nextCharacter !== undefined) {
        stripped += nextCharacter;
        index += 2;
        continue;
      }
      if (character === quote) quote = undefined;
      index += 1;
      continue;
    }

    if (character === '"' || character === "'") {
      quote = character;
      stripped += character;
      index += 1;
      continue;
    }

    if (character === '/' && nextCharacter === '/') {
      stripped += ' ';
      index += 2;
      while (index < source.length && source[index] !== '\n' && source[index] !== '\r') index += 1;
      continue;
    }

    if (character === '/' && nextCharacter === '*') {
      stripped += ' ';
      index += 2;
      while (index < source.length && !(source[index] === '*' && source[index + 1] === '/')) index += 1;
      index += 2;
      continue;
    }

    stripped += character;
    index += 1;
  }

  return stripped;
}

function calledFunctionNames(source: string): string[] {
  return [...stripGmlComments(source).matchAll(/\b([A-Za-z_][A-Za-z0-9_]*)\s*\(/g)].map((match) => match[1]);
}

function sessionIdSymbolNames(source: string): string[] {
  return [
    ...calledFunctionNames(source),
    ...[...source.matchAll(/\bcurrent_time\b/g)].map((match) => match[0]),
  ];
}

function extractFunction(source: string, name: string): string {
  let start = -1;
  let bodyStart = -1;
  let depth = 0;
  let state: 'code' | 'line-comment' | 'block-comment' | '"' | "'" = 'code';
  for (let index = 0; index < source.length; index += 1) {
    const character = source[index];
    const nextCharacter = source[index + 1];

    if (state === 'line-comment') {
      if (character === '\n' || character === '\r') state = 'code';
      continue;
    }
    if (state === 'block-comment') {
      if (character === '*' && nextCharacter === '/') {
        state = 'code';
        index += 1;
      }
      continue;
    }
    if (state === '"' || state === "'") {
      if (character === '\\') index += 1;
      else if (character === state) state = 'code';
      continue;
    }
    if (character === '/' && (nextCharacter === '/' || nextCharacter === '*')) {
      state = nextCharacter === '/' ? 'line-comment' : 'block-comment';
      index += 1;
      continue;
    }
    if (character === '"' || character === "'") {
      state = character;
      continue;
    }

    if (start < 0) {
      if (source.startsWith(`function ${name}(`, index)
        && (index === 0 || !/[A-Za-z0-9_]/.test(source[index - 1]))) start = index;
      continue;
    }
    if (bodyStart < 0) {
      if (character !== '{') continue;
      bodyStart = index;
    }

    if (character === '{') depth += 1;
    if (character === '}') depth -= 1;
    if (depth === 0) return stripGmlComments(source.slice(start, index + 1));
  }

  expect(start, `expected ${name} to be declared`).toBeGreaterThanOrEqual(0);
  expect(bodyStart, `expected ${name} to have a body`).toBeGreaterThanOrEqual(0);
  throw new Error(`expected ${name} to have a complete body`);
}

describe('passive companion safety boundary', () => {
  it('offers a safe companion install and optional desktop shortcut', () => {
    const installer = readFileSync(INSTALLER_SCRIPT, 'utf8');

    const componentsPage = installer.indexOf('Page components');
    const directoryPage = installer.indexOf('Page directory');
    expect(componentsPage, 'expected an installer components-choice page').toBeGreaterThanOrEqual(0);
    expect(componentsPage, 'expected the components-choice page before the destination page').toBeLessThan(directoryPage);
    expect(installer).toContain('RequestExecutionLevel admin');
    expect(installer).toContain('Section /o "Live tracking companion (AIM/MOMI, recommended)"');
    expect(installer).toContain('Section /o "Add a desktop shortcut"');
    expect(installer).toContain('CreateShortcut "$DESKTOP\\\\Mistria Tracker.lnk" "$INSTDIR\\\\mistria-tracker.exe"');
    expect(installer).toContain('IfFileExists "$GameDirectory\\\\assets.zip"');
    expect(installer).toContain('${StrRep} $R0 $R0 "/" "\\\\"');
    expect(installer).toContain('StrCpy $CompanionTarget "$GameDirectory\\\\mods\\\\MistriaTrackerCompanion"');
    expect(installer).toMatch(/File .*mistria_tracker_companion.*manifest\.json/);
    expect(installer).toMatch(/File .*mistria_tracker_companion.*MistriaTrackerCompanion\.gml/);
    expect(installer).toContain('manifest.json');
    expect(installer).toMatch(/IfFileExists .*manifest\.json/);
    expect(installer).toMatch(/IfFileExists .*MistriaTrackerCompanion\.gml/);
    expect(installer).not.toContain('RMDir /r');
    expect(installer).not.toContain('.sav');
  });
  it('permits one failure-safe non-mutating filter and no prohibited API family', () => {
    const gml = readFileSync(COMPANION_GML, 'utf8');
    expect(gml.match(/mmapi_filter\s*\(/g)).toHaveLength(1);
    expect(gml).toContain('mmapi_filter("items.give", mistria_tracker_companion_items_give)');
    const handler = extractFunction(gml, 'mistria_tracker_companion_items_give');
    const returns = [...handler.matchAll(/\breturn\s+([^;]+);/g)].map((match) => match[1].trim());
    expect(returns.length).toBeGreaterThan(0);
    expect(new Set(returns)).toEqual(new Set(['undefined']));
    expect(handler).toMatch(/^function mistria_tracker_companion_items_give\([^)]*\)\s*\{\s*try\s*\{/s);
    expect(handler).toMatch(/catch\s*\([^)]*\)\s*\{[\s\S]*?\}\s*return\s+undefined;/);
    expect(inputMutationMatches(handler)).toEqual([]);
    expect(forbiddenApiMatches(gml)).toEqual([]);
    expect(gml.match(/\bmmapi_register\s*\(/g)).toHaveLength(1);
    expect(gml).toContain('mmapi_register(mistria_tracker_companion_tick)');

    const logCalls = [...gml.matchAll(/\b(mmapi_log_[A-Za-z0-9_]+)\s*\(/gi)].map((match) => match[1]);
    expect(new Set(logCalls)).toEqual(new Set(['mmapi_log_info', 'mmapi_log_flush']));
    expect(gml).toContain('mmapi_log_info("mistria_tracker_companion", "MISTRIA_TRACKER_EVENT|" + json_stringify(_event))');

    const mmapiCalls = [...gml.matchAll(/\b(mmapi_[A-Za-z0-9_]+)\s*\(/gi)].map((match) => match[1]);
    expect(new Set(mmapiCalls)).toEqual(new Set([
      'mmapi_mod_declare',
      'mmapi_register',
      'mmapi_filter',
      'mmapi_on',
      'mmapi_log_info',
      'mmapi_log_flush',
    ]));
  });

  it('rejects casing and whitespace bypasses across every forbidden operation family', () => {
    expect(forbiddenApiMatches('FiLe _ DeLeTe ( path )')).not.toEqual([]);
    expect(forbiddenApiMatches('BuFfEr _ LoAd ( source )')).not.toEqual([]);
    expect(forbiddenApiMatches('SaVe _ GaMe ( )')).not.toEqual([]);
    expect(forbiddenApiMatches('DiReCtOrY _ DeLeTe ( )')).not.toEqual([]);
    expect(forbiddenApiMatches('MMAPI _ MoDsAvE _ Register ( )')).not.toEqual([]);
    expect(forbiddenApiMatches('MmApI _ ReGiStEr _ ItEm ( )')).not.toEqual([]);
    expect(forbiddenApiMatches('ObJeCt _ ReGiStEr ( )')).not.toEqual([]);
    expect(forbiddenApiMatches('MoDiFy _ ReLaTiOnShIp ( )')).not.toEqual([]);
  });

  it('rejects direct, bracket, nested, and helper-based input mutation', () => {
    expect(inputMutationMatches('_value = replacement;')).not.toEqual([]);
    expect(inputMutationMatches('_value.count += 1;')).not.toEqual([]);
    expect(inputMutationMatches('_ctx["metadata"]["source"] = "tracker";')).not.toEqual([]);
    expect(inputMutationMatches('variable_struct_set(_ctx, "source", "tracker");')).not.toEqual([]);
    expect(inputMutationMatches('array_push(_value.items, "tracker");')).not.toEqual([]);
  });

  it('allows only the audited pure timer and UUID-slicing symbols in the session identifier', () => {
    const gml = readFileSync(COMPANION_GML, 'utf8');
    const sessionId = extractFunction(gml, 'mistria_tracker_companion_session_id');
    const hashCalls = [...gml.matchAll(/\b([A-Za-z_][A-Za-z0-9_]*)\s*\(/gi)]
      .map((match) => match[1])
      .filter((name) => /(?:sha|md5|hash)/i.test(name));

    expect(randomApiMatches(gml)).toEqual([]);
    expect(hashCalls).toEqual([]);
    expect(new Set(sessionIdSymbolNames(sessionId))).toEqual(SESSION_ID_ALLOWED_SYMBOLS);
    expect(sessionId).toMatch(/\bstring\s*\(\s*get_timer\s*\(\s*\)\s*\)/);
    expect(sessionId).not.toMatch(/\bGame\b/i);
  });

  it('rejects random-family calls and every Game access form in session identifiers', () => {
    expect(randomApiMatches('IrAnDoM _ RaNgE ( 1, 2 )')).not.toEqual([]);
    expect(randomApiMatches('random_set_seed ( 1 )')).not.toEqual([]);
    expect(extractFunction('function mistria_tracker_companion_session_id() { return Game["last_serde_path"]; }', 'mistria_tracker_companion_session_id')).toMatch(/\bGame\b/i);
  });

  it.each([
    'item_id_to_string /* bypass */ (_digest);',
    'item_id_to_string/* bypass */(_digest);',
    'item_id_to_string // bypass\n (_digest);',
    '/* } */ item_id_to_string /* bypass */ (_digest);',
  ])('rejects comment-obscured unapproved session-helper calls: %s', (body) => {
    const sessionId = extractFunction(
      `function mistria_tracker_companion_session_id() { ${body} }`,
      'mistria_tracker_companion_session_id',
    );

    expect(sessionIdSymbolNames(sessionId)).toContain('item_id_to_string');
    expect(sessionIdSymbolNames(sessionId).filter((name) => !SESSION_ID_ALLOWED_SYMBOLS.has(name)))
      .toEqual(['item_id_to_string']);
  });

  it.each([
    String.raw`"}"`,
    String.raw`'}'`,
    String.raw`"\"}/* literal */"`,
    String.raw`'\'}// literal'`,
  ])('detects trailing unsafe session operations after a quoted brace: %s', (literal) => {
    const sessionId = extractFunction(
      `function mistria_tracker_companion_session_id() {
        var _literal = ${literal};
        // } ignored comment brace
        /* { } ignored comment braces */
        item_id_to_string /* bypass */ (_digest);
        var _game = Game["last_serde_path"];
        random /* bypass */ (1);
      }`,
      'mistria_tracker_companion_session_id',
    );

    expect.soft(sessionIdSymbolNames(sessionId).filter((name) => !SESSION_ID_ALLOWED_SYMBOLS.has(name)))
      .toEqual(['item_id_to_string', 'random']);
    expect.soft(sessionId).toMatch(/\bGame\b/i);
    expect.soft(randomApiMatches(sessionId)).not.toEqual([]);
  });

  it.each([
    '// function mistria_tracker_companion_session_id() { return undefined; }',
    '/* function mistria_tracker_companion_session_id() { return undefined; } */',
    'var _decoy = "function mistria_tracker_companion_session_id() { return undefined; }";',
    "var _decoy = 'function mistria_tracker_companion_session_id() { return undefined; }';",
  ])('ignores declarations in comments and strings before the real session helper: %s', (prefix) => {
    const sessionId = extractFunction(
      `${prefix}\nfunction mistria_tracker_companion_session_id() {
        item_id_to_string /* bypass */ (_digest);
      }`,
      'mistria_tracker_companion_session_id',
    );

    expect(sessionIdSymbolNames(sessionId).filter((name) => !SESSION_ID_ALLOWED_SYMBOLS.has(name)))
      .toEqual(['item_id_to_string']);
    expect(sessionId).toContain('item_id_to_string');
  });

  it.each([
    ') /* {} */',
    ') // {}\n',
    '_literal = "{}")',
    "_literal = '{}')",
  ])('ignores comment and string braces before the real session-helper body: %s', (signatureEnd) => {
    const sessionId = extractFunction(
      `function mistria_tracker_companion_session_id(${signatureEnd} {
        item_id_to_string /* bypass */ (_digest);
      }`,
      'mistria_tracker_companion_session_id',
    );

    expect(sessionIdSymbolNames(sessionId).filter((name) => !SESSION_ID_ALLOWED_SYMBOLS.has(name)))
      .toEqual(['item_id_to_string']);
    expect(sessionId).toContain('item_id_to_string');
  });

  it('preserves quoted strings, escaped quotes, line endings, and token boundaries when stripping comments', () => {
    const quoted = String.raw`"https://example.test/* literal */\"//still quoted" '/* literal */\'//still quoted'`;
    expect(stripGmlComments(`${quoted} /* removed */ // removed\r\nreturn/* removed */item_id_to_string(_digest);`))
      .toBe(`${quoted}    \r\nreturn item_id_to_string(_digest);`);
    expect(calledFunctionNames('item_id_to_string/* bypass */(_digest);'))
      .toEqual(['item_id_to_string']);
  });

  it('declares every registered hook in the manifest and namespaces companion code', () => {
    const gml = readFileSync(COMPANION_GML, 'utf8');
    const manifest = JSON.parse(readFileSync(MANIFEST, 'utf8')) as { requires_hooks: string[] };
    const registeredHooks = [...gml.matchAll(/mmapi_(?:on|filter)\(\s*"([^"]+)"/g)].map((match) => match[1]);

    expect(registeredHooks).toEqual([
      'items.give',
      'npc.gift_received',
      'museum.donate_item',
      'game.room_changed',
    ]);
    expect(manifest.requires_hooks).toEqual(registeredHooks);

    const functionNames = [...gml.matchAll(/\bfunction\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(/g)].map((match) => match[1]);
    expect(functionNames).not.toHaveLength(0);
    expect(functionNames.every((name) => name.startsWith('mistria_tracker_companion_'))).toBe(true);

    const globalNames = [...gml.matchAll(/\bglobal\.([A-Za-z_][A-Za-z0-9_]*)/g)].map((match) => match[1]);
    expect(globalNames).not.toHaveLength(0);
    expect(globalNames.every((name) => name.startsWith('mistria_tracker_companion_'))).toBe(true);
  });

  it('accepts MMAPI event contexts as one struct and leaves every callback observational', () => {
    const gml = readFileSync(COMPANION_GML, 'utf8');
    const gift = extractFunction(gml, 'mistria_tracker_companion_gift_received');
    const museum = extractFunction(gml, 'mistria_tracker_companion_museum_donate_item');
    const room = extractFunction(gml, 'mistria_tracker_companion_room_changed');

    expect(gift).toMatch(/^function mistria_tracker_companion_gift_received\(_ctx\)/);
    expect(gift).toContain('_ctx.npc');
    expect(gift).toContain('_ctx.item');
    expect(museum).toMatch(/^function mistria_tracker_companion_museum_donate_item\(_ctx\)/);
    expect(museum).toContain('_ctx.item_id');
    expect(room).toMatch(/^function mistria_tracker_companion_room_changed\(_ctx\)/);
    expect(inputMutationMatches(`${gift}\n${museum}\n${room}`)).toEqual([]);
  });

  it('does not derive a profile identifier until a loaded save path is available', () => {
    const gml = readFileSync(COMPANION_GML, 'utf8');
    const profile = extractFunction(gml, 'mistria_tracker_companion_profile_id');

    expect(profile).toMatch(/var _path = undefined;/);
    expect(profile).toMatch(/try\s*\{\s*_path = Game\.last_serde_path;/);
    expect(profile).toMatch(/if \(_path == undefined\) return undefined;/);
    expect(profile.indexOf('if (_path == undefined) return undefined;'))
      .toBeLessThan(profile.indexOf('filename_name(_path)'));
  });

  it('uses only a timer-derived UUID-shaped session identifier', () => {
    const gml = readFileSync(COMPANION_GML, 'utf8');
    const sessionId = extractFunction(gml, 'mistria_tracker_companion_session_id');

    expect(sessionId).not.toContain('sha1_string_utf8');
    expect(sessionId).toContain('"00000000-0000-7000-8000-"');
    expect(sessionId).toContain('get_timer()');
  });

});

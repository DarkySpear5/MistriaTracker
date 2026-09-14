// Read-only schema inspection of an explicitly supplied backup copy.
const fs = require('node:fs');
const zlib = require('node:zlib');
const bytes = zlib.inflateSync(fs.readFileSync(process.argv[2]), { maxOutputLength: 128 * 1024 * 1024 });
let offset = 0;
const integer = () => { const result = Number(bytes.readBigUInt64LE(offset)); offset += 8; return result; };
const take = () => { const length = integer(); const result = bytes.subarray(offset, offset + length).toString('utf8'); offset += length; return result; };
const count = integer();
const shape = (value) => Array.isArray(value) ? `array(${value.length}) ${JSON.stringify(value.slice(0, 1).map(v => typeof v === 'object' ? Object.keys(v ?? {}) : typeof v))}` : value && typeof value === 'object' ? Object.fromEntries(Object.entries(value).map(([key, item]) => [key, Array.isArray(item) ? `array(${item.length})` : typeof item])) : typeof value;
for (let index = 0; index < count; index++) {
  const name = take(); const value = JSON.parse(take());
  if (['header', 'player', 'gamedata', 'game_stats'].includes(name)) console.log(name, JSON.stringify(shape(value)));
  if (name === 'gamedata') console.log('museum sample type', typeof value.museum_progress?.[0]);
  if (name === 'game_stats') for (const key of ['npcs_spoken_to', 'gifts_given', 'fish_caught', 'items_cooked', 'animals', 'end_of_day_stats']) console.log(key, JSON.stringify(shape(Array.isArray(value[key]) ? value[key][0] : value[key])));
  if (name === 'player') for (const key of ['recipe_unlocks', 'recipes_created', 'cosmetic_unlocks', 'animal_variant_unlocks', 'date_unlocks', 'song_unlocks']) console.log(key, JSON.stringify(shape(value[key])));
  if (name === 'npcs') { const npc = Object.values(value)[0]; console.log('npc', JSON.stringify(shape(npc))); for (const key of ['gifts_given', 'known_gift_preferences']) console.log(key, JSON.stringify(shape(npc[key]))); }
}

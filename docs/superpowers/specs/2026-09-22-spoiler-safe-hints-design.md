# Spoiler-safe discovery hints (testing branch)

## Intent and scope

The next testing-branch implementation replaces vague hidden-slot hints with
useful acquisition clues. A hint may reveal a broad area, season, activity, or
way to obtain a recipe. It must never contain an undiscovered item's name,
identifier, image, exact spawn point, or a villager's untried reaction. Hints
remain optional in spoiler-free mode. Game saves and companion behavior do not
change. This work belongs to the 0.1.7 testing line; it does not add an
updater or produce an installer.

## Source of truth

Read only the already validated game `assets.zip`. Extend the existing bounded
TOML catalog reader to admit these specific definitions:

| Category | Authoritative game metadata | Allowed clue |
| --- | --- | --- |
| Fish | `fish.toml` spawn seasons, water type, retrieval, special region | Pond, river, ocean, mines, deep woods; season |
| Bugs | `bugs.toml` spawn seasons, tags, dungeon biome | Beach, deep woods, mines, outdoors; season |
| Crops | `object_prototypes/crop.toml` seasons | Grow a seed; season |
| Forageables | `forageables.toml` season lists and sand list; crop prototype seasons | Outdoors or beach; season |
| Artifacts | `artifacts.toml` area-to-group map and archaeology museum-set members | Confirmed broad area; otherwise excavation advice |
| Recipes | `stores.toml`, `letters.toml`, quest, museum, festival and random-reward definitions; item `recipe_is_default` | Shop, mail, request, museum reward, random reward, starting recipe, or generic scroll advice |
| Gifts | The associated item's validated category and availability metadata | Activity and broad area/season, never the item or reaction |

For recipes, direct `recipe_scroll` or `crafting_scroll` references are
authoritative. A shop's `include_recipe = true` item links through its
`recipe_key`. When a recipe has several confirmed sources, prefer an available
shop, then mail, quest, museum, random reward, then starting recipe. Text
searches in item descriptions are never used as source evidence.

## Data boundary

The catalog keeps internal facts in a typed `HintFacts` value. The journal
snapshot sends only a typed `Hint` with whitelisted category, area, season,
and source codes. Every hidden entry still has `id`, `name`, `description`,
`places`, `seasons`, and detailed `sources` withheld as today. Hidden gift
slots get the same limited hint facts without an item identifier or name.
Unexpected, malformed, or missing metadata yields the category's generic
clue. A mined fish overrides the fish default water type; it must not appear
as a river hint. Duplicate or contradictory source references are resolved in
a fixed order.

## Presentation

The existing hint setting and click action remain. The dialog shows one short
category-specific sentence and, when supported, localized chips for broad
area, season, activity, or recipe acquisition. For example, a hidden fish may
show `Pond · Summer`. A recipe sold by a vendor may say `Buy a recipe scroll
from a store`; a random reward says `Random reward`. Gift hints describe the
type of activity involved, without naming the gift. All new interface text
has entries in each of the eight testing-branch languages and follows the one
unified language preference.

## Verification and release boundary

Focused tests cover each metadata source, wrong/default mappings, source
priority, hidden-entry and hidden-gift privacy, and localized dialog output.
Run the Rust and frontend suites plus type/build checks after implementation.
Inspect hints against the installed 1.0.5 archive without reading a game save.
The work remains on `testing` for manual game review; there is no merge,
public release, updater, or installer build in this stage.

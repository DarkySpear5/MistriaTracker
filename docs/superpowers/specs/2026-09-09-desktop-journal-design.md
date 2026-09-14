# Mistria desktop journal

Approved direction: user references and subsequent permission to make the remaining design choices, 2026-09-09. This specification supersedes the old flat Collections/Items presentation. Implementation proceeds in the existing build worktree.

## Experience and appearance

A finished Windows companion with a cozy dusk palette, compact application chrome, persistent sidebar, fixed search toolbar, independent content scrolling, and a detail drawer. Preserve the functioning Tauri tracker, local icon pack, SQLite history, notes, and companion protocol. No hosted service, accounts, subscription, external runtime font requests, or game overlay.

Sidebar order: Overview, Museum, Villagers, Encyclopaedia; Settings anchored at the bottom. Overview remains the data dashboard. Recipes and Cooked Dishes are separate Encyclopaedia categories. Use a restrained blue-green background (#131d26), layered surfaces (#1b2833, #223440), warm off-white text (#f3eee4), muted sage (#acd0b4), lavender (#bba7dd), and small gold accents (#dfbc7c). Corners 10–16px; 8px spacing rhythm; readable 14–15px Segoe UI Variable/Segoe UI on Windows, stronger weights rather than serif headings. No emoji as primary application icons. Pixel artwork uses nearest-neighbor rendering. Hover, focus, selection, missing, completed, and disabled states must be distinct. Reduced-motion and keyboard navigation are supported. Never rely on color alone for completion.

## Shared identity and evidence

Each item has one stable identity with explicit category and links to museum sets, recipes, and villager gift relations. Recipe learned, dish acquired, item donated, villager met, and gift preference discovered are separate facts. Catalog membership is not evidence of completion. All numbers are derived from verified tracker evidence; absent measurements are unavailable, never invented zeros or sample charts. Unknown or mismatched game data must not overwrite existing progress.

Extract the exact museum sets and NPC tables from installed game assets. Use canonical structured fish/bug availability, item tags, and source paths for categories. Keep source labels and definitions in English and French with English fallback. Keep all extracted game resources local and excluded from source control. Pack preparation occurs at startup only when game assets change. Request artwork only for visible cards through a bounded queue and cache successful and missing results; do not reopen assets.zip per card.

## Navigation and browsing

Encyclopaedia opens a category launcher grid. Each category tile has an icon, name, and count. Selecting a category opens its catalog in the same window and closes the previous category. Back restores the launcher. Filters and sorting sit beside the search box; season, place, discovery state, category, and name/completion order are applied only where meaningful. Remember navigation and selection while opening and closing item details.

Museum shows Archaeology, Fish, Flora, and Insects as four balanced columns at wide desktop sizes, reflowing to fewer columns at smaller sizes. Each column contains the game's named sets and compact icon slots. A donated check and a distinct acquired state prevent confusion between found and donated. Set and wing progress counts remain visible.

Villagers uses portrait cards with name, separate Loved and Liked gift grids, known birthday/bio details, and consistent spacing. Unmet villagers occupy anonymous black silhouette cards. Existing NPC gift evidence must be imported from the approved backup with schema validation; had_arrived alone does not mean the player met that NPC. Use recorded conversations/gifts to establish a meeting.

Overview is a calm dashboard, not a wall of charts. Show overall item discoveries, museum donations, known recipes, villagers met, and per-category progress from real evidence. Show richer play statistics only when their meaning and units are verified. Category cards and progress bars navigate into their catalog. Do not imply that the fraction of arbitrary catalog entries equals total game completion or Steam achievement completion.

## Search, details, and spoilers

Search opens multiple matching results as the user types. Match contiguous text case-insensitively and accent-insensitively; Trou matches Trout. Search only allowed identities. Current-page matches rank first. Selecting a Museum match stays in Museum; other item matches open the correct Encyclopaedia category. No automatic navigation while typing. Escape closes results; arrow keys and Enter select results.

The detail drawer contains permitted artwork, name, description, season/place information, notes, and a compact Also appears in area linking to other permitted contexts. A discovered item must not reveal an unknown villager's identity or an undiscovered gift preference through a related link. Recipe ingredients must obey the same policy. Search and sorting cannot reveal hidden names.

Spoiler-free: undiscovered slots are black with no identifying name, tooltip, alt text, hidden DOM label, searchable identity, or accessible real artwork URL. Unknown villagers stay anonymous. Hints only appear on click when both spoiler-free and hints are enabled; use deliberately broad category-derived guidance in both languages, not an item name or an exact method/location. With spoilers enabled, all catalog identities are visible; missing entries are greyed and discovered/completed entries render normally. A black hidden silhouette is an intentional final state, not a missing asset placeholder.

## Reliability and delivery

Scope remains read-only for game assets and the explicitly approved Desktop backup. No source save, game installation, or companion changes are needed for this redesign. Store additional verified backup evidence only in the matching tracker profile. Preserve notes, imported history and restart persistence. Never merge profiles. Compatibility failures must leave useful existing views available and show an accurate status rather than a permanent generic paused banner.

Ship a packaged native executable with the UI bundled inside, using the existing launch path. Test actual assets and approved backup read-only, spoiler serialization, category membership, distinct recipe/donation/gift evidence, contextual search, filters, navigation, notes, reduced-motion CSS, and a native-size visual render. Review at 1100×760 and a larger desktop size. The acceptance target is a coherent durable edition with real data and working interactions; future game updates may still require maintenance.

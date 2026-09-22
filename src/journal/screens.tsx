import { useState } from "react";
import { genericHint, type Entry, type Filters, type Hint, type Journal, type Language, type View } from "./types";
import { categoryOrder, tr } from "./copy";
import { filterEntries } from "./search";
import { Icon } from "./Icon";
import { Artwork } from "./Artwork";
export type ScreenProps = {
  journal: Journal;
  language: Language;
  onEntry: (key: string) => void;
  onHint: (hint: Hint) => void;
  onNavigate: (view: View, category?: string) => void;
  filters: Filters;
};
export function Meter({ value, total }: { value: number; total: number }) {
  return (
    <div
      className="meter"
      role="progressbar"
      aria-valuemin={0}
      aria-valuemax={total || 1}
      aria-valuenow={value}
    >
      <span
        style={{
          width: `${total ? Math.min(100, (value / total) * 100) : 0}%`,
        }}
      />
    </div>
  );
}
export function PageHeading({
  title,
  subtitle,
  children,
}: {
  title: string;
  subtitle: string;
  children?: React.ReactNode;
}) {
  return (
    <header className="page-heading">
      <div>
        <h1>{title}</h1>
        <p>{subtitle}</p>
      </div>
      {children}
    </header>
  );
}
export function EntrySlot({
  entry,
  language,
  onEntry,
  onHint,
  museum = false,
}: {
  entry: Entry;
  language: Language;
  onEntry: (key: string) => void;
  onHint: (hint: Hint) => void;
  museum?: boolean;
}) {
  return (
    <button
      className={`entry-slot ${!entry.revealed ? "slot-hidden" : !entry.found ? "slot-missing" : ""} ${museum && entry.donated ? "slot-donated" : ""}`}
      title={
        entry.revealed ? (entry.name ?? undefined) : tr(language, "unknownItem")
      }
      aria-label={
        entry.revealed
          ? `${entry.name}${museum ? ` · ${tr(language, entry.donated ? "donated" : "notDonated")}` : ""}`
          : tr(language, "unknownItem")
      }
      onClick={() =>
        entry.revealed ? onEntry(entry.key) : onHint(entry.hint ?? genericHint(entry.category))
      }
    >
      <Artwork
        token={entry.art}
        kind={entry.category}
        hidden={!entry.revealed}
      />
      {museum && entry.donated && (
        <span className="slot-check">
          <Icon name="check" size={10} />
        </span>
      )}
    </button>
  );
}
export function Overview(props: ScreenProps) {
  const { journal, language, onNavigate } = props;
  const items = journal.entries.filter(
    (entry) =>
      !entry.id?.includes(":") &&
      entry.category !== "recipes" &&
      entry.category !== "perks" &&
      entry.category !== "scrolls" &&
      entry.category !== "invitations",
  );
  const allDonations = journal.sets.reduce(
      (sum, set) => sum + set.completed,
      0,
    ),
    totalDonations = journal.sets.reduce((sum, set) => sum + set.total, 0);
  const recipes = journal.categories.find(
    (category) => category.id === "recipes",
  );
  const stats = [
    {
      label: "discoveries",
      icon: "encyclopedia",
      done: items.filter((item) => item.found).length,
      total: items.length,
      target: "encyclopedia",
    },
    {
      label: "donations",
      icon: "museum",
      done: allDonations,
      total: totalDonations,
      target: "museum",
    },
    {
      label: "recipesKnown",
      icon: "recipes",
      done: recipes?.completed ?? 0,
      total: recipes?.total ?? 0,
      target: "recipes",
    },
    {
      label: "villagersMet",
      icon: "villagers",
      done: journal.villagers.filter((npc) => npc.met).length,
      total: journal.villagers.length,
      target: "villagers",
    },
  ];
  return (
    <>
      <PageHeading
        title={tr(language, "yourJourney")}
        subtitle={tr(language, "welcome")}
      />
      <div className="journey-banner">
        <div className="journey-emblem">
          <Icon name="moon" size={38} />
          <i />
          <i />
          <i />
        </div>
        <div>
          <span className="eyebrow">
            {journal.profile.farm || "FIELDS OF MISTRIA"}
          </span>
          <h2>{journal.profile.name || tr(language, "yourJourney")}</h2>
          <p>{tr(language, "scopeNote")}</p>
        </div>
        <div className="banner-sprig">
          <Icon name="crops" size={90} />
        </div>
      </div>
      <div className="stat-grid">
        {stats.map((stat) => (
          <button
            className="stat-card"
            key={stat.label}
            onClick={() =>
              onNavigate(
                stat.target === "recipes"
                  ? "encyclopedia"
                  : (stat.target as View),
                stat.target === "recipes" ? "recipes" : undefined,
              )
            }
          >
            <span className={`stat-icon tone-${stat.icon}`}>
              <Icon name={stat.icon} />
            </span>
            <span className="stat-label">{tr(language, stat.label)}</span>
            <strong>
              {stat.done.toLocaleString()}
              <small> / {stat.total.toLocaleString()}</small>
            </strong>
            <Meter value={stat.done} total={stat.total} />
          </button>
        ))}
      </div>
      <div className="section-title">
        <h2>{tr(language, "collectionProgress")}</h2>
        <button
          className="text-button"
          onClick={() => onNavigate("encyclopedia")}
        >
          {tr(language, "encyclopedia")}
          <Icon name="chevron" size={15} />
        </button>
      </div>
      <div className="progress-grid">
        {categoryOrder
          .map((id) =>
            journal.categories.find((category) => category.id === id),
          )
          .filter((category) => category !== undefined)
          .map((category) => (
            <button
              className="category-progress"
              key={category.id}
              onClick={() => onNavigate("encyclopedia", category.id)}
            >
              <span className={`category-icon tone-${category.id}`}>
                <Icon name={category.id} size={23} />
              </span>
              <div>
                <div className="progress-label">
                  <span>{tr(language, category.id)}</span>
                  <small>
                    {category.completed} / {category.total}
                  </small>
                </div>
                <Meter value={category.completed} total={category.total} />
              </div>
              <strong>
                {category.total
                  ? Math.round((category.completed / category.total) * 100)
                  : 0}
                %
              </strong>
            </button>
          ))}
      </div>
      {Object.keys(journal.stats).length > 0 && (
        <>
          <div className="section-title">
            <h2>{tr(language, "recorded")}</h2>
          </div>
          <div className="activity-grid">
            {Object.entries(journal.stats).map(([id, value]) => (
              <div className="activity-card" key={id}>
                <span>{tr(language, id)}</span>
                <strong>{value.toLocaleString()}</strong>
              </div>
            ))}
          </div>
        </>
      )}
    </>
  );
}
export function Encyclopedia(props: ScreenProps & { category: string | null }) {
  const { journal, language, onNavigate, category, filters } = props;
  const [limit, setLimit] = useState(60);
  if (!category)
    return (
      <>
        <PageHeading
          title={tr(language, "encyclopedia")}
          subtitle={tr(language, "encyclopediaHelp")}
        />
        <div className="category-grid">
          {categoryOrder
            .map((id) =>
              journal.categories.find((category) => category.id === id),
            )
            .filter((category) => category !== undefined)
            .map((category) => (
              <button
                className="category-tile"
                key={category.id}
                onClick={() => {
                  setLimit(60);
                  onNavigate("encyclopedia", category.id);
                }}
              >
                <span className={`category-icon tone-${category.id}`}>
                  <Icon name={category.id} size={31} />
                </span>
                <Icon name="chevron" className="tile-chevron" size={16} />
                <h2>{tr(language, category.id)}</h2>
                <span>
                  {category.completed} / {category.total}
                </span>
                <Meter value={category.completed} total={category.total} />
              </button>
            ))}
        </div>
      </>
    );
  const entries = filterEntries(
    journal.entries.filter((entry) => entry.category === category),
    filters,
  );
  return (
    <>
      <button
        className="text-button back-button"
        onClick={() => onNavigate("encyclopedia")}
      >
        <Icon name="back" size={16} />
        {tr(language, "back")}
      </button>
      <PageHeading
        title={tr(language, category)}
        subtitle={`${entries.length} ${tr(language, "results")}`}
      >
        <span className="category-icon">
          <Icon name={category} size={28} />
        </span>
      </PageHeading>
      <div className="catalog-grid">
        {entries.slice(0, limit).map((entry) => (
          <button
            className={`catalog-card ${!entry.revealed ? "card-hidden" : !entry.found ? "card-missing" : ""}`}
            key={entry.key}
            onClick={() =>
              entry.revealed
                ? props.onEntry(entry.key)
                : props.onHint(entry.hint ?? genericHint(category))
            }
          >
            <div className="catalog-art">
              <Artwork
                token={entry.art}
                kind={category}
                large
                hidden={!entry.revealed}
              />
              {entry.found && (
                <span className="found-badge">
                  <Icon name="check" size={12} />
                </span>
              )}
            </div>
            <span className="catalog-name">
              {entry.name || tr(language, "unknownItem")}
            </span>
            <small>
              {entry.revealed
                ? entry.seasons
                    .map((season) => tr(language, season))
                    .join(" · ") ||
                  tr(language, entry.found ? "found" : "missing")
                : tr(language, "spoilerFree")}
            </small>
          </button>
        ))}
      </div>
      {!entries.length && (
        <div className="empty-results">
          <Icon name="filter" size={32} />
          <p>{tr(language, "noFilterResults")}</p>
        </div>
      )}
      {entries.length > limit && (
        <button className="more-button" onClick={() => setLimit(limit + 60)}>
          {tr(language, "viewMore")}
          <small>
            {Math.min(limit, entries.length)} / {entries.length}
          </small>
        </button>
      )}
    </>
  );
}
export function Museum(props: ScreenProps) {
  const { journal, language, filters } = props;
  const entries = new Map(journal.entries.map((entry) => [entry.key, entry]));
  const allowed = new Set(
    filterEntries(
      journal.entries.map((entry) => ({ ...entry, found: entry.donated })),
      filters,
    ).map((entry) => entry.key),
  );
  return (
    <>
      <PageHeading
        title={tr(language, "museum")}
        subtitle={tr(language, "museumHelp")}
      />
      <p className="page-tip">
        <Icon name="info" size={15} />
        {tr(language, "countHelp")}
      </p>
      <div className="museum-columns">
        {["archaeology", "fish", "flora", "insect"].map((wing) => {
          const sets = journal.sets
            .filter((set) => set.wing === wing)
            .sort((a, b) => {
              const ratio =
                (b.total ? b.completed / b.total : 0) -
                (a.total ? a.completed / a.total : 0);
              if (filters.sort === "found" && ratio) return ratio;
              if (filters.sort === "missing" && ratio) return -ratio;
              return (
                a.name.localeCompare(b.name) *
                (filters.sort === "name-desc" ? -1 : 1)
              );
            });
          const done = sets.reduce((sum, set) => sum + set.completed, 0),
            total = sets.reduce((sum, set) => sum + set.total, 0);
          return (
            <div key={wing} className={`museum-wing wing-${wing}`}>
              <header>
                <div className="museum-emblem">
                  <Icon name={wing} size={35} />
                </div>
                <h2>{tr(language, wing)}</h2>
                <span>
                  {done} / {total}
                </span>
                <Meter value={done} total={total} />
              </header>
              {sets
                .filter((set) => set.items.some((key) => allowed.has(key)))
                .map((set) => (
                  <section
                    id={`set-${set.id}`}
                    className={`museum-set ${set.completed === set.total ? "set-complete" : ""}`}
                    key={set.id}
                  >
                    <div className="set-heading">
                      <h3>{set.name}</h3>
                      <small>
                        {set.completed}/{set.total}
                      </small>
                    </div>
                    <div className="set-slots">
                      {set.items
                        .map((key) => entries.get(key))
                        .filter(
                          (entry): entry is Entry =>
                            entry !== undefined && allowed.has(entry.key),
                        )
                        .map((entry) => (
                          <EntrySlot
                            key={entry.key}
                            entry={entry}
                            language={language}
                            onEntry={props.onEntry}
                            onHint={props.onHint}
                            museum
                          />
                        ))}
                    </div>
                  </section>
                ))}
            </div>
          );
        })}
      </div>
    </>
  );
}
export function Villagers(props: ScreenProps) {
  const { journal, language, filters } = props;
  const villagers = journal.villagers
    .filter(
      (npc) =>
        filters.state === "all" || npc.met === (filters.state === "found"),
    )
    .sort((a, b) => {
      const met = Number(b.met) - Number(a.met);
      if (filters.sort === "found" && met) return met;
      if (filters.sort === "missing" && met) return -met;
      if (a.revealed !== b.revealed) return a.revealed ? -1 : 1;
      return (
        (a.name ?? "").localeCompare(b.name ?? "") *
        (filters.sort === "name-desc" ? -1 : 1)
      );
    });
  return (
    <>
      <PageHeading
        title={tr(language, "villagers")}
        subtitle={tr(language, "villagersHelp")}
      />
      <div className="villager-grid">
        {villagers.map((npc) => (
          <article
            className={`villager-card ${!npc.revealed ? "villager-hidden" : ""}`}
            key={npc.key}
            id={`villager-${npc.key}`}
          >
            <header>
              <div className="portrait-glow" />
              <Artwork
                token={npc.art}
                kind="villagers"
                large
                hidden={!npc.revealed}
              />
              <h2>{npc.name || tr(language, "unknownVillager")}</h2>
              {!npc.revealed && <p>{tr(language, "hiddenHelp")}</p>}
            </header>
            {npc.revealed && (
              <>
                {(["loved", "liked"] as const).map((group) => (
                  <section className="gift-group" key={group}>
                    <h3>
                      <Icon
                        name={group === "loved" ? "heart" : "spark"}
                        size={13}
                      />
                      {tr(language, group)}
                      <span>
                        {npc[group].filter((gift) => gift.found).length}/
                        {npc[group].length}
                      </span>
                    </h3>
                    <div className="gift-slots">
                      {npc[group].map((gift) => (
                        <button
                          className={`entry-slot ${!gift.revealed ? "slot-hidden" : !gift.found ? "slot-missing" : ""}`}
                          key={gift.key}
                          title={gift.name || tr(language, "unknownItem")}
                          aria-label={gift.name || tr(language, "unknownItem")}
                          onClick={() =>
                            gift.revealed && gift.entry
                              ? props.onEntry(gift.entry)
                              : props.onHint(gift.hint ?? genericHint("gift"))
                          }
                        >
                          <Artwork
                            token={gift.art}
                            hidden={!gift.revealed}
                            kind="heart"
                          />
                        </button>
                      ))}
                    </div>
                  </section>
                ))}
                {npc.bio && (
                  <details className="villager-bio">
                    <summary>{tr(language, "details")}</summary>
                    <p>{npc.bio}</p>
                  </details>
                )}
              </>
            )}
          </article>
        ))}
      </div>
    </>
  );
}

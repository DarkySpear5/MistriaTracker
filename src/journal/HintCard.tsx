import type { Hint, Language } from "./types";
import { tr } from "./copy";
import { Icon } from "./Icon";

const kinds = new Set(["fish", "bugs", "crops", "forageables", "artifacts", "recipes", "gift"]);
const areas = new Set(["pond", "river", "ocean", "mines", "beach", "deep_woods", "outdoors", "narrows", "eastern_road", "haydens_farm", "western_ruins", "farm"]);
const seasons = new Set(["spring", "summer", "fall", "winter"]);
const sources = new Set(["store", "mail", "quest", "museum", "random", "start"]);
const areaKey = (code: string) => ["pond", "river", "ocean", "mines"].includes(code) ? code : `hintArea_${code}`;

export function HintCard({ hint, language, onClose }: { hint: Hint; language: Language; onClose: () => void }) {
  const kind = kinds.has(hint.kind) ? hint.kind : "generic";
  const activity = hint.activity && kinds.has(hint.activity) ? hint.activity : null;
  return (
    <>
      <button autoFocus className="icon-button" aria-label={tr(language, "close")} onClick={onClose}>
        <Icon name="close" />
      </button>
      <Icon name="spark" size={30} />
      <h2 id="hint-title">{tr(language, "hintTitle")}</h2>
      <p>{tr(language, `hint_${kind}`)}</p>
      {activity && kind === "gift" && <p>{tr(language, "hintGiftActivity", { activity: tr(language, activity) })}</p>}
      {hint.areas.filter((code) => areas.has(code)).length > 0 && (
        <p>{tr(language, "place")}: {hint.areas.filter((code) => areas.has(code)).map((code, index) => (
          <span key={code}>{index > 0 ? " · " : ""}{tr(language, areaKey(code))}</span>
        ))}</p>
      )}
      {hint.seasons.filter((code) => seasons.has(code)).length > 0 && (
        <p>{tr(language, "season")}: {hint.seasons.filter((code) => seasons.has(code)).map((code, index) => (
          <span key={code}>{index > 0 ? " · " : ""}{tr(language, code)}</span>
        ))}</p>
      )}
      {kind === "recipes" && hint.source && sources.has(hint.source) && <p>{tr(language, `hintSource_${hint.source}`)}</p>}
    </>
  );
}

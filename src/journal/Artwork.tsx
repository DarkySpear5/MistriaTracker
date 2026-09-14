import {
  createContext,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { invoke } from "@tauri-apps/api/core";
import { Icon } from "./Icon";
type ArtLoader = (keys: string[]) => Promise<Record<string, string | null>>;
type Store = {
  read: (key: string) => string | null | undefined;
  subscribe: (key: string, listener: () => void) => () => void;
};
const Context = createContext<Store | null>(null);
const nativeLoader: ArtLoader = (keys) => invoke("get_journal_art", { keys });
export function ArtworkProvider({
  children,
  loader = nativeLoader,
  scope,
}: {
  children: React.ReactNode;
  loader?: ArtLoader;
  scope: string;
}) {
  const store = useMemo(() => {
    const values = new Map<string, string | null>();
    const waiting = new Map<string, Set<() => void>>();
    const queue = new Set<string>();
    const pending = new Set<string>();
    let scheduled = false;
    let active = false;
    const drain = async () => {
      scheduled = false;
      if (active || !queue.size) return;
      active = true;
      const batch = [...queue].slice(0, 24);
      batch.forEach((key) => {
        queue.delete(key);
        pending.add(key);
      });
      try {
        const result = await loader([
          ...new Set(batch.map((key) => key.slice(2))),
        ]);
        for (const key of batch) values.set(key, result[key.slice(2)] ?? null);
      } catch {
        for (const key of batch) values.set(key, null);
      } finally {
        batch.forEach((key) => {
          pending.delete(key);
          waiting.get(key)?.forEach((fn) => fn());
        });
        active = false;
        if (queue.size) void drain();
      }
    };
    return {
      read: (key: string) => values.get(key),
      subscribe: (key: string, listener: () => void) => {
        if (!waiting.has(key)) waiting.set(key, new Set());
        waiting.get(key)!.add(listener);
        if (!values.has(key) && !pending.has(key)) {
          queue.add(key);
          if (!scheduled) {
            scheduled = true;
            queueMicrotask(() => void drain());
          }
        }
        return () => {
          waiting.get(key)?.delete(listener);
          if (!waiting.get(key)?.size) queue.delete(key);
        };
      },
    };
  }, [loader, scope]);
  return <Context.Provider value={store}>{children}</Context.Provider>;
}
export function Artwork({
  token,
  kind = "materials",
  large = false,
  hidden = false,
}: {
  token?: string | null;
  kind?: string;
  large?: boolean;
  hidden?: boolean;
}) {
  const store = useContext(Context);
  const ref = useRef<HTMLSpanElement>(null);
  const [, render] = useState(0);
  const [visible, setVisible] = useState(false);
  useEffect(() => {
    if (!ref.current) return;
    if (!("IntersectionObserver" in window)) {
      setVisible(true);
      return;
    }
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries.some((entry) => entry.isIntersecting)) {
          setVisible(true);
          observer.disconnect();
        }
      },
      { rootMargin: "100px" },
    );
    observer.observe(ref.current);
    return () => observer.disconnect();
  }, []);
  const cacheKey = token ? `${hidden ? "h" : "v"}|${token}` : null;
  useEffect(() => {
    if (!cacheKey || !store || !visible) return;
    return store.subscribe(cacheKey, () => render((value) => value + 1));
  }, [store, cacheKey, visible]);
  const url = cacheKey ? store?.read(cacheKey) : null;
  return (
    <span
      ref={ref}
      className={`artwork ${large ? "artwork-large" : ""} ${hidden ? "artwork-hidden" : ""}`}
      aria-hidden="true"
    >
      {url ? (
        <img src={url} alt="" decoding="async" />
      ) : (
        <Icon
          name={hidden && kind === "villagers" ? "person" : kind}
          size={large ? 72 : 26}
        />
      )}
    </span>
  );
}

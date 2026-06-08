// Mini React — minimal createElement/render/useState/useEffect implementation.
// No virtual DOM diffing — direct DOM manipulation for build verification.

export type VNode = {
  tag: string | Function | typeof Fragment;
  props: Record<string, any>;
  children: (VNode | string)[];
};

// Fragment symbol for <></> support (R5)
export const Fragment = Symbol.for("mini-react.fragment");

// --- createElement ---

export function createElement(
  tag: string | Function,
  props: Record<string, any> | null,
  ...children: any[]
): VNode {
  return {
    tag,
    props: props || {},
    children: children.flat().map((c) =>
      typeof c === "object" && c !== null ? c : String(c ?? "")
    ),
  };
}

// --- Hooks state ---

let currentComponent: (() => VNode) | null = null;
let currentHookIndex = 0;
const hookStates = new Map<Function, any[]>();
const hookEffects = new Map<Function, { cb: () => void; deps: any[] | undefined }[]>();
let rerenderQueue: Set<Function> = new Set();
let scheduledRerender = false;

// --- useState ---

export function useState<T>(initial: T): [T, (v: T | ((prev: T) => T)) => void] {
  const comp = currentComponent!;
  const idx = currentHookIndex++;

  if (!hookStates.has(comp)) hookStates.set(comp, []);
  const states = hookStates.get(comp)!;

  if (states.length <= idx) {
    states.push(initial);
  }

  const setState = (value: T | ((prev: T) => T)) => {
    const current = hookStates.get(comp)![idx];
    const next = typeof value === "function" ? (value as (prev: T) => T)(current) : value;
    if (next !== current) {
      hookStates.get(comp)![idx] = next;
      rerenderQueue.add(comp);
      scheduleRerender();
    }
  };

  return [states[idx], setState];
}

// --- useEffect ---

export function useEffect(callback: () => void, deps?: any[]): void {
  const comp = currentComponent!;
  const idx = currentHookIndex++;

  if (!hookEffects.has(comp)) hookEffects.set(comp, []);
  const effects = hookEffects.get(comp)!;

  if (effects.length <= idx) {
    effects.push({ cb: callback, deps });
    // Run on mount
    queueMicrotask(callback);
  } else {
    const prev = effects[idx];
    const depsChanged = !deps || !prev.deps || deps.some((d, i) => d !== prev.deps![i]);
    if (depsChanged) {
      effects[idx] = { cb: callback, deps };
      queueMicrotask(callback);
    }
  }
}

// --- render ---

let rootContainer: HTMLElement | null = null;
let rootComponent: (() => VNode) | null = null;

export function render(vnode: VNode | (() => VNode), container: HTMLElement): void {
  if (typeof vnode === "function") {
    rootComponent = vnode;
    rootContainer = container;
    doRender();
  } else {
    container.innerHTML = "";
    container.appendChild(createDOM(vnode));
  }
}

function doRender(): void {
  if (!rootComponent || !rootContainer) return;

  currentComponent = rootComponent;
  currentHookIndex = 0;
  const vnode = rootComponent();
  currentComponent = null;

  rootContainer.innerHTML = "";
  rootContainer.appendChild(createDOM(vnode));

  // Run effects
  rerenderQueue.clear();
}

function scheduleRerender(): void {
  if (scheduledRerender) return;
  scheduledRerender = true;
  queueMicrotask(() => {
    scheduledRerender = false;
    doRender();
  });
}

// --- DOM creation ---

function createDOM(vnode: VNode | string): Node {
  if (typeof vnode === "string") {
    return document.createTextNode(vnode);
  }

  // Fragment support: render children without a wrapper element
  if (vnode.tag === Fragment) {
    const frag = document.createDocumentFragment();
    for (const child of vnode.children) {
      if (child === "false" || child === "null" || child === "undefined") continue;
      frag.appendChild(createDOM(child));
    }
    return frag;
  }

  if (typeof vnode.tag === "function") {
    currentComponent = vnode.tag as unknown as () => VNode;
    currentHookIndex = 0;
    const result = (vnode.tag as Function)(vnode.props);
    currentComponent = null;
    return createDOM(result);
  }

  const el = document.createElement(vnode.tag as string);

  for (const [key, value] of Object.entries(vnode.props)) {
    if (key === "className") {
      el.setAttribute("class", value);
    } else if (key === "style" && typeof value === "object") {
      Object.assign(el.style, value);
    } else if (key.startsWith("on") && typeof value === "function") {
      const event = key.slice(2).toLowerCase();
      el.addEventListener(event, value);
    } else if (key === "checked") {
      (el as HTMLInputElement).checked = value;
    } else if (key === "value") {
      (el as HTMLInputElement).value = value;
    } else if (typeof value === "boolean") {
      if (value) el.setAttribute(key, "");
    } else {
      el.setAttribute(key, String(value));
    }
  }

  for (const child of vnode.children) {
    if (child === "false" || child === "null" || child === "undefined") continue;
    el.appendChild(createDOM(child));
  }

  return el;
}

// --- JSX namespace for TypeScript ---

export const h = createElement;

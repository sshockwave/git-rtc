'use client';

import { DependencyList, useEffect, useState } from "react";

export function useAbort(effect: (signal: AbortSignal) => void, deps?: DependencyList) {
  useEffect(() => {
    const controller = new AbortController();
    effect(controller.signal);
    return () => controller.abort();
  }, deps);
}

export function useRerender() {
  const [, set] = useState({});
  return () => set({});
}

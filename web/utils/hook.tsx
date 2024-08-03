'use client';

import { DependencyList, useEffect } from "react";

export function useAbort(effect: (signal: AbortSignal) => void, deps: DependencyList) {
  useEffect(() => {
    const controller = new AbortController();
    effect(controller.signal);
    return () => controller.abort();
  }, deps);
}

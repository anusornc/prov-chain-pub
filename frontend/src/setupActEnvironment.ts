// React reads this flag before setupFilesAfterEnv is evaluated. Keep it in
// Jest setupFiles so React 19 treats Testing Library act() scopes as supported.
(globalThis as typeof globalThis & { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;

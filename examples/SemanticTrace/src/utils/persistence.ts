type Persistence = {
  setItem(key: string, value: string): Promise<void>;
  getItem(key: string): Promise<string | null>;
  removeItem(key: string): Promise<void>;
  clear(): Promise<void>;
};

const hasPersistentStorage = typeof (window as any).persistentStorage !== 'undefined';

const storage: {
  setItem(k: string, v: string): Promise<void> | void;
  getItem(k: string): Promise<string | null> | string | null;
  removeItem(k: string): Promise<void> | void;
  clear(): Promise<void> | void;
} = hasPersistentStorage ? (window as any).persistentStorage : {
  setItem(k: string, v: string) {
    try {
      localStorage.setItem(k, v);
    } catch (e) {
      // ignore
    }
  },
  getItem(k: string) {
    try {
      return localStorage.getItem(k);
    } catch (e) {
      return null;
    }
  },
  removeItem(k: string) {
    try {
      localStorage.removeItem(k);
    } catch (e) {
      // ignore
    }
  },
  clear() {
    try {
      localStorage.clear();
    } catch (e) {
      // ignore
    }
  }
};

export const persistence: Persistence = {
  async setItem(key, value) {
    const res = storage.setItem(key, value);
    if (res instanceof Promise) await res;
  },
  async getItem(key) {
    const res = storage.getItem(key);
    if (res instanceof Promise) return await res;
    return res as string | null;
  },
  async removeItem(key) {
    const res = storage.removeItem(key);
    if (res instanceof Promise) await res;
  },
  async clear() {
    const res = storage.clear();
    if (res instanceof Promise) await res;
  }
};
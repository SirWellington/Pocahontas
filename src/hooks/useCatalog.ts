import { create } from "zustand";
import { CatalogState } from "../types";
import * as catalog from "../services/catalog";

interface CatalogStore extends CatalogState {
  selectedImageIds: Set<number>;
  currentPage: number;
  pageSize: number;

  // Actions
  openCatalog: (path: string) => Promise<void>;
  importDirectory: (dirPath: string) => Promise<number>;
  loadImages: (page?: number) => Promise<void>;
  selectImage: (id: number) => void;
  deselectImage: (id: number) => void;
  clearSelection: () => void;
  setPage: (page: number) => void;
}

const PAGE_SIZE = 60;

export const useCatalogStore = create<CatalogStore>((set, get) => ({
  path: null,
  images: [],
  totalImages: 0,
  isLoading: false,
  error: null,
  selectedImageIds: new Set(),
  currentPage: 0,
  pageSize: PAGE_SIZE,

  openCatalog: async (path: string) => {
    set({ isLoading: true, error: null });
    try {
      await catalog.openCatalog(path);
      set({ path, isLoading: false });
      await get().loadImages(0);
    } catch (e) {
      set({
        error: e instanceof Error ? e.message : "Failed to open catalog",
        isLoading: false,
      });
    }
  },

  importDirectory: async (dirPath: string) => {
    set({ isLoading: true, error: null });
    try {
      const imported = await catalog.importDirectory(dirPath);
      set({ isLoading: false });
      await get().loadImages(0);
      return imported;
    } catch (e) {
      set({
        error: e instanceof Error ? e.message : "Import failed",
        isLoading: false,
      });
      throw e;
    }
  },

  loadImages: async (page?: number) => {
    const { currentPage, pageSize } = get();
    const targetPage = page ?? currentPage;
    set({ isLoading: true });

    try {
      const [images, total] = await Promise.all([
        catalog.listImages(targetPage * pageSize, pageSize),
        catalog.countImages(),
      ]);
      set({
        images,
        totalImages: total,
        currentPage: targetPage,
        isLoading: false,
      });
    } catch (e) {
      set({
        error: e instanceof Error ? e.message : "Failed to load images",
        isLoading: false,
      });
    }
  },

  selectImage: (id: number) => {
    const { selectedImageIds } = get();
    const next = new Set(selectedImageIds);
    next.add(id);
    set({ selectedImageIds: next });
  },

  deselectImage: (id: number) => {
    const { selectedImageIds } = get();
    const next = new Set(selectedImageIds);
    next.delete(id);
    set({ selectedImageIds: next });
  },

  clearSelection: () => {
    set({ selectedImageIds: new Set() });
  },

  setPage: (page: number) => {
    set({ currentPage: page });
    get().loadImages(page);
  },
}));

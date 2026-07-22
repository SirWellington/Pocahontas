import { create } from "zustand";
import { CatalogState, SidebarView, FolderRecord } from "@/types";
import { generateMockImages, generateMockFolders } from "@/data/mock";

interface CatalogStore extends CatalogState {
  selectedImageIds: Set<number>;
  folders: FolderRecord[];
  activeView: SidebarView;
  leftPanelVisible: boolean;
  rightPanelVisible: boolean;

  openCatalog: (path: string) => Promise<void>;
  importDirectory: (dirPath: string) => Promise<number>;
  selectImage: (id: number) => void;
  deselectImage: (id: number) => void;
  clearSelection: () => void;
  setActiveView: (view: SidebarView) => void;
  toggleLeftPanel: () => void;
  toggleRightPanel: () => void;
  updateRating: (id: number, rating: number) => void;
  toggleFavorite: (id: number) => void;
}

export const useCatalogStore = create<CatalogStore>((set, get) => ({
  path: null,
  images: [],
  totalImages: 0,
  isLoading: false,
  error: null,
  selectedImageIds: new Set(),
  folders: [],
  activeView: "library",
  leftPanelVisible: true,
  rightPanelVisible: true,

  openCatalog: async (path: string) => {
    set({ isLoading: true, error: null });
    // Simulate async load
    await new Promise((r) => setTimeout(r, 300));
    const images = generateMockImages(60);
    const folders = generateMockFolders();
    set({
      path,
      images,
      totalImages: images.length,
      folders,
      isLoading: false,
    });
  },

  importDirectory: async (_dirPath: string) => {
    await new Promise((r) => setTimeout(r, 500));
    const current = get().images;
    const newImages = generateMockImages(12).map((img, i) => ({
      ...img,
      id: current.length + i + 1,
    }));
    const updated = [...current, ...newImages];
    set({
      images: updated,
      totalImages: updated.length,
    });
    return newImages.length;
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

  setActiveView: (view: SidebarView) => {
    set({ activeView: view });
  },

  toggleLeftPanel: () => {
    set((s) => ({ leftPanelVisible: !s.leftPanelVisible }));
  },

  toggleRightPanel: () => {
    set((s) => ({ rightPanelVisible: !s.rightPanelVisible }));
  },

  updateRating: (id: number, rating: number) => {
    set((s) => ({
      images: s.images.map((img) =>
        img.id === id ? { ...img, rating } : img
      ),
    }));
  },

  toggleFavorite: (id: number) => {
    set((s) => ({
      images: s.images.map((img) =>
        img.id === id ? { ...img, is_favorite: !img.is_favorite } : img
      ),
    }));
  },
}));

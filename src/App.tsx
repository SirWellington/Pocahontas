import React, { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { useCatalogStore } from "../hooks/useCatalog";
import GalleryGrid from "./GalleryGrid";
import MetadataPanel from "./MetadataPanel";

const App: React.FC = () => {
  const { path, isLoading, error, importDirectory, clearSelection } =
    useCatalogStore();
  const [showMetadata, setShowMetadata] = useState(true);

  const handleOpenCatalog = async () => {
    const selected = await open({
      multiple: false,
      filters: [
        {
          name: "Praetorian Catalog",
          extensions: ["praetorian"],
        },
      ],
    });
    if (selected) {
      await useCatalogStore.getState().openCatalog(selected);
    }
  };

  const handleImportDirectory = async () => {
    const selected = await open({
      directory: true,
      multiple: false,
    });
    if (selected) {
      await importDirectory(selected);
    }
  };

  return (
    <div className="h-screen w-screen flex flex-col bg-gray-950">
      {/* Top toolbar */}
      <div className="toolbar">
        <div className="flex items-center gap-2">
          <svg className="w-5 h-5 text-blue-400" fill="currentColor" viewBox="0 0 24 24">
            <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z" />
          </svg>
          <span className="font-semibold text-sm">Praetorian</span>
        </div>

        <div className="flex-1" />

        {path && (
          <>
            <button className="btn-secondary" onClick={handleOpenCatalog}>
              Open Catalog
            </button>
            <button className="btn-primary" onClick={handleImportDirectory}>
              Import
            </button>
            <button
              className="btn-secondary"
              onClick={() => setShowMetadata(!showMetadata)}
            >
              {showMetadata ? "Hide" : "Show"} Details
            </button>
          </>
        )}
      </div>

      {/* Main content area */}
      <div className="flex-1 flex overflow-hidden">
        {/* Gallery */}
        <div className="flex-1 flex flex-col overflow-hidden">
          {!path ? (
            <div className="flex-1 flex flex-col items-center justify-center gap-4">
              <svg
                className="w-16 h-16 text-gray-600"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={1.5}
                  d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"
                />
              </svg>
              <p className="text-gray-400 text-sm">
                No catalog open. Create or open a catalog to get started.
              </p>
              <button className="btn-primary" onClick={handleOpenCatalog}>
                Open Catalog
              </button>
            </div>
          ) : isLoading ? (
            <div className="flex-1 flex items-center justify-center">
              <div className="w-8 h-8 border-2 border-blue-400 border-t-transparent rounded-full animate-spin" />
            </div>
          ) : (
            <GalleryGrid />
          )}
        </div>

        {/* Right metadata panel */}
        {showMetadata && path && <MetadataPanel />}
      </div>

      {/* Error toast */}
      {error && (
        <div className="absolute bottom-4 right-4 bg-red-900/90 border border-red-700 text-red-100 px-4 py-2 rounded text-sm">
          {error}
        </div>
      )}
    </div>
  );
};

export default App;

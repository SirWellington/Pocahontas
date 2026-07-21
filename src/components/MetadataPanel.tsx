import React from "react";
import { useCatalogStore } from "../hooks/useCatalog";

const MetadataPanel: React.FC = () => {
  const { selectedImageIds, images } = useCatalogStore();

  const selectedId = selectedImageIds.values().next().value;
  const selectedImage = images.find((img) => img.id === selectedId);

  if (!selectedImage) {
    return (
      <div className="sidebar-panel">
        <div className="p-4 text-gray-500 text-sm text-center flex-1 flex items-center justify-center">
          Select an image to view details
        </div>
      </div>
    );
  }

  return (
    <div className="sidebar-panel">
      <div className="p-4 border-b border-gray-800">
        <h3 className="font-medium text-sm truncate">{selectedImage.file_name}</h3>
        <p className="text-xs text-gray-500 mt-1">
          {selectedImage.width} x {selectedImage.height}
        </p>
      </div>

      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        {/* File info */}
        <section>
          <h4 className="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-2">
            File
          </h4>
          <dl className="space-y-1 text-sm">
            <div className="flex justify-between">
              <dt className="text-gray-500">Type</dt>
              <dd className="text-gray-200">{selectedImage.file_extension.toUpperCase()}</dd>
            </div>
            <div className="flex justify-between">
              <dt className="text-gray-500">Size</dt>
              <dd className="text-gray-200">
                {(selectedImage.file_size_bytes / 1024 / 1024).toFixed(1)} MB
              </dd>
            </div>
            <div className="flex justify-between">
              <dt className="text-gray-500">Imported</dt>
              <dd className="text-gray-200">
                {new Date(selectedImage.date_imported).toLocaleDateString()}
              </dd>
            </div>
          </dl>
        </section>

        {/* Camera info */}
        <section>
          <h4 className="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-2">
            Camera
          </h4>
          <dl className="space-y-1 text-sm">
            <div className="flex justify-between">
              <dt className="text-gray-500">Date Taken</dt>
              <dd className="text-gray-200">
                {selectedImage.date_taken
                  ? new Date(selectedImage.date_taken).toLocaleString()
                  : "Unknown"}
              </dd>
            </div>
          </dl>
        </section>

        {/* Rating */}
        <section>
          <h4 className="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-2">
            Rating
          </h4>
          <div className="flex gap-1">
            {[1, 2, 3, 4, 5].map((star) => (
              <button
                key={star}
                className={`w-6 h-6 ${
                  star <= selectedImage.rating
                    ? "text-yellow-400"
                    : "text-gray-600"
                }`}
              >
                <svg fill="currentColor" viewBox="0 0 24 24">
                  <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z" />
                </svg>
              </button>
            ))}
          </div>
        </section>

        {/* Status */}
        <section>
          <h4 className="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-2">
            Status
          </h4>
          <div className="space-y-1 text-sm">
            <div className="flex justify-between">
              <dt className="text-gray-500">Thumbnail</dt>
              <dd className={selectedImage.has_thumbnail ? "text-green-400" : "text-gray-600"}>
                {selectedImage.has_thumbnail ? "Generated" : "Pending"}
              </dd>
            </div>
            <div className="flex justify-between">
              <dt className="text-gray-500">Preview</dt>
              <dd className={selectedImage.has_preview ? "text-green-400" : "text-gray-600"}>
                {selectedImage.has_preview ? "Generated" : "Pending"}
              </dd>
            </div>
            <div className="flex justify-between">
              <dt className="text-gray-500">Faces</dt>
              <dd className={selectedImage.faces_indexed ? "text-green-400" : "text-gray-600"}>
                {selectedImage.faces_indexed ? "Indexed" : "Not indexed"}
              </dd>
            </div>
          </div>
        </section>
      </div>
    </div>
  );
};

export default MetadataPanel;

import { useCatalogStore } from "@/hooks/useCatalog";

const DetailsPanel: React.FC = () => {
  const { selectedImageIds, images, updateRating, toggleFavorite } =
    useCatalogStore();

  const selectedId = selectedImageIds.values().next().value;
  const selectedImage = images.find((img) => img.id === selectedId);

  if (!selectedImage) {
    return (
      <div className="w-72 bg-[#1e1e1e] border-l border-[#2a2a2a] flex flex-col overflow-hidden shrink-0">
        <div className="flex-1 flex items-center justify-center">
          <p className="text-[12px] text-[#555]">Select an image to view details</p>
        </div>
      </div>
    );
  }

  const exif = selectedImage.exif;

  return (
    <div className="w-72 bg-[#1e1e1e] border-l border-[#2a2a2a] flex flex-col overflow-hidden shrink-0">
      {/* Image name header */}
      <div className="px-3 py-2 border-b border-[#2a2a2a]">
        <h3 className="text-[12px] font-medium text-[#d4d4d4] truncate">
          {selectedImage.file_name}
        </h3>
        <p className="text-[11px] text-[#555] mt-0.5">
          {selectedImage.width} &times; {selectedImage.height} &middot;{" "}
          {(selectedImage.file_size_bytes / 1024 / 1024).toFixed(1)} MB
        </p>
      </div>

      <div className="flex-1 overflow-y-auto">
        {/* Rating */}
        <PanelSection title="Rating">
          <div className="flex items-center gap-3">
            <div className="flex gap-0.5">
              {[1, 2, 3, 4, 5].map((star) => (
                <button
                  key={star}
                  className={`w-5 h-5 transition-colors ${
                    star <= selectedImage.rating
                      ? "text-yellow-400"
                      : "text-[#444] hover:text-[#666]"
                  }`}
                  onClick={() => updateRating(selectedImage.id, star)}
                >
                  <svg fill="currentColor" viewBox="0 0 24 24">
                    <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z" />
                  </svg>
                </button>
              ))}
            </div>
            <button
              className={`w-5 h-5 transition-colors ${
                selectedImage.is_favorite ? "text-red-400" : "text-[#444] hover:text-[#666]"
              }`}
              onClick={() => toggleFavorite(selectedImage.id)}
            >
              <svg
                fill={selectedImage.is_favorite ? "currentColor" : "none"}
                stroke="currentColor"
                viewBox="0 0 24 24"
                strokeWidth={2}
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  d="M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z"
                />
              </svg>
            </button>
          </div>
        </PanelSection>

        {/* Camera */}
        <PanelSection title="Camera">
          <KeyValue label="Make" value={exif?.camera_make ?? "—"} />
          <KeyValue label="Model" value={exif?.camera_model ?? "—"} />
          <KeyValue label="Lens" value={exif?.lens_model ?? "—"} />
        </PanelSection>

        {/* Exposure */}
        <PanelSection title="Exposure">
          <KeyValue label="ISO" value={exif?.iso?.toString() ?? "—"} />
          <KeyValue
            label="Aperture"
            value={exif?.aperture ? `f/${exif.aperture}` : "—"}
          />
          <KeyValue
            label="Shutter"
            value={
              exif?.shutter_speed_den
                ? `1/${exif.shutter_speed_den}s`
                : "—"
            }
          />
          <KeyValue
            label="Focal Length"
            value={exif?.focal_length ? `${exif.focal_length}mm` : "—"}
          />
        </PanelSection>

        {/* Date */}
        <PanelSection title="Date">
          <KeyValue
            label="Taken"
            value={
              selectedImage.date_taken
                ? new Date(selectedImage.date_taken).toLocaleString()
                : "—"
            }
          />
          <KeyValue
            label="Imported"
            value={new Date(selectedImage.date_imported).toLocaleDateString()}
          />
        </PanelSection>

        {/* Location */}
        {exif?.gps_latitude != null && (
          <PanelSection title="Location">
            <KeyValue
              label="Latitude"
              value={exif.gps_latitude.toFixed(6)}
            />
            <KeyValue
              label="Longitude"
              value={exif.gps_longitude!.toFixed(6)}
            />
            {exif.gps_altitude != null && (
              <KeyValue
                label="Altitude"
                value={`${exif.gps_altitude}m`}
              />
            )}
          </PanelSection>
        )}

        {/* Processing Status */}
        <PanelSection title="Processing">
          <StatusRow label="Thumbnail" done={selectedImage.has_thumbnail} />
          <StatusRow label="Smart Preview" done={selectedImage.has_preview} />
          <StatusRow label="Faces Indexed" done={selectedImage.faces_indexed} />
        </PanelSection>
      </div>
    </div>
  );
};

function PanelSection({
  title,
  children,
}: {
  title: string;
  children: React.ReactNode;
}) {
  return (
    <div className="px-3 py-2.5 border-b border-[#2a2a2a]/50">
      <h4 className="text-[10px] font-semibold text-[#555] uppercase tracking-wider mb-1.5">
        {title}
      </h4>
      <div className="space-y-0.5">{children}</div>
    </div>
  );
}

function KeyValue({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex justify-between items-baseline">
      <span className="text-[11px] text-[#666]">{label}</span>
      <span className="text-[11px] text-[#ccc] font-mono">{value}</span>
    </div>
  );
}

function StatusRow({ label, done }: { label: string; done: boolean }) {
  return (
    <div className="flex justify-between items-center">
      <span className="text-[11px] text-[#666]">{label}</span>
      <span
        className={`text-[10px] font-medium px-1.5 py-0.5 rounded ${
          done
            ? "text-green-400/80 bg-green-400/10"
            : "text-[#555] bg-[#2a2a2a]"
        }`}
      >
        {done ? "Done" : "Pending"}
      </span>
    </div>
  );
}

export default DetailsPanel;

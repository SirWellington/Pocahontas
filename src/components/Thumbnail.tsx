import { memo } from "react";
import { ImageRecord } from "../types";

interface ThumbnailProps {
  image: ImageRecord;
  isSelected: boolean;
  onSelect: (id: number) => void;
}

const Thumbnail = memo(({ image, isSelected, onSelect }: ThumbnailProps) => {
  const thumbSrc = image.has_thumbnail
    ? `file://cache/thumb_${image.id}.jpg`
    : null;

  return (
    <div
      className={`thumbnail-cell ${isSelected ? "selected" : ""}`}
      onClick={() => onSelect(image.id)}
    >
      {thumbSrc ? (
        <img
          src={thumbSrc}
          alt={image.file_name}
          className="w-full h-full object-cover"
          loading="lazy"
        />
      ) : (
        <div className="w-full h-full flex items-center justify-center text-gray-600 text-xs">
          <svg
            className="w-6 h-6"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"
            />
          </svg>
        </div>
      )}

      {image.rating > 0 && (
        <div className="absolute bottom-0 left-0 right-0 flex justify-center gap-0.5 py-1 bg-gradient-to-t from-black/60 to-transparent">
          {Array.from({ length: image.rating }).map((_, i) => (
            <svg
              key={i}
              className="w-3 h-3 text-yellow-400 fill-current"
              viewBox="0 0 24 24"
            >
              <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z" />
            </svg>
          ))}
        </div>
      )}
    </div>
  );
});

Thumbnail.displayName = "Thumbnail";
export default Thumbnail;

import React, { useCallback, useRef } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { useCatalogStore } from "../hooks/useCatalog";
import Thumbnail from "./Thumbnail";

const COLUMNS = 6;
const ITEM_HEIGHT = 200;

const GalleryGrid: React.FC = () => {
  const { images, selectedImageIds, selectImage, deselectImage } =
    useCatalogStore();

  const parentRef = useRef<HTMLDivElement>(null);

  const totalItems = images.length;
  const totalRows = Math.ceil(totalItems / COLUMNS);

  const virtualizer = useVirtualizer({
    count: totalRows,
    getScrollElement: () => parentRef.current,
    estimateSize: () => ITEM_HEIGHT,
    overscan: 5,
  });

  const handleSelect = useCallback(
    (id: number) => {
      if (selectedImageIds.has(id)) {
        deselectImage(id);
      } else {
        selectImage(id);
      }
    },
    [selectedImageIds, selectImage, deselectImage]
  );

  return (
    <div
      ref={parentRef}
      className="w-full h-full overflow-auto"
      style={{ contain: "strict" }}
    >
      <div
        style={{
          height: virtualizer.getTotalSize(),
          width: "100%",
          position: "relative",
        }}
      >
        {virtualizer.getVirtualItems().map((virtualRow) => {
          const startIndex = virtualRow.start * COLUMNS;
          const rowImages = images.slice(startIndex, startIndex + COLUMNS);

          return (
            <div
              key={virtualRow.key}
              style={{
                position: "absolute",
                top: 0,
                left: 0,
                width: "100%",
                height: virtualRow.size,
                transform: `translateY(${virtualRow.start}px)`,
                display: "flex",
                gap: "4px",
                padding: "4px",
              }}
            >
              {rowImages.map((img) => (
                <div
                  key={img.id}
                  style={{ flex: `0 0 calc((100% - ${4 * (COLUMNS - 1)}px) / ${COLUMNS})` }}
                >
                  <Thumbnail
                    image={img}
                    isSelected={selectedImageIds.has(img.id)}
                    onSelect={handleSelect}
                  />
                </div>
              ))}
            </div>
          );
        })}
      </div>
    </div>
  );
};

export default GalleryGrid;

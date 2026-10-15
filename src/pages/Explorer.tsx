import { useState, useEffect, useMemo, useRef } from 'react';
import { useWorkspace } from '../hooks/useWorkspace';
import { useDataset } from '../hooks/useDataset';
import { invoke, convertFileSrc } from '@tauri-apps/api/tauri';
import { UnifiedDataset, UnifiedImage, UnifiedAnnotation } from '../types/dataset';

function useThumbnails(imagePaths: string[]) {
  const [thumbnails, setThumbnails] = useState<Record<string, string>>({});
  const requestedRef = useRef<Set<string>>(new Set());
  const key = imagePaths.join('|');

  useEffect(() => {
    const toLoad = imagePaths.filter((p) => p && !requestedRef.current.has(p));
    if (toLoad.length === 0) return;

    toLoad.forEach((p) => requestedRef.current.add(p));

    invoke<string[]>('generate_thumbnails_batch', {
      imagePaths: toLoad,
      maxSize: 200,
    })
      .then((results) => {
        setThumbnails((prev) => {
          const next = { ...prev };
          toLoad.forEach((path, idx) => {
            if (results[idx]) next[path] = results[idx];
          });
          return next;
        });
      })
      .catch((err) => {
        console.error('Failed to load thumbnails:', err);
        toLoad.forEach((p) => requestedRef.current.delete(p));
      });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [key]);

  return thumbnails;
}

export default function Explorer() {
  const { currentWorkspace } = useWorkspace();
  const { dataset, loading, error, scanWorkspace, progress, images } = useDataset();
  const [page, setPage] = useState(1);
  const [pageSize] = useState(50);
  const [selectedImage, setSelectedImage] = useState<UnifiedImage | null>(null);
  const [filterClass, setFilterClass] = useState<string | null>(null);
  const [search, setSearch] = useState('');
  const [debouncedSearch, setDebouncedSearch] = useState('');

  const workspacePath = currentWorkspace?.path ?? null;

  useEffect(() => {
    if (!workspacePath) return;
    if (!dataset || dataset.info.path !== workspacePath) {
      scanWorkspace(workspacePath, { emit_progress: true });
    }
  }, [workspacePath, dataset, scanWorkspace]);

  useEffect(() => {
    const timer = setTimeout(() => setDebouncedSearch(search), 300);
    return () => clearTimeout(timer);
  }, [search]);

  useEffect(() => {
    setPage(1);
  }, [debouncedSearch, filterClass]);

  const filteredImages = useMemo(() => {
    if (!dataset || images.length === 0) return [];

    const categoryMap = new Map(dataset.categories.map((c) => [c.id, c.name]));
    const needle = debouncedSearch.trim().toLowerCase();

    return images.filter((img) => {
      if (needle && !img.file_name.toLowerCase().includes(needle)) return false;
      if (filterClass) {
        return img.annotations.some((ann) => categoryMap.get(ann.category_id) === filterClass);
      }
      return true;
    });
  }, [images, dataset, debouncedSearch, filterClass]);

  const totalPages = Math.max(1, Math.ceil(filteredImages.length / pageSize));
  const currentPage = Math.min(page, totalPages);
  const paginatedImages = filteredImages.slice((currentPage - 1) * pageSize, currentPage * pageSize);

  const imagePaths = useMemo(
    () => paginatedImages.map((img) => stripFileProtocol(img.source_path)),
    [paginatedImages],
  );
  const thumbnails = useThumbnails(imagePaths);

  if (!currentWorkspace) {
    return <div className="text-center py-20 text-gray-500">Select a workspace first</div>;
  }

  if (loading) {
    return (
      <div className="flex flex-col items-center justify-center h-64 space-y-4">
        <div className="animate-spin rounded-full h-12 w-12 border-4 border-blue-500 border-t-transparent"></div>
        <div className="text-sm text-gray-600">
          {progress
            ? `Scanning: ${progress.current} / ${progress.total} (${progress.stage})`
            : 'Scanning dataset...'}
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="text-center py-20 text-red-600">
        <h2 className="text-xl font-semibold">Error loading dataset</h2>
        <p className="mt-2 break-all">{error}</p>
      </div>
    );
  }

  return (
    <div className="space-y-6 flex-1 flex flex-col min-h-0">
      {/* Toolbar */}
      <div className="bg-white rounded-xl border border-gray-100 shadow-sm p-4 flex flex-col md:flex-row md:items-center justify-between gap-4 flex-shrink-0">
        <div className="flex items-center space-x-4">
          <h2 className="text-lg font-semibold text-gray-900">Dataset Explorer</h2>
          <span className="px-2 py-1 text-sm bg-gray-100 text-gray-600 rounded-full">
            {filteredImages.length} / {images.length} images
          </span>
        </div>
        <div className="flex items-center space-x-4 flex-wrap">
          <input
            type="text"
            placeholder="Search filenames..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="px-3 py-2 border border-gray-300 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 w-64"
          />
          <select
            value={filterClass || ''}
            onChange={(e) => setFilterClass(e.target.value || null)}
            className="px-3 py-2 border border-gray-300 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          >
            <option value="">All Classes</option>
            {dataset?.categories.map((cat) => (
              <option key={cat.id} value={cat.name}>
                {cat.name} ({dataset.info.class_distribution[cat.name] || 0})
              </option>
            ))}
          </select>
        </div>
      </div>

      {/* Gallery */}
      <div className="bg-white rounded-xl border border-gray-100 shadow-sm flex-1 overflow-hidden flex flex-col min-h-[400px]">
        <div className="flex-1 overflow-auto p-4">
          {paginatedImages.length === 0 ? (
            <div className="flex items-center justify-center h-full text-gray-500 min-h-[300px]">
              <div className="text-center">
                <svg className="mx-auto h-12 w-12" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
                </svg>
                <p className="mt-4">No images found</p>
              </div>
            </div>
          ) : (
            <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4">
              {paginatedImages.map((img) => (
                <ImageThumb
                  key={img.id}
                  image={img}
                  selected={selectedImage?.id === img.id}
                  onClick={() => setSelectedImage(img)}
                  thumbnail={thumbnails[stripFileProtocol(img.source_path)]}
                />
              ))}
            </div>
          )}
        </div>

        {/* Pagination */}
        {totalPages > 1 && (
          <div className="px-4 py-4 border-t border-gray-100 flex items-center justify-between flex-shrink-0">
            <span className="text-sm text-gray-500">
              Page {currentPage} of {totalPages} ({filteredImages.length} images)
            </span>
            <div className="flex items-center space-x-2">
              <button
                onClick={() => setPage((p) => Math.max(1, p - 1))}
                disabled={currentPage === 1}
                className="px-3 py-1.5 text-sm border border-gray-300 rounded-lg disabled:opacity-50 hover:bg-gray-50"
              >
                Previous
              </button>
              <button
                onClick={() => setPage((p) => Math.min(totalPages, p + 1))}
                disabled={currentPage === totalPages}
                className="px-3 py-1.5 text-sm border border-gray-300 rounded-lg disabled:opacity-50 hover:bg-gray-50"
              >
                Next
              </button>
            </div>
          </div>
        )}
      </div>

      {/* Detail Modal */}
      {selectedImage && dataset && (
        <ImageDetailModal
          image={selectedImage}
          dataset={dataset}
          onClose={() => setSelectedImage(null)}
        />
      )}
    </div>
  );
}

function stripFileProtocol(path: string): string {
  return path.replace(/^file:\/\//, '');
}

function imageSrc(path: string, thumbnail?: string): string {
  if (thumbnail && thumbnail.length > 0) return thumbnail;
  try {
    return convertFileSrc(path);
  } catch {
    return path;
  }
}

function ImageThumb({ image, selected, onClick, thumbnail }: { image: UnifiedImage; selected: boolean; onClick: () => void; thumbnail?: string }) {
  const hasAnnotations = image.annotations.length > 0;
  const path = stripFileProtocol(image.source_path);

  return (
    <button
      onClick={onClick}
      className={`relative aspect-square rounded-lg overflow-hidden border-2 bg-gray-50 transition-all w-full ${
        selected ? 'border-blue-500 ring-2 ring-blue-500' : 'border-gray-100 hover:border-gray-300'
      }`}
      title={image.file_name}
    >
      <img
        src={imageSrc(path, thumbnail)}
        alt={image.file_name}
        className="w-full h-full object-cover"
        loading="lazy"
        onError={(e) => {
          (e.currentTarget as HTMLImageElement).style.opacity = '0.2';
        }}
      />
      {hasAnnotations && (
        <div className="absolute top-2 right-2">
          <span className="px-1.5 py-0.5 text-xs font-medium bg-black/70 text-white rounded">
            {image.annotations.length}
          </span>
        </div>
      )}
      <div className="absolute bottom-0 left-0 right-0 p-2 bg-gradient-to-t from-black/70 to-transparent">
        <p className="text-xs text-white truncate">{image.file_name}</p>
      </div>
    </button>
  );
}

function ImageDetailModal({ image, dataset, onClose }: { image: UnifiedImage; dataset: UnifiedDataset; onClose: () => void }) {
  const path = stripFileProtocol(image.source_path);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/80 p-4" onClick={onClose}>
      <div className="bg-white rounded-xl max-w-4xl w-full max-h-[90vh] flex flex-col" onClick={(e) => e.stopPropagation()}>
        <div className="p-4 border-b border-gray-100 flex items-center justify-between">
          <h3 className="font-semibold text-gray-900 truncate">{image.file_name}</h3>
          <button onClick={onClose} className="p-2 hover:bg-gray-100 rounded-lg" aria-label="Close">
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
        <div className="flex-1 overflow-auto flex flex-col md:flex-row">
          <div className="relative flex-1 flex items-center justify-center bg-gray-900/5 p-4">
            <div className="relative inline-block">
              <img src={imageSrc(path)} alt={image.file_name} className="block max-w-full max-h-[70vh]" />
              {image.annotations.map((ann: UnifiedAnnotation, idx: number) => {
                const cat = dataset.categories.find((c) => c.id === ann.category_id);
                const bbox = ann.bbox;
                const w = image.width || 1;
                const h = image.height || 1;
                return (
                  <div
                    key={idx}
                    className="absolute border-2"
                    style={{
                      left: `${(bbox.x / w) * 100}%`,
                      top: `${(bbox.y / h) * 100}%`,
                      width: `${(bbox.width / w) * 100}%`,
                      height: `${(bbox.height / h) * 100}%`,
                      borderColor: cat?.color || '#3B82F6',
                    }}
                  >
                    <div
                      className="absolute -top-5 left-0 px-1 py-0.5 text-xs font-medium text-white rounded whitespace-nowrap"
                      style={{ backgroundColor: cat?.color || '#3B82F6' }}
                    >
                      {cat?.name}
                    </div>
                  </div>
                );
              })}
            </div>
          </div>
          <div className="w-full md:w-64 border-t md:border-t-0 md:border-l border-gray-100 p-4 overflow-auto">
            <h4 className="font-semibold mb-3">Annotations ({image.annotations.length})</h4>
            {image.annotations.length === 0 ? (
              <p className="text-gray-500 text-sm">No annotations</p>
            ) : (
              <ul className="space-y-2">
                {image.annotations.map((ann, idx) => {
                  const cat = dataset.categories.find((c) => c.id === ann.category_id);
                  return (
                    <li key={idx} className="flex items-center space-x-2 p-2 bg-gray-50 rounded">
                      <div className="w-3 h-3 rounded flex-shrink-0" style={{ backgroundColor: cat?.color }} />
                      <span className="text-sm">{cat?.name}</span>
                      <span className="text-xs text-gray-500">
                        [{ann.bbox.x.toFixed(0)}, {ann.bbox.y.toFixed(0)}, {ann.bbox.width.toFixed(0)}, {ann.bbox.height.toFixed(0)}]
                      </span>
                    </li>
                  );
                })}
              </ul>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

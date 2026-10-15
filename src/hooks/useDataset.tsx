import { createContext, useContext, useState, useEffect, useMemo, useCallback, useRef, ReactNode } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import { UnifiedDataset, UnifiedImage, ScanOptions } from '../types/dataset';

interface ScanProgress {
  current: number;
  total: number;
  stage: string;
}

interface DatasetStats {
  totalImages: number;
  totalAnnotations: number;
  totalClasses: number;
  avgAnnotationsPerImage: number;
}

interface DatasetContextType {
  dataset: UnifiedDataset | null;
  images: UnifiedImage[];
  stats: DatasetStats;
  loading: boolean;
  error: string | null;
  progress: ScanProgress | null;
  scanWorkspace: (path: string, options?: ScanOptions) => Promise<void>;
  clearDataset: () => void;
}

const DatasetContext = createContext<DatasetContextType | undefined>(undefined);

export function DatasetProvider({ children }: { children: ReactNode }) {
  const [dataset, setDataset] = useState<UnifiedDataset | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [progress, setProgress] = useState<ScanProgress | null>(null);
  const scanningRef = useRef(false);

  useEffect(() => {
    let unlisten: (() => void) | null = null;
    let disposed = false;
    listen<ScanProgress>('scan-progress', (event) => {
      setProgress(event.payload);
    }).then(fn => {
      if (disposed) {
        fn();
      } else {
        unlisten = fn;
      }
    });
    return () => {
      disposed = true;
      if (unlisten) unlisten();
    };
  }, []);

  const scanWorkspace = useCallback(async (path: string, options?: ScanOptions) => {
    if (scanningRef.current) return;
    scanningRef.current = true;
    setLoading(true);
    setError(null);
    setProgress(null);
    try {
      const result = await invoke<UnifiedDataset>('scan_workspace', { path, options });
      setDataset(result);
    } catch (e) {
      setError(typeof e === 'string' ? e : (e as any)?.message ?? String(e));
      setDataset(null);
    } finally {
      scanningRef.current = false;
      setLoading(false);
      setProgress(null);
    }
  }, []);

  const clearDataset = useCallback(() => {
    setDataset(null);
    setError(null);
    setProgress(null);
  }, []);

  const images = useMemo(() => dataset?.images ?? [], [dataset]);

  const stats: DatasetStats = useMemo(() => {
    const totalImages = dataset?.images?.length ?? 0;
    const totalAnnotations =
      dataset?.info?.total_annotations ??
      dataset?.images?.reduce((sum, img) => sum + img.annotations.length, 0) ??
      0;
    return {
      totalImages,
      totalAnnotations,
      totalClasses: dataset?.categories?.length ?? 0,
      avgAnnotationsPerImage: totalImages > 0 ? totalAnnotations / totalImages : 0,
    };
  }, [dataset]);

  return (
    <DatasetContext.Provider value={{
      dataset,
      images,
      stats,
      loading,
      error,
      progress,
      scanWorkspace,
      clearDataset,
    }}>
      {children}
    </DatasetContext.Provider>
  );
}

export function useDataset() {
  const context = useContext(DatasetContext);
  if (!context) {
    throw new Error('useDataset must be used within a DatasetProvider');
  }
  return context;
}

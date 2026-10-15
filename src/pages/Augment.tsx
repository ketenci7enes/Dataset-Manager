import { useState } from 'react';
import { useDataset } from '../hooks/useDataset';
import { invoke } from '@tauri-apps/api/tauri';
import { open } from '@tauri-apps/api/dialog';

const AUGMENT_OPS = [
  { opType: 'Flip', label: 'Flip', params: { horizontal: true, vertical: false }, icon: '↔️' },
  { opType: 'Rotate', label: 'Rotate', params: { degrees: 15 }, icon: '🔄' },
  { opType: 'Brightness', label: 'Brightness', params: { factor: 1.3 }, icon: '☀️' },
  { opType: 'Contrast', label: 'Contrast', params: { factor: 1.3 }, icon: '🌓' },
  { opType: 'Noise', label: 'Gaussian Noise', params: { std: 15 }, icon: '📺' },
  { opType: 'Blur', label: 'Gaussian Blur', params: { sigma: 1.5 }, icon: '🌫️' },
];

export default function Augment() {
  const { dataset } = useDataset();
  const [pipeline, setPipeline] = useState<{ opType: string; params: any; probability: number }[]>([]);
  const [outputDir, setOutputDir] = useState('');
  const [seed, setSeed] = useState(42);
  const [previewImages, setPreviewImages] = useState<string[]>([]);
  const [previewing, setPreviewing] = useState(false);
  const [running, setRunning] = useState(false);
  const [result, setResult] = useState<any>(null);

  const addOp = (op: any) => {
    setPipeline([...pipeline, { ...op, probability: 0.5 }]);
  };

  const removeOp = (index: number) => {
    setPipeline(pipeline.filter((_, i) => i !== index));
  };

  const updateOp = (index: number, updates: any) => {
    setPipeline(pipeline.map((op, i) => i === index ? { ...op, ...updates } : op));
  };

  const handlePreview = async () => {
    if (!dataset || pipeline.length === 0) return;
    setPreviewing(true);
    try {
      const sample = dataset.images[0]?.source_path;
      if (sample) {
        const images = await invoke<string[]>('preview_augmentation', {
          imagePath: sample,
          pipeline: { operations: pipeline, seed },
        });
        setPreviewImages(images);
      }
    } catch (err) {
      console.error(err);
    } finally {
      setPreviewing(false);
    }
  };

  const handleRun = async () => {
    if (!dataset || pipeline.length === 0 || !outputDir) return;
    setRunning(true);
    try {
      const res = await invoke<any>('run_augmentation', {
        dataset,
        pipeline: { operations: pipeline, seed },
        outputDir: outputDir,
      });
      setResult(res);
    } catch (err) {
      setResult({ success: false, message: String(err) });
    } finally {
      setRunning(false);
    }
  };

  if (!dataset) return <div className="text-center py-20 text-gray-500">No dataset loaded</div>;

  return (
    <div className="space-y-6 max-w-5xl">
      {/* Pipeline Builder */}
      <div className="bg-white rounded-xl border border-gray-100 shadow-sm p-6">
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-lg font-semibold text-gray-900">Augmentation Pipeline</h2>
          <div className="flex items-center space-x-4">
            <button onClick={handlePreview} disabled={previewing || pipeline.length === 0} className="px-4 py-2 text-sm border border-gray-300 rounded-lg hover:bg-gray-50 disabled:opacity-50">
              {previewing ? 'Previewing...' : 'Preview (First Image)'}
            </button>
          </div>
        </div>

        {/* Available Operations */}
        <div className="mb-6">
          <h3 className="font-medium text-gray-900 mb-3">Available Operations (click to add)</h3>
          <div className="flex flex-wrap gap-2">
            {AUGMENT_OPS.map(op => (
              <button
                key={op.opType}
                onClick={() => addOp({ opType: op.opType, params: op.params, probability: 0.5 })}
                className="px-3 py-2 bg-gray-50 border border-gray-200 rounded-lg hover:bg-gray-100 text-sm flex items-center space-x-1 transition-colors"
              >
                <span>{op.icon}</span>
                <span>{op.label}</span>
              </button>
            ))}
          </div>
        </div>

        {/* Pipeline */}
        <div className="mb-6">
          <h3 className="font-medium text-gray-900 mb-3">Pipeline ({pipeline.length} operations)</h3>
          {pipeline.length === 0 ? (
            <div className="text-center py-8 border-2 border-dashed border-gray-200 rounded-lg text-gray-500">
              Click operations above to build your pipeline
            </div>
          ) : (
            <div className="space-y-2">
              {pipeline.map((op, i) => (
                <div key={i} className="flex items-center space-x-3 p-3 bg-gray-50 rounded-lg border border-gray-200">
                  <span className="w-6 h-6 rounded bg-blue-100 text-blue-600 flex items-center justify-center text-sm font-medium">{i + 1}</span>
                  <span className="font-medium text-gray-900 capitalize">{op.opType.toLowerCase()}</span>
                  <div className="flex-1 flex items-center space-x-2">
                    {Object.entries(op.params).map(([key, value]) => (
                      <div key={key} className="flex items-center space-x-1">
                        <label className="text-xs text-gray-500">{key}:</label>
                        <input
                          type="number"
                          value={typeof value === 'number' || typeof value === 'string' ? value : String(value ?? '')}
                          onChange={(e) => updateOp(i, { params: { ...op.params, [key]: parseFloat(e.target.value) } })}
                          step="any"
                          className="w-16 px-2 py-1 text-sm border border-gray-300 rounded"
                        />
                      </div>
                    ))}
                    <div className="flex items-center space-x-1 ml-2">
                      <span className="text-xs text-gray-500">Prob:</span>
                      <input
                        type="range"
                        min={0}
                        max={1}
                        step={0.1}
                        value={op.probability}
                        onChange={(e) => updateOp(i, { probability: parseFloat(e.target.value) })}
                        className="w-32"
                      />
                      <span className="text-sm w-8">{Math.round(op.probability * 100)}%</span>
                    </div>
                    <button onClick={() => removeOp(i)} className="text-red-500 hover:text-red-700 p-1">
                      <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                      </svg>
                    </button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Settings */}
        <div className="grid md:grid-cols-3 gap-4 mb-6">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">Random Seed</label>
            <input type="number" value={seed} onChange={(e) => setSeed(parseInt(e.target.value))} className="w-full px-3 py-2 border border-gray-300 rounded-lg" />
          </div>
          <div className="md:col-span-2">
            <label className="block text-sm font-medium text-gray-700 mb-1">Output Directory</label>
            <div className="flex space-x-2">
              <input type="text" value={outputDir} onChange={(e) => setOutputDir(e.target.value)} readOnly className="flex-1 px-3 py-2 border border-gray-300 rounded-lg" />
              <button onClick={async () => { const dir = await open({ directory: true, title: 'Select Output Directory' }); if (dir && typeof dir === 'string') setOutputDir(dir); }} className="px-4 py-2 bg-gray-100 border border-gray-300 rounded-lg hover:bg-gray-200 text-sm">Browse</button>
            </div>
          </div>
        </div>

        <button
          onClick={handleRun}
          disabled={running || pipeline.length === 0 || !outputDir}
          className="w-full px-6 py-3 bg-green-600 text-white font-medium rounded-lg hover:bg-green-700 disabled:opacity-50"
        >
          {running ? 'Running Augmentation...' : 'Run Augmentation on Full Dataset'}
        </button>

        {result && (
          <div className="mt-4 p-4 rounded-lg text-sm border-l-4"
            style={{
              backgroundColor: result.success ? '#f0fdf4' : '#fef2f2',
              borderColor: result.success ? '#22c55e' : '#ef4444',
              color: result.success ? '#166534' : '#991b1b'
            }}
          >
            {result.message}
            {result.output_path && <div className="mt-1 font-mono text-xs">{result.output_path}</div>}
          </div>
        )}
      </div>

      {/* Preview Grid */}
      {previewImages.length > 0 && (
        <div className="bg-white rounded-xl border border-gray-100 shadow-sm p-6">
          <h3 className="font-semibold text-gray-900 mb-4">Preview Results</h3>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            {previewImages.map((img, i) => (
              <div key={i} className="aspect-square rounded-lg overflow-hidden border border-gray-200">
                <img src={`data:image/png;base64,${img}`} alt={`Preview ${i}`} className="w-full h-full object-cover" />
                <div className="absolute bottom-0 left-0 right-0 p-1 bg-black/70 text-white text-xs text-center">
                  {i === 0 ? 'Original' : `Variant ${i}`}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
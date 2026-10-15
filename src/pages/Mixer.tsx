import { useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { open } from '@tauri-apps/api/dialog';

export default function Mixer() {
  const [sources, setSources] = useState<{ path: string; weight: number; max_samples?: number }[]>([]);
  const [strategy, setStrategy] = useState<'Concatenate' | 'Stratified' | 'Balanced'>('Stratified');
  const [balanceClasses, setBalanceClasses] = useState(true);
  const [mixing, setMixing] = useState(false);
  const [result, setResult] = useState<any>(null);

  const addSource = async () => {
    const path = await open({ directory: true, title: 'Select Dataset Folder' });
    if (path && typeof path === 'string') {
      setSources([...sources, { path, weight: 1, max_samples: undefined }]);
    }
  };

  const handleMix = async () => {
    if (sources.length === 0) return;
    setMixing(true);
    try {
      const res = await invoke<any>('mix_datasets', {
        sources,
        config: { strategy, outputFormat: { type: 'YOLO' }, balanceClasses },
      });
      setResult(res);
    } catch (err) {
      setResult({ success: false, message: String(err) });
    } finally {
      setMixing(false);
    }
  };

  return (
    <div className="space-y-6 max-w-4xl">
      <div className="bg-white rounded-xl border border-gray-100 shadow-sm p-6">
        <h2 className="text-lg font-semibold text-gray-900 mb-4">Dataset Mixer</h2>
        <p className="text-gray-600 mb-6">Combine multiple datasets with class balancing</p>

        {/* Sources */}
        <div className="mb-6">
          <div className="flex items-center justify-between mb-4">
            <h3 className="font-medium text-gray-900">Source Datasets</h3>
            <button onClick={addSource} className="px-4 py-2 text-sm bg-blue-600 text-white rounded-lg hover:bg-blue-700">
              + Add Dataset Folder
            </button>
          </div>
          {sources.length === 0 ? (
            <div className="text-center py-8 border-2 border-dashed border-gray-200 rounded-lg text-gray-500">
              No datasets added yet
            </div>
          ) : (
            <div className="space-y-3">
              {sources.map((src, i) => (
                <div key={i} className="flex items-center space-x-3 p-3 bg-gray-50 rounded-lg">
                  <span className="w-8 h-8 rounded bg-blue-100 flex items-center justify-center text-blue-600 font-medium">{i + 1}</span>
                  <div className="flex-1 min-w-0">
                    <p className="text-sm font-medium text-gray-900 truncate">{src.path}</p>
                    <div className="flex items-center space-x-4 mt-1">
                      <div className="flex items-center space-x-1">
                        <span className="text-xs text-gray-500">Weight:</span>
                        <input
                          type="number"
                          value={src.weight}
                          onChange={(e) => setSources(s => s.map((s, idx) => idx === i ? { ...s, weight: parseFloat(e.target.value) } : s))}
                          step={0.1}
                          min={0.1}
                          className="w-16 px-2 py-1 text-sm border border-gray-300 rounded"
                        />
                      </div>
                      <div className="flex items-center space-x-1">
                        <span className="text-xs text-gray-500">Max samples:</span>
                        <input
                          type="number"
                          value={src.max_samples || ''}
                          onChange={(e) => setSources(s => s.map((s, idx) => idx === i ? { ...s, max_samples: e.target.value ? parseInt(e.target.value) : undefined } : s))}
                          placeholder="No limit"
                          className="w-24 px-2 py-1 text-sm border border-gray-300 rounded"
                        />
                      </div>
                      <button onClick={() => setSources(s => s.filter((_, idx) => idx !== i))} className="text-red-500 hover:text-red-700 text-sm">Remove</button>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Strategy */}
        <div className="mb-6">
          <label className="block text-sm font-medium text-gray-700 mb-2">Mixing Strategy</label>
          <div className="flex space-x-4">
            {(['Concatenate', 'Stratified', 'Balanced'] as const).map(s => (
              <label key={s} className="flex items-center space-x-2 cursor-pointer">
                <input
                  type="radio"
                  value={s}
                  checked={strategy === s}
                  onChange={(e) => setStrategy(e.target.value as any)}
                  className="text-blue-600 focus:ring-blue-500"
                />
                <span className="text-sm text-gray-700">{s}</span>
              </label>
            ))}
          </div>
        </div>

        <label className="flex items-center space-x-2 cursor-pointer mb-6">
          <input
            type="checkbox"
            checked={balanceClasses}
            onChange={(e) => setBalanceClasses(e.target.checked)}
            className="w-5 h-5 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
          />
          <span className="text-sm text-gray-700">Balance classes (stratified sampling)</span>
        </label>

        <button
          onClick={handleMix}
          disabled={mixing || sources.length === 0}
          className="w-full px-6 py-3 bg-blue-600 text-white font-medium rounded-lg hover:bg-blue-700 disabled:opacity-50"
        >
          {mixing ? 'Mixing...' : 'Mix Datasets'}
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
    </div>
  );
}
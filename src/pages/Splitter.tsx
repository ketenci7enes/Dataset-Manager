import { useState } from 'react';
import { useDataset } from '../hooks/useDataset';
import { invoke } from '@tauri-apps/api/tauri';

export default function Splitter() {
  const { dataset } = useDataset();
  const [ratios, setRatios] = useState({ train: 0.7, val: 0.2, test: 0.1 });
  const [strategy, setStrategy] = useState<'Random' | 'Stratified' | 'BySource'>('Stratified');
  const [outputFormats, setOutputFormats] = useState<string[]>(['YOLO', 'COCO']);
  const [seed, setSeed] = useState(42);
  const [splitting, setSplitting] = useState(false);
  const [result, setResult] = useState<any>(null);

  const handleRatioChange = (key: string, value: number) => {
    setRatios(prev => ({ ...prev, [key]: value }));
  };

  const totalRatio = ratios.train + ratios.val + ratios.test;

  const handleSplit = async () => {
    if (!dataset) return;
    setSplitting(true);
    try {
      const res = await invoke<any>('split_dataset', {
        dataset,
        config: {
          ratios: { train: ratios.train, val: ratios.val, test: ratios.test },
          strategy,
          outputFormats: outputFormats.map(f => ({ type: f })),
          seed,
        },
      });
      setResult(res);
    } catch (err) {
      setResult({ success: false, message: String(err) });
    } finally {
      setSplitting(false);
    }
  };

  if (!dataset) return <div className="text-center py-20 text-gray-500">No dataset loaded</div>;

  return (
    <div className="space-y-6 max-w-3xl">
      <div className="bg-white rounded-xl border border-gray-100 shadow-sm p-6">
        <h2 className="text-lg font-semibold text-gray-900 mb-4">Train / Val / Test Split</h2>
        <p className="text-gray-600 mb-6">Split your dataset with stratified sampling</p>

        {/* Ratios */}
        <div className="mb-6">
          <h3 className="font-medium text-gray-900 mb-4">Split Ratios</h3>
          <div className="space-y-4">
            {(['train', 'val', 'test'] as const).map(key => (
              <div key={key} className="flex items-center space-x-4">
                <label className="w-20 font-medium text-gray-700 capitalize">{key}</label>
                <input
                  type="range"
                  min={0}
                  max={1}
                  step={0.05}
                  value={ratios[key]}
                  onChange={(e) => handleRatioChange(key, parseFloat(e.target.value))}
                  className="flex-1"
                />
                <input
                  type="number"
                  min={0}
                  max={1}
                  step={0.05}
                  value={ratios[key]}
                  onChange={(e) => handleRatioChange(key, parseFloat(e.target.value))}
                  className="w-16 px-2 py-1 text-sm border border-gray-300 rounded text-right"
                />
                <span className="w-12 text-sm text-gray-500 text-right">
                  {Math.round(ratios[key] * 100)}%
                </span>
              </div>
            ))}
            <div className={`flex items-center space-x-4 p-2 rounded ${Math.abs(totalRatio - 1) < 0.01 ? 'bg-green-50' : 'bg-red-50'}`}>
              <span className="w-20 font-medium text-gray-700">Total</span>
              <span className={`font-mono ${Math.abs(totalRatio - 1) < 0.01 ? 'text-green-600' : 'text-red-600'}`}>
                {Math.round(totalRatio * 100)}%
              </span>
            </div>
          </div>
        </div>

        {/* Strategy */}
        <div className="mb-6">
          <label className="block text-sm font-medium text-gray-700 mb-2">Split Strategy</label>
          <div className="space-y-2">
            {(['Random', 'Stratified', 'BySource'] as const).map(s => (
              <label key={s} className="flex items-center space-x-3 cursor-pointer p-3 border rounded-lg hover:bg-gray-50 transition-colors">
                <input
                  type="radio"
                  value={s}
                  checked={strategy === s}
                  onChange={(e) => setStrategy(e.target.value as any)}
                  className="text-blue-600 focus:ring-blue-500"
                />
                <div>
                  <p className="font-medium text-gray-900">{s}</p>
                  <p className="text-sm text-gray-500">
                    {s === 'Random' && 'Random shuffle of all images'}
                    {s === 'Stratified' && 'Maintain class distribution in each split'}
                    {s === 'BySource' && 'Split by source dataset folder'}
                  </p>
                </div>
              </label>
            ))}
          </div>
        </div>

        {/* Output Formats */}
        <div className="mb-6">
          <label className="block text-sm font-medium text-gray-700 mb-2">Export Formats</label>
          <div className="flex flex-wrap gap-2">
            {['YOLO', 'COCO', 'VOC', 'LabelStudio'].map(f => (
              <label key={f} className="flex items-center space-x-2 cursor-pointer">
                <input
                  type="checkbox"
                  checked={outputFormats.includes(f)}
                  onChange={(e) => setOutputFormats(prev => e.target.checked ? [...prev, f] : prev.filter(x => x !== f))}
                  className="w-4 h-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
                />
                <span className="text-sm text-gray-700">{f}</span>
              </label>
            ))}
          </div>
        </div>

        {/* Seed */}
        <div className="mb-6">
          <label className="block text-sm font-medium text-gray-700 mb-1">Random Seed</label>
          <input type="number" value={seed} onChange={(e) => setSeed(parseInt(e.target.value))} className="w-full max-w-xs px-3 py-2 border border-gray-300 rounded-lg" />
        </div>

        <button
          onClick={handleSplit}
          disabled={splitting || Math.abs(totalRatio - 1) > 0.01}
          className="w-full px-6 py-3 bg-green-600 text-white font-medium rounded-lg hover:bg-green-700 disabled:opacity-50"
        >
          {splitting ? 'Splitting...' : 'Split Dataset'}
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
            {result.split_info && (
              <div className="mt-3 grid grid-cols-3 gap-4 text-center">
                <div className="bg-white/50 rounded p-2">
                  <div className="font-bold text-lg">{result.split_info.train_count}</div>
                  <div className="text-xs text-gray-500">Train</div>
                </div>
                <div className="bg-white/50 rounded p-2">
                  <div className="font-bold text-lg">{result.split_info.val_count}</div>
                  <div className="text-xs text-gray-500">Val</div>
                </div>
                <div className="bg-white/50 rounded p-2">
                  <div className="font-bold text-lg">{result.split_info.test_count}</div>
                  <div className="text-xs text-gray-500">Test</div>
                </div>
              </div>
            )}
          </div>
        )}
      </div>

      {/* Class Distribution Preview */}
      {dataset && (
        <div className="bg-white rounded-xl border border-gray-100 shadow-sm p-6">
          <h3 className="font-semibold text-gray-900 mb-4">Current Class Distribution</h3>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="text-left text-gray-500 border-b border-gray-200">
                  <th className="pb-2">Class</th>
                  <th className="pb-2 text-right">Count</th>
                  <th className="pb-2 text-right">Train (est.)</th>
                  <th className="pb-2 text-right">Val (est.)</th>
                  <th className="pb-2 text-right">Test (est.)</th>
                </tr>
              </thead>
              <tbody>
                {dataset.categories.map(cat => (
                  <tr key={cat.id} className="border-b border-gray-100">
                    <td className="py-2 flex items-center space-x-2">
                      <div className="w-3 h-3 rounded" style={{ backgroundColor: cat.color }} />
                      <span>{cat.name}</span>
                    </td>
                    <td className="py-2 text-right">{dataset.info.class_distribution[cat.name] || 0}</td>
                    <td className="py-2 text-right">~{Math.round((dataset.info.class_distribution[cat.name] || 0) * ratios.train)}</td>
                    <td className="py-2 text-right">~{Math.round((dataset.info.class_distribution[cat.name] || 0) * ratios.val)}</td>
                    <td className="py-2 text-right">~{Math.round((dataset.info.class_distribution[cat.name] || 0) * ratios.test)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}
    </div>
  );
}
import { useState } from 'react';
import { useDataset } from '../hooks/useDataset';
import { invoke } from '@tauri-apps/api/tauri';
import { open } from '@tauri-apps/api/dialog';
import { DetectedFormat } from '../types/dataset';

const FORMATS: { value: DetectedFormat['type']; label: string; description: string }[] = [
  { value: 'YOLO', label: 'YOLO', description: 'labels/*.txt + classes.txt + images/' },
  { value: 'COCO', label: 'COCO JSON', description: 'annotations/instances.json + images/' },
  { value: 'VOC', label: 'Pascal VOC', description: 'Annotations/*.xml + JPEGImages/' },
  { value: 'LabelStudio', label: 'Label Studio', description: 'LS JSON (Tasks format)' },
  { value: 'TFRecord', label: 'TFRecord', description: '*.tfrecord files (write only)' },
];

export default function Converter() {
  const { dataset } = useDataset();
  const [targetFormat, setTargetFormat] = useState<'YOLO' | 'COCO' | 'VOC' | 'LabelStudio' | 'TFRecord'>('COCO');
  const [outputDir, setOutputDir] = useState('');
  const [converting, setConverting] = useState(false);
  const [result, setResult] = useState<string | null>(null);

  const handleConvert = async () => {
    if (!dataset || !outputDir) return;
    setConverting(true);
    setResult(null);
    try {
      const target = { type: targetFormat } as any;
      const res = await invoke<any>('convert_format', {
        source: dataset,
        targetFormat: target,
        options: { outputDir },
      });
      setResult(`Success! Output: ${res.output_path} (${res.affected_files} files)`);
    } catch (err) {
      setResult(`Error: ${err}`);
    } finally {
      setConverting(false);
    }
  };

  if (!dataset) {
    return <div className="text-center py-20 text-gray-500">No dataset loaded</div>;
  }

  return (
    <div className="space-y-6 max-w-3xl">
      <div className="bg-white rounded-xl border border-gray-100 shadow-sm p-6">
        <h2 className="text-lg font-semibold text-gray-900 mb-4">Format Converter</h2>

        <div className="grid md:grid-cols-2 gap-6 mb-6">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">Source Format (Auto-detected)</label>
            <div className="px-3 py-2 bg-gray-50 border border-gray-200 rounded-lg text-sm text-gray-600">
              {dataset.images[0]?.source_format?.type || 'Unknown'}
            </div>
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">Target Format</label>
            <select
              value={targetFormat}
              onChange={(e) => setTargetFormat(e.target.value as any)}
              className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              {FORMATS.map(f => (
                <option key={f.value} value={f.value}>{f.label}</option>
              ))}
            </select>
            <p className="mt-1 text-sm text-gray-500">
              {FORMATS.find(f => f.value === targetFormat)?.description}
            </p>
          </div>
        </div>

        <div className="mb-6">
          <label className="block text-sm font-medium text-gray-700 mb-2">Output Directory</label>
          <div className="flex space-x-2">
            <input
              type="text"
              value={outputDir}
              onChange={(e) => setOutputDir(e.target.value)}
              placeholder="Select output directory..."
              className="flex-1 px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              readOnly
            />
            <button
              onClick={async () => {
                const dir = await open({ directory: true, title: 'Select Output Directory' });
                if (dir && typeof dir === 'string') setOutputDir(dir);
              }}
              className="px-4 py-2 bg-gray-100 border border-gray-300 rounded-lg hover:bg-gray-200 text-sm font-medium"
            >
              Browse
            </button>
          </div>
        </div>

        <button
          onClick={handleConvert}
          disabled={converting || !outputDir}
          className="w-full px-6 py-3 bg-blue-600 text-white font-medium rounded-lg hover:bg-blue-700 disabled:opacity-50 transition-colors"
        >
          {converting ? 'Converting...' : 'Convert Dataset'}
        </button>

        {result && (
          <div className="mt-4 p-4 rounded-lg text-sm border-l-4"
            style={{
              backgroundColor: result.startsWith('Error') ? '#fef2f2' : '#f0fdf4',
              borderColor: result.startsWith('Error') ? '#ef4444' : '#22c55e',
              color: result.startsWith('Error') ? '#991b1b' : '#166534'
            }}
          >
            {result}
          </div>
        )}
      </div>

      {/* Format Info Cards */}
      <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-4">
        {FORMATS.map(fmt => (
          <div key={fmt.value} className="bg-white rounded-xl border border-gray-100 p-5 hover:shadow-md transition-shadow">
            <div className="flex items-start space-x-3">
              <div className="w-10 h-10 rounded-lg bg-blue-100 flex items-center justify-center">
                <span className="text-xl">{fmt.value === 'YOLO' ? '📁' : fmt.value === 'COCO' ? '📄' : fmt.value === 'VOC' ? '📋' : fmt.value === 'LabelStudio' ? '🏷️' : '📦'}</span>
              </div>
              <div>
                <h4 className="font-semibold text-gray-900">{fmt.label}</h4>
                <p className="text-sm text-gray-500 mt-1">{fmt.description}</p>
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
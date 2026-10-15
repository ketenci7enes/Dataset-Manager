import React, { useState } from 'react';
import { useDataset } from '../hooks/useDataset';
import { invoke } from '@tauri-apps/api/tauri';

const PRESETS = [
  { name: 'Class + Index', template: '{class}_{index:06d}{ext}' },
  { name: 'Class + Hash', template: '{class}_{hash}{ext}' },
  { name: 'Date + Index', template: '{date}_{index:06d}{ext}' },
  { name: 'Original + Index', template: '{original}_{index:06d}{ext}' },
];

export default function Organizer() {
  const { dataset } = useDataset();
  const [step, setStep] = useState(1);
  const [namingTemplate, setNamingTemplate] = useState('{class}_{index:06d}{ext}');
  const [createClassFolders, setCreateClassFolders] = useState(true);
  const [conflictStrategy, setConflictStrategy] = useState<'Skip' | 'Overwrite' | 'AutoRename'>('AutoRename');
  const [dryRun, setDryRun] = useState(true);
  const [organizing, setOrganizing] = useState(false);
  const [result, setResult] = useState<any>(null);
  const [preview, setPreview] = useState<any[]>([]);

  const generatePreview = () => {
    if (!dataset) return;
    const samples = dataset.images.slice(0, 10);
    const classCounters: Record<number, number> = {};
    const previews = samples.map(img => {
      const primaryClass = img.annotations[0]?.category_id || 0;
      const cat = dataset.categories.find(c => c.id === primaryClass);
      const className = cat?.name || 'unknown';
      const counter = (classCounters[primaryClass] = (classCounters[primaryClass] || 0) + 1);
      const ext = img.file_name.split('.').pop() || 'jpg';
      const newName = namingTemplate
        .replace('{class}', className)
        .replace('{index:06d}', counter.toString().padStart(6, '0'))
        .replace('{hash}', 'abc123ef')
        .replace('{date}', new Date().toISOString().split('T')[0].replace(/-/g, ''))
        .replace('{original}', img.file_name.split('.').slice(0, -1).join('.'))
        .replace('{ext}', '.' + ext);
      return { old: img.file_name, new: newName };
    });
    setPreview(previews);
  };

  const handleOrganize = async () => {
    if (!dataset) return;
    setOrganizing(true);
    try {
      const res = await invoke<any>('organize_files', {
        dataset,
        config: { namingTemplate, createClassFolders, conflictStrategy, dryRun },
      });
      setResult(res);
    } catch (err) {
      setResult({ success: false, message: String(err) });
    } finally {
      setOrganizing(false);
    }
  };

  if (!dataset) return <div className="text-center py-20 text-gray-500">No dataset loaded</div>;

  return (
    <div className="space-y-6 max-w-4xl">
      {/* Progress Steps */}
      <div className="flex items-center justify-between">
        {[1, 2, 3].map(s => (
          <React.Fragment key={s}>
            <div className={`flex items-center ${s < 3 ? 'flex-1' : ''}`}>
              <div className={`w-8 h-8 rounded-full flex items-center justify-center text-sm font-medium ${
                step >= s ? 'bg-blue-600 text-white' : 'bg-gray-200 text-gray-500'
              }`}>{s}</div>
              {s < 3 && <div className={`flex-1 h-1 mx-2 ${step > s ? 'bg-blue-600' : 'bg-gray-200'}`} />}
            </div>
          </React.Fragment>
        ))}
      </div>
      <div className="flex justify-between text-sm text-gray-500">
        <span className="w-1/3 text-center">Template</span>
        <span className="w-1/3 text-center">Structure</span>
        <span className="w-1/3 text-center">Options</span>
      </div>

      {/* Step 1: Naming Template */}
      {step === 1 && (
        <div className="bg-white rounded-xl border border-gray-100 shadow-sm p-6">
          <h2 className="text-lg font-semibold text-gray-900 mb-4">Step 1: Naming Template</h2>
          <div className="mb-6">
            <label className="block text-sm font-medium text-gray-700 mb-2">Presets</label>
            <div className="flex flex-wrap gap-2">
              {PRESETS.map(p => (
                <button
                  key={p.name}
                  onClick={() => { setNamingTemplate(p.template); generatePreview(); }}
                  className={`px-4 py-2 rounded-lg text-sm border transition-colors ${
                    namingTemplate === p.template
                      ? 'bg-blue-600 border-blue-600 text-white'
                      : 'bg-white border-gray-300 text-gray-700 hover:bg-gray-50'
                  }`}
                >
                  {p.name}
                </button>
              ))}
            </div>
          </div>
          <div className="mb-6">
            <label className="block text-sm font-medium text-gray-700 mb-2">Custom Template</label>
            <input
              type="text"
              value={namingTemplate}
              onChange={(e) => { setNamingTemplate(e.target.value); generatePreview(); }}
              className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 font-mono text-sm"
              placeholder="{class}_{index:06d}{ext}"
            />
            <p className="mt-1 text-xs text-gray-500">
              Variables: &#123;class&#125;, &#123;index:06d&#125;, &#123;hash&#125;, &#123;date&#125;, &#123;original&#125;, &#123;ext&#125;
            </p>
          </div>
          <div className="flex justify-end">
            <button onClick={() => setStep(2)} className="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700">Next</button>
          </div>
        </div>
      )}

      {/* Step 2: Folder Structure */}
      {step === 2 && (
        <div className="bg-white rounded-xl border border-gray-100 shadow-sm p-6">
          <h2 className="text-lg font-semibold text-gray-900 mb-4">Step 2: Folder Structure</h2>
          <div className="space-y-4">
            <label className="flex items-center space-x-3 cursor-pointer">
              <input
                type="checkbox"
                checked={createClassFolders}
                onChange={(e) => setCreateClassFolders(e.target.checked)}
                className="w-5 h-5 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
              />
              <div>
                <p className="font-medium text-gray-900">Create class subfolders</p>
                <p className="text-sm text-gray-500">Organize images/labels into class-name folders (images/class/, labels/class/)</p>
              </div>
            </label>
            <div className="bg-gray-50 rounded-lg p-4 font-mono text-sm">
              {createClassFolders ? (
                <>
                  <div>images/</div>
                  <div className="ml-4">├── class1/</div>
                  <div className="ml-4">│   ├── image1.jpg</div>
                  <div className="ml-4">│   └── image2.jpg</div>
                  <div className="ml-4">└── class2/</div>
                  <div className="mt-2">labels/</div>
                  <div className="ml-4">├── class1/</div>
                  <div className="ml-4">│   ├── image1.txt</div>
                  <div className="ml-4">└── class2/</div>
                </>
              ) : (
                <>
                  <div>images/</div>
                  <div className="ml-4">├── class1_image1.jpg</div>
                  <div className="ml-4">└── class2_image2.jpg</div>
                  <div className="mt-2">labels/</div>
                  <div className="ml-4">├── class1_image1.txt</div>
                  <div className="ml-4">└── class2_image2.txt</div>
                </>
              )}
            </div>
          </div>
          <div className="flex justify-between mt-6">
            <button onClick={() => setStep(1)} className="px-6 py-2 border border-gray-300 rounded-lg hover:bg-gray-50">Back</button>
            <button onClick={() => setStep(3)} className="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700">Next</button>
          </div>
        </div>
      )}

      {/* Step 3: Options & Preview */}
      {step === 3 && (
        <div className="bg-white rounded-xl border border-gray-100 shadow-sm p-6">
          <h2 className="text-lg font-semibold text-gray-900 mb-4">Step 3: Options & Preview</h2>
          <div className="grid md:grid-cols-2 gap-6 mb-6">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">Conflict Strategy</label>
              <select
                value={conflictStrategy}
                onChange={(e) => setConflictStrategy(e.target.value as any)}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              >
                <option value="Skip">Skip existing files</option>
                <option value="Overwrite">Overwrite existing files</option>
                <option value="AutoRename">Auto-rename (add _1, _2...)</option>
              </select>
            </div>
            <div>
              <label className="flex items-center space-x-3 cursor-pointer">
                <input
                  type="checkbox"
                  checked={dryRun}
                  onChange={(e) => setDryRun(e.target.checked)}
                  className="w-5 h-5 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
                />
                <span className="font-medium text-gray-900">Dry run (preview only, no files copied)</span>
              </label>
            </div>
          </div>

          <div className="mb-6">
            <h3 className="font-medium text-gray-900 mb-2">Preview (first 10 files)</h3>
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <thead>
                  <tr className="text-left text-gray-500 border-b border-gray-200">
                    <th className="pb-2">Original</th>
                    <th className="pb-2">New Name</th>
                  </tr>
                </thead>
                <tbody>
                  {preview.map((p, i) => (
                    <tr key={i} className="border-b border-gray-100 hover:bg-gray-50">
                      <td className="py-2 font-mono text-gray-600">{p.old}</td>
                      <td className="py-2 font-mono text-blue-600">{p.new}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>

          <div className="flex justify-between">
            <button onClick={() => setStep(2)} className="px-6 py-2 border border-gray-300 rounded-lg hover:bg-gray-50">Back</button>
            <button
              onClick={handleOrganize}
              disabled={organizing}
              className="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50"
            >
              {organizing ? 'Organizing...' : dryRun ? 'Preview Changes' : 'Apply Changes'}
            </button>
          </div>

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
      )}
    </div>
  );
}
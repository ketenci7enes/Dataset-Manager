import { useEffect, useState } from 'react';
import { useWorkspace } from '../hooks/useWorkspace';
import { useDataset } from '../hooks/useDataset';
import { invoke } from '@tauri-apps/api/tauri';
import { useNavigate } from 'react-router-dom';

export default function Dashboard() {
  const { currentWorkspace } = useWorkspace();
  const { dataset, loading, error, scanWorkspace, stats } = useDataset();
  const [selectedClass, setSelectedClass] = useState<string | null>(null);
  const [exporting, setExporting] = useState(false);

  useEffect(() => {
    if (currentWorkspace && !dataset) {
      scanWorkspace(currentWorkspace.path);
    }
  }, [currentWorkspace, dataset, scanWorkspace]);

  const handleExportLabelStudio = async () => {
    if (!dataset) return;
    setExporting(true);
    try {
      const outputPath = await invoke<string>('export_label_studio', {
        dataset,
        config: {
          fromName: 'label',
          toName: 'image',
          imageSource: 'local-files',
          outputFormat: 'Tasks',
        },
      });
      alert(`Label Studio export saved to: ${outputPath}`);
    } catch (err) {
      alert(`Export failed: ${err}`);
    } finally {
      setExporting(false);
    }
  };

  const formatCount = (count: number) => {
    if (count >= 1000000) return `${(count / 1000000).toFixed(1)}M`;
    if (count >= 1000) return `${(count / 1000).toFixed(1)}K`;
    return count.toString();
  };

  if (!currentWorkspace) {
    return (
      <div className="text-center py-20">
        <svg className="mx-auto h-16 w-16 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
        </svg>
        <h2 className="mt-4 text-xl font-semibold text-gray-700">No workspace selected</h2>
        <p className="mt-2 text-gray-500">Select a workspace from the header to get started</p>
      </div>
    );
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-12 w-12 border-4 border-blue-500 border-t-transparent"></div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="text-center py-20 text-red-600">
        <h2 className="text-xl font-semibold">Error loading dataset</h2>
        <p className="mt-2">{error}</p>
      </div>
    );
  }

  if (!dataset) {
    return (
      <div className="text-center py-20">
        <p className="text-gray-500">Dataset not loaded</p>
      </div>
    );
  }

  const classColors: Record<string, string> = {};
  dataset.categories.forEach((cat) => {
    classColors[cat.name] = cat.color;
  });

  return (
    <div className="space-y-6">
      {/* Header Stats */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <StatCard title="Images" value={formatCount(stats?.totalImages || 0)} icon="🖼️" color="blue" />
        <StatCard title="Annotations" value={formatCount(stats?.totalAnnotations || 0)} icon="🏷️" color="green" />
        <StatCard title="Classes" value={stats?.totalClasses || 0} icon="📊" color="purple" />
        <StatCard title="Avg/Img" value={stats?.avgAnnotationsPerImage?.toFixed(1) || '0'} icon="📈" color="orange" />
      </div>

      {/* Class Summary Grid */}
      <div className="bg-white rounded-xl border border-gray-100 shadow-sm">
        <div className="px-6 py-4 border-b border-gray-100 flex items-center justify-between">
          <h2 className="text-lg font-semibold text-gray-900">Class Distribution</h2>
          <div className="flex items-center space-x-2">
            <button
              onClick={handleExportLabelStudio}
              disabled={exporting}
              className="px-4 py-2 text-sm font-medium text-white bg-blue-600 rounded-lg hover:bg-blue-700 disabled:opacity-50 transition-colors flex items-center space-x-2"
            >
              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
              </svg>
              <span>Export Label Studio</span>
            </button>
          </div>
        </div>
        <div className="p-6">
          {dataset.categories.length === 0 ? (
            <p className="text-center text-gray-500 py-8">No classes found in dataset</p>
          ) : (
            <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-6 gap-4">
              {dataset.categories.map((cat) => {
                const count = dataset.info.class_distribution[cat.name] || 0;
                return (
                  <ClassCard
                    key={cat.id}
                    className={cat.name}
                    count={count}
                    color={cat.color}
                    selected={selectedClass === cat.name}
                    onClick={() => setSelectedClass(selectedClass === cat.name ? null : cat.name)}
                  />
                );
              })}
            </div>
          )}
        </div>
      </div>

      {/* Quick Actions */}
      <div className="bg-white rounded-xl border border-gray-100 shadow-sm">
        <div className="px-6 py-4 border-b border-gray-100">
          <h2 className="text-lg font-semibold text-gray-900">Quick Actions</h2>
        </div>
        <div className="p-6 grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-4">
          <ActionCard title="Explore" desc="Browse images" icon="🔍" path="/explorer" />
          <ActionCard title="Convert" desc="Change format" icon="🔄" path="/converter" />
          <ActionCard title="Organize" desc="Rename & structure" icon="📁" path="/organizer" />
          <ActionCard title="Mix" desc="Merge datasets" icon="🔀" path="/mixer" />
          <ActionCard title="Augment" desc="Data augmentation" icon="✨" path="/augment" />
          <ActionCard title="Split" desc="Train/Val/Test" icon="✂️" path="/splitter" />
        </div>
      </div>

      {/* Dataset Info */}
      <div className="bg-white rounded-xl border border-gray-100 shadow-sm">
        <div className="px-6 py-4 border-b border-gray-100">
          <h2 className="text-lg font-semibold text-gray-900">Dataset Information</h2>
        </div>
        <div className="p-6 grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
          <InfoRow label="Name" value={dataset.info.name} />
          <InfoRow label="Path" value={dataset.info.path} />
          <InfoRow label="Format" value={dataset.images[0]?.source_format?.type || 'Unknown'} />
          <InfoRow label="Classes" value={dataset.categories.length.toString()} />
        </div>
      </div>
    </div>
  );
}

function StatCard({ title, value, icon, color }: { title: string; value: string | number; icon: string; color: string }) {
  const colors = {
    blue: 'bg-blue-50 text-blue-600 border-blue-200',
    green: 'bg-green-50 text-green-600 border-green-200',
    purple: 'bg-purple-50 text-purple-600 border-purple-200',
    orange: 'bg-orange-50 text-orange-600 border-orange-200',
  };
  return (
    <div className={`bg-white rounded-xl border p-6 ${colors[color as keyof typeof colors]}`}>
      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm font-medium text-gray-600">{title}</p>
          <p className="mt-1 text-3xl font-bold">{value}</p>
        </div>
        <span className="text-3xl">{icon}</span>
      </div>
    </div>
  );
}

function ClassCard({ className, count, color, selected, onClick }: { className: string; count: number; color: string; selected: boolean; onClick: () => void }) {
  return (
    <button
      onClick={onClick}
      className={`relative p-4 rounded-xl border-2 transition-all text-left ${
        selected
          ? `border-2 border-current bg-opacity-10`
          : 'border-gray-100 hover:border-gray-300'
      }`}
      style={{ borderColor: selected ? color : undefined, backgroundColor: selected ? color + '15' : undefined }}
    >
      <div className="flex items-center space-x-3">
        <div className="w-3 h-3 rounded-full" style={{ backgroundColor: color }} />
        <div className="flex-1 min-w-0">
          <p className="font-medium text-gray-900 truncate">{className}</p>
          <p className="text-sm text-gray-500">{count.toLocaleString()} annotations</p>
        </div>
      </div>
      {selected && (
        <div className="absolute inset-0 rounded-xl border-2" style={{ borderColor: color }} />
      )}
    </button>
  );
}

function ActionCard({ title, desc, icon, path }: { title: string; desc: string; icon: string; path: string }) {
  const navigate = useNavigate();
  return (
    <button
      onClick={() => navigate(path)}
      className="p-4 rounded-xl border border-gray-100 hover:border-blue-300 hover:bg-blue-50 transition-all text-center w-full"
    >
      <div className="text-2xl mb-2">{icon}</div>
      <p className="font-medium text-gray-900">{title}</p>
      <p className="text-sm text-gray-500">{desc}</p>
    </button>
  );
}

function InfoRow({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <p className="text-sm text-gray-500">{label}</p>
      <p className="text-sm font-medium text-gray-900 truncate">{value}</p>
    </div>
  );
}
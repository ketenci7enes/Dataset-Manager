import { useState, useCallback } from 'react';
import { useWorkspace, buildWorkspaceInfo } from '../hooks/useWorkspace';
import { invoke } from '@tauri-apps/api/tauri';
import { open } from '@tauri-apps/api/dialog';
import { useNavigate } from 'react-router-dom';

export default function Home() {
  const { addWorkspace, workspaces } = useWorkspace();
  const navigate = useNavigate();
  const [dragging, setDragging] = useState(false);
  const [hovered, setHovered] = useState(false);
  const [selectedPath, setSelectedPath] = useState<string | null>(null);

  const handleSelectFolder = useCallback(async () => {
    try {
      const path = await open({
        directory: true,
        title: 'Select Dataset Folder (containing images/ and labels/ folders)'
      });
      if (path) {
        const ws = await invoke<import('../types/dataset').WorkspaceInfo>('detect_workspace_format', { path });
        const fullWs = buildWorkspaceInfo(path as string, ws as any);
        addWorkspace(fullWs);
        setSelectedPath(path as string);
        navigate('/dashboard');
      }
    } catch (err) {
      console.error('Failed to pick workspace:', err);
    }
  }, [navigate]);

  const handleDrop = useCallback(async (e: React.DragEvent) => {
    e.preventDefault();
    setDragging(false);
    setHovered(false);

    const files = e.dataTransfer.files;
    if (files.length > 0) {
      const file = files[0];
      let path: string | null = null;
      if (file && (file as any).webkitGetAsEntry) {
        const entry = (file as any).webkitGetAsEntry();
        if (entry && entry.isDirectory) {
          const full = entry.fullPath as string;
          // fullPath is only usable if it's already an absolute OS path.
          if (/^([a-zA-Z]:[\\/]|\/)/.test(full)) {
            path = full;
          }
        }
      }
      if (path) {
        try {
          const ws = await invoke<import('../types/dataset').WorkspaceInfo>('detect_workspace_format', { path });
          const fullWs = buildWorkspaceInfo(path, ws as any);
          addWorkspace(fullWs);
          setSelectedPath(path);
          navigate('/dashboard');
          return;
        } catch (err) {
          console.error('Failed to detect format:', err);
        }
      }
      // Browsers don't expose absolute paths on drop; fall back to the picker.
      await handleSelectFolder();
    }
  }, [handleSelectFolder, addWorkspace, navigate]);

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setDragging(true);
    setHovered(true);
  }, []);

  const handleDragLeave = useCallback(() => {
    setDragging(false);
    setHovered(false);
  }, []);

  return (
    <div className="min-h-screen bg-gray-50 flex items-center justify-center p-4">
      <div className="max-w-3xl w-full">
        {/* Hero Section */}
        <div className="text-center mb-12">
          <svg className="mx-auto h-20 w-20 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
          </svg>
          <h1 className="mt-6 text-4xl font-bold text-gray-900">Dataset Manager</h1>
          <p className="mt-3 text-lg text-gray-600">
            Manage, convert, and augment your computer vision datasets with ease.
            Supports YOLO, COCO, Pascal VOC, Label Studio, and TFRecord formats.
          </p>
        </div>

        {/* Drop Zone */}
        <div
          onDragOver={handleDragOver}
          onDragLeave={handleDragLeave}
          onDrop={handleDrop}
          onClick={handleSelectFolder}
          className={`relative border-2 border-dashed rounded-2xl p-12 transition-all duration-200 cursor-pointer ${
            dragging
              ? 'border-blue-500 bg-blue-50'
              : hovered
              ? 'border-blue-400 bg-blue-25'
              : 'border-gray-300 hover:border-blue-400 hover:bg-gray-50'
          }`}
        >
          <div className="text-center">
            <svg className="mx-auto h-16 w-16 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
            </svg>
            <h2 className="mt-4 text-xl font-semibold text-gray-700">
              {dragging ? 'Drop your dataset folder here' : 'Select or drag a dataset folder'}
            </h2>
            <p className="mt-2 text-gray-500">
              Folder should contain images/ and labels/ folders (YOLO format)
            </p>
            <p className="mt-4 text-sm text-gray-400">
              Click to browse or drag & drop a folder
            </p>
            {selectedPath && (
              <div className="mt-4 p-3 bg-blue-50 border border-blue-200 rounded-lg text-left">
                <p className="text-sm font-medium text-blue-900">Selected folder:</p>
                <p className="text-sm text-blue-700 truncate max-w-md">{selectedPath}</p>
                <p className="mt-2 text-xs text-blue-600">Click or drag another folder to change</p>
              </div>
            )}
          </div>
        </div>

        {/* Supported Formats */}
        <div className="mt-10">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">Supported Formats</h3>
          <div className="grid grid-cols-2 md:grid-cols-5 gap-4">
            {[
              { name: 'YOLO', icon: '📁', desc: 'labels/ + classes.txt' },
              { name: 'COCO', icon: '📄', desc: 'instances.json' },
              { name: 'VOC', icon: '📋', desc: 'Annotations/*.xml' },
              { name: 'Label Studio', icon: '🏷️', desc: 'LS JSON export' },
              { name: 'TFRecord', icon: '📦', desc: '*.tfrecord' },
            ].map((fmt) => (
              <div key={fmt.name} className="bg-white rounded-xl p-4 border border-gray-100 hover:border-blue-200 hover:shadow-md transition-all">
                <div className="text-3xl mb-2">{fmt.icon}</div>
                <div className="font-semibold text-gray-900">{fmt.name}</div>
                <div className="text-sm text-gray-500 mt-1">{fmt.desc}</div>
              </div>
            ))}
          </div>
        </div>

        {/* Recent Workspaces */}
        {workspaces.length > 0 && (
          <div className="mt-10">
            <h3 className="text-lg font-semibold text-gray-900 mb-4">Recent Workspaces</h3>
            <div className="bg-white rounded-xl border border-gray-100 overflow-hidden">
              {workspaces.slice(0, 5).map((ws) => (
                <button
                  key={ws.id}
                  onClick={() => { addWorkspace(ws); navigate('/dashboard'); }}
                  className="w-full px-4 py-3 text-left hover:bg-gray-50 border-b border-gray-100 last:border-0 transition-colors flex items-center justify-between"
                >
                  <div className="flex items-center space-x-3">
                    <div className="w-10 h-10 rounded-lg bg-blue-100 flex items-center justify-center">
                      <svg className="w-5 h-5 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
                      </svg>
                    </div>
                    <div>
                      <p className="font-medium text-gray-900">{ws.name}</p>
                      <p className="text-sm text-gray-500 truncate max-w-xs">{ws.path}</p>
                    </div>
                  </div>
                  <span className="px-2 py-1 text-xs font-medium rounded-full bg-gray-100 text-gray-600 capitalize">
                    {ws.detected_format?.type || 'unknown'}
                  </span>
                </button>
              ))}
            </div>
          </div>
        )}

        {/* Features */}
        <div className="mt-10 grid md:grid-cols-3 gap-6">
          {[
            { title: 'Format Conversion', desc: 'Convert between YOLO, COCO, VOC, Label Studio, and TFRecord', path: '/converter' },
            { title: 'Label Studio Export', desc: 'One-click export for Label Studio annotation workflow', path: '/dashboard' },
            { title: 'Data Augmentation', desc: 'Pipeline-based augmentation with preview and batch processing', path: '/augment' },
            { title: 'Dataset Mixing', desc: 'Combine multiple datasets with class balancing', path: '/mixer' },
            { title: 'Train/Val/Test Split', desc: 'Stratified splitting with multi-format export', path: '/splitter' },
            { title: 'File Organization', desc: 'Template-based renaming and class folder structure', path: '/organizer' },
          ].map((feature) => (
            <button
              key={feature.title}
              onClick={() => navigate(feature.path)}
              className="bg-white rounded-xl p-6 border border-gray-100 hover:shadow-lg hover:border-blue-300 transition-all text-left w-full"
            >
              <h4 className="font-semibold text-gray-900">{feature.title}</h4>
              <p className="mt-2 text-gray-600 text-sm">{feature.desc}</p>
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}
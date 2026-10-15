import { useState, useEffect } from 'react';
import { open } from '@tauri-apps/api/dialog';

export default function Settings() {
  const [theme, setTheme] = useState<'light' | 'dark' | 'system'>('system');
  const [language, setLanguage] = useState('tr');
  const [defaultExportDir, setDefaultExportDir] = useState('');
  const [lsFromName, setLsFromName] = useState('label');
  const [lsToName, setLsToName] = useState('image');
  const [organizerTemplate, setOrganizerTemplate] = useState('{class}_{index:06d}{ext}');
  const [organizerClassFolders, setOrganizerClassFolders] = useState(true);
  const [augmentSeed, setAugmentSeed] = useState(42);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    loadSettings();
  }, []);

  const loadSettings = async () => {
    try {
      const stored = localStorage.getItem('dataset-manager-settings');
      if (stored) {
        const settings = JSON.parse(stored);
        setTheme(settings.theme || 'system');
        setLanguage(settings.language || 'tr');
        setDefaultExportDir(settings.defaultExportDir || '');
        setLsFromName(settings.lsFromName || 'label');
        setLsToName(settings.lsToName || 'image');
        setOrganizerTemplate(settings.organizerTemplate || '{class}_{index:06d}{ext}');
        setOrganizerClassFolders(settings.organizerClassFolders ?? true);
        setAugmentSeed(settings.augmentSeed || 42);
      }
    } catch (e) {
      console.error('Failed to load settings:', e);
    }
  };

  const saveSettings = () => {
    const settings = {
      theme,
      language,
      defaultExportDir,
      lsFromName,
      lsToName,
      organizerTemplate,
      organizerClassFolders,
      augmentSeed,
    };
    localStorage.setItem('dataset-manager-settings', JSON.stringify(settings));
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  };

  return (
    <div className="space-y-6 max-w-2xl">
      <div className="bg-white rounded-xl border border-gray-100 shadow-sm p-6">
        <h2 className="text-lg font-semibold text-gray-900 mb-6">Settings</h2>

        {saved && (
          <div className="mb-6 p-3 bg-green-50 border border-green-200 rounded-lg text-green-800 text-sm">
            Settings saved successfully!
          </div>
        )}

        {/* Appearance */}
        <div className="mb-8">
          <h3 className="font-medium text-gray-900 mb-4">Appearance</h3>
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">Theme</label>
              <select
                value={theme}
                onChange={(e) => setTheme(e.target.value as any)}
                className="w-full max-w-xs px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              >
                <option value="system">System</option>
                <option value="light">Light</option>
                <option value="dark">Dark</option>
              </select>
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">Language</label>
              <select
                value={language}
                onChange={(e) => setLanguage(e.target.value)}
                className="w-full max-w-xs px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              >
                <option value="tr">Türkçe</option>
                <option value="en">English</option>
              </select>
            </div>
          </div>
        </div>

        {/* Default Paths */}
        <div className="mb-8">
          <h3 className="font-medium text-gray-900 mb-4">Default Paths</h3>
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">Default Export Directory</label>
            <div className="flex space-x-2">
              <input
                type="text"
                value={defaultExportDir}
                onChange={(e) => setDefaultExportDir(e.target.value)}
                placeholder="Leave empty for workspace/exports"
                className="flex-1 px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
              <button
                onClick={async () => {
                  const dir = await open({ directory: true, title: 'Select Default Export Directory' });
                  if (dir && typeof dir === 'string') setDefaultExportDir(dir);
                }}
                className="px-4 py-2 bg-gray-100 border border-gray-300 rounded-lg hover:bg-gray-200 text-sm"
              >
                Browse
              </button>
            </div>
          </div>
        </div>

        {/* Label Studio Defaults */}
        <div className="mb-8">
          <h3 className="font-medium text-gray-900 mb-4">Label Studio Defaults</h3>
          <div className="grid md:grid-cols-2 gap-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">From Name</label>
              <input
                type="text"
                value={lsFromName}
                onChange={(e) => setLsFromName(e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">To Name</label>
              <input
                type="text"
                value={lsToName}
                onChange={(e) => setLsToName(e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
          </div>
        </div>

        {/* Organizer Defaults */}
        <div className="mb-8">
          <h3 className="font-medium text-gray-900 mb-4">Organizer Defaults</h3>
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">Naming Template</label>
              <input
                type="text"
                value={organizerTemplate}
                onChange={(e) => setOrganizerTemplate(e.target.value)}
                className="w-full max-w-md px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 font-mono text-sm"
              />
              <p className="mt-1 text-xs text-gray-500">Variables: &#123;class&#125;, &#123;index:06d&#125;, &#123;hash&#125;, &#123;date&#125;, &#123;original&#125;, &#123;ext&#125;</p>
            </div>
            <label className="flex items-center space-x-3 cursor-pointer">
              <input
                type="checkbox"
                checked={organizerClassFolders}
                onChange={(e) => setOrganizerClassFolders(e.target.checked)}
                className="w-5 h-5 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
              />
              <span className="text-sm text-gray-700">Create class subfolders by default</span>
            </label>
          </div>
        </div>

        {/* Augmentation Defaults */}
        <div className="mb-8">
          <h3 className="font-medium text-gray-900 mb-4">Augmentation Defaults</h3>
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">Default Random Seed</label>
            <input
              type="number"
              value={augmentSeed}
              onChange={(e) => setAugmentSeed(parseInt(e.target.value) || 42)}
              className="w-full max-w-xs px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>
        </div>

        {/* Data Management */}
        <div className="mb-8 border-t border-gray-200 pt-6">
          <h3 className="font-medium text-gray-900 mb-4">Data Management</h3>
          <div className="space-y-3">
            <button
              onClick={() => {
                localStorage.removeItem('dataset-manager-workspaces');
                localStorage.removeItem('dataset-manager-settings');
                loadSettings();
                window.location.reload();
              }}
              className="px-4 py-2 bg-red-50 border border-red-200 text-red-700 rounded-lg hover:bg-red-100 text-sm font-medium"
            >
              Clear All Local Data (Workspaces & Settings)
            </button>
          </div>
        </div>

        <button
          onClick={saveSettings}
          className="w-full px-6 py-3 bg-blue-600 text-white font-medium rounded-lg hover:bg-blue-700 transition-colors"
        >
          Save Settings
        </button>
      </div>

      {/* About */}
      <div className="bg-white rounded-xl border border-gray-100 shadow-sm p-6 text-center text-gray-500">
        <h3 className="font-semibold text-gray-900 mb-2">Dataset Manager v0.1.0</h3>
        <p className="text-sm">Built with Tauri + React + TypeScript + Rust</p>
        <p className="text-sm mt-1">Supports YOLO, COCO, VOC, Label Studio, TFRecord formats</p>
      </div>
    </div>
  );
}
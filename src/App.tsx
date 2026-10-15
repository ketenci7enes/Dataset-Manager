import { HashRouter, Routes, Route, Navigate, useLocation, useNavigate } from 'react-router-dom';
import Home from './pages/Home';
import Dashboard from './pages/Dashboard';
import Explorer from './pages/Explorer';
import Converter from './pages/Converter';
import Organizer from './pages/Organizer';
import Mixer from './pages/Mixer';
import Augment from './pages/Augment';
import Splitter from './pages/Splitter';
import Settings from './pages/Settings';
import { WorkspaceProvider, useWorkspace, buildWorkspaceInfo } from './hooks/useWorkspace';
import { DatasetProvider } from './hooks/useDataset';

function AppContent() {
  const { currentWorkspace, loading } = useWorkspace();

  if (loading) {
    return (
      <div className="flex items-center justify-center h-screen bg-gray-100">
        <div className="animate-spin rounded-full h-12 w-12 border-4 border-blue-500 border-t-transparent"></div>
      </div>
    );
  }

  if (!currentWorkspace) {
    return <Home />;
  }

  return (
    <div className="min-h-screen bg-gray-50 flex flex-col">
      <header className="bg-white shadow-sm border-b border-gray-200 sticky top-0 z-10">
        <nav className="max-w-full mx-auto px-4" aria-label="Main navigation">
          <div className="flex items-center justify-between h-16">
            <div className="flex items-center space-x-8">
              <div className="flex items-center space-x-2">
                <svg className="h-8 w-8 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
                </svg>
                <span className="text-xl font-bold text-gray-900">Dataset Manager</span>
              </div>
              <div className="hidden md:flex items-center space-x-1">
                <NavLink to="/dashboard">Dashboard</NavLink>
                <NavLink to="/explorer">Explorer</NavLink>
                <NavLink to="/converter">Converter</NavLink>
                <NavLink to="/organizer">Organizer</NavLink>
                <NavLink to="/mixer">Mixer</NavLink>
                <NavLink to="/augment">Augment</NavLink>
                <NavLink to="/splitter">Splitter</NavLink>
              </div>
            </div>
            <div className="flex items-center space-x-4">
              <WorkspaceSelector />
            </div>
          </div>
        </nav>
      </header>

      <main className="flex-1 max-w-full mx-auto w-full px-4 py-6">
        <Routes>
          <Route path="/dashboard" element={<Dashboard />} />
          <Route path="/explorer" element={<Explorer />} />
          <Route path="/converter" element={<Converter />} />
          <Route path="/organizer" element={<Organizer />} />
          <Route path="/mixer" element={<Mixer />} />
          <Route path="/augment" element={<Augment />} />
          <Route path="/splitter" element={<Splitter />} />
          <Route path="/settings" element={<Settings />} />
          <Route path="*" element={<Navigate to="/dashboard" replace />} />
        </Routes>
      </main>
    </div>
  );
}

function NavLink({ to, children }: { to: string; children: React.ReactNode }) {
  const location = useLocation();
  const navigate = useNavigate();
  const isActive = location.pathname === to;
  return (
    <button
      onClick={() => navigate(to)}
      className={`px-3 py-2 rounded-lg text-sm font-medium transition-colors ${
        isActive
          ? 'bg-blue-50 text-blue-700'
          : 'text-gray-600 hover:bg-gray-100 hover:text-gray-900'
      }`}
    >
      {children}
    </button>
  );
}

function WorkspaceSelector() {
  const { currentWorkspace, workspaces, setCurrentWorkspace, addWorkspace } = useWorkspace();
  const navigate = useNavigate();
  return (
    <div className="flex items-center space-x-2">
      <select
        value={currentWorkspace?.id || ''}
        onChange={(e) => {
          const ws = workspaces.find(w => w.id === e.target.value);
          if (ws) {
            setCurrentWorkspace(ws);
            navigate('/dashboard');
          }
        }}
        className="px-3 py-1.5 text-sm border border-gray-300 rounded-lg bg-white focus:outline-none focus:ring-2 focus:ring-blue-500"
      >
        <option value="">Select Workspace</option>
        {workspaces.map(ws => (
          <option key={ws.id} value={ws.id}>{ws.name}</option>
        ))}
      </select>
      <button
        onClick={async () => {
          try {
            const { open } = await import('@tauri-apps/api/dialog');
            const path = await open({ directory: true, title: 'Select Dataset Folder' });
            if (path) {
              const { invoke } = await import('@tauri-apps/api/tauri');
              const ws = await invoke('detect_workspace_format', { path });
              const fullWs = buildWorkspaceInfo(path as string, ws as any);
              addWorkspace(fullWs);
              navigate('/dashboard');
            }
          } catch (err) {
            console.error('Failed to pick workspace:', err);
          }
        }}
        className="px-3 py-1.5 text-sm font-medium text-white bg-blue-600 rounded-lg hover:bg-blue-700 transition-colors"
      >
        New Workspace
      </button>
    </div>
  );
}

function App() {
  return (
    <WorkspaceProvider>
      <DatasetProvider>
        <HashRouter>
          <AppContent />
        </HashRouter>
      </DatasetProvider>
    </WorkspaceProvider>
  );
}

export default App;
import { createContext, useContext, useState, ReactNode } from 'react';
import { DetectedFormat, WorkspaceInfo } from '../types/dataset';

export function buildWorkspaceInfo(path: string, detectedFormat: DetectedFormat): WorkspaceInfo {
  const normalized = path.replace(/[\\/]+$/, '');
  const name = normalized.split(/[\\/]/).filter(Boolean).pop() || normalized;
  return {
    id: normalized,
    path,
    name,
    detected_format: detectedFormat,
    last_opened: new Date().toISOString(),
  };
}

interface WorkspaceContextType {
  currentWorkspace: WorkspaceInfo | null;
  workspaces: WorkspaceInfo[];
  loading: boolean;
  setCurrentWorkspace: (ws: WorkspaceInfo) => void;
  addWorkspace: (ws: WorkspaceInfo) => void;
  removeWorkspace: (id: string) => void;
}

const WorkspaceContext = createContext<WorkspaceContextType | undefined>(undefined);

export function WorkspaceProvider({ children }: { children: ReactNode }) {
  const [currentWorkspace, setCurrentWorkspaceState] = useState<WorkspaceInfo | null>(null);
  const [workspaces, setWorkspaces] = useState<WorkspaceInfo[]>([]);
  const [loading] = useState(false);

  const setCurrentWorkspace = (ws: WorkspaceInfo) => {
    setCurrentWorkspaceState(ws);
  };

  const addWorkspace = (ws: WorkspaceInfo) => {
    const existing = workspaces.find((w) => w.id === ws.id);
    if (existing) {
      setCurrentWorkspaceState(existing);
      return;
    }
    const newWorkspaces = [ws, ...workspaces];
    setWorkspaces(newWorkspaces);
    setCurrentWorkspaceState(ws);
  };

  const removeWorkspace = (id: string) => {
    const newWorkspaces = workspaces.filter(w => w.id !== id);
    setWorkspaces(newWorkspaces);
    if (currentWorkspace?.id === id) {
      setCurrentWorkspaceState(newWorkspaces[0] || null);
    }
  };

  return (
    <WorkspaceContext.Provider value={{
      currentWorkspace,
      workspaces,
      loading,
      setCurrentWorkspace,
      addWorkspace,
      removeWorkspace,
    }}>
      {children}
    </WorkspaceContext.Provider>
  );
}

export function useWorkspace() {
  const context = useContext(WorkspaceContext);
  if (!context) {
    throw new Error('useWorkspace must be used within a WorkspaceProvider');
  }
  return context;
}
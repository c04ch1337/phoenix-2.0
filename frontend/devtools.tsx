import React, { useEffect, useMemo, useState } from 'react';

type SystemStatus = {
  full_access_granted: boolean;
  self_modification_enabled: boolean;
};

const PHOENIX_API_BASE =
  ((import.meta as any).env?.VITE_PHOENIX_API_BASE as string | undefined)?.replace(/\/$/, '') || '';

function apiUrl(path: string) {
  // If VITE_PHOENIX_API_BASE isn't set, we rely on same-origin (prod) or Vite dev proxy.
  return PHOENIX_API_BASE ? `${PHOENIX_API_BASE}${path}` : path;
}

export const DevToolsView: React.FC = () => {
  const [status, setStatus] = useState<SystemStatus | null>(null);
  const [loading, setLoading] = useState<string | null>(null);

  const [cmd, setCmd] = useState('cargo --version');
  const [cwd, setCwd] = useState('');
  const [execOut, setExecOut] = useState<{ exit_code: number; stdout: string; stderr: string } | null>(null);
  const [execErr, setExecErr] = useState<string | null>(null);

  const [readPath, setReadPath] = useState('README.md');
  const [readContent, setReadContent] = useState<string>('');
  const [readErr, setReadErr] = useState<string | null>(null);

  const [writePath, setWritePath] = useState('tmp/self_mod_test.txt');
  const [writeContent, setWriteContent] = useState('Phoenix self-mod is online.');
  const [writeErr, setWriteErr] = useState<string | null>(null);
  const [writeOk, setWriteOk] = useState<boolean>(false);

  const statusText = useMemo(() => {
    if (!status) return 'Unknown';
    const a = status.full_access_granted ? 'FULL_ACCESS' : 'NO_ACCESS';
    const s = status.self_modification_enabled ? 'SELF_MOD=ON' : 'SELF_MOD=OFF';
    return `${a} • ${s}`;
  }, [status]);

  const refreshStatus = async () => {
    setLoading('status');
    try {
      const res = await fetch(apiUrl('/api/system/status'));
      const j = await res.json();
      setStatus(j);
    } catch (e: any) {
      setStatus(null);
    } finally {
      setLoading(null);
    }
  };

  useEffect(() => {
    refreshStatus();
  }, []);

  const runExec = async () => {
    setLoading('exec');
    setExecErr(null);
    try {
      const res = await fetch(apiUrl('/api/system/exec'), {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ command: cmd, cwd: cwd.trim() ? cwd.trim() : undefined }),
      });
      const j = await res.json();
      if (!res.ok) {
        setExecOut(null);
        setExecErr(j?.message || `Exec failed (${res.status})`);
        return;
      }
      setExecOut(j);
    } catch (e: any) {
      setExecOut(null);
      setExecErr(e?.message || String(e));
    } finally {
      setLoading(null);
      refreshStatus();
    }
  };

  const runRead = async () => {
    setLoading('read');
    setReadErr(null);
    try {
      const res = await fetch(apiUrl('/api/system/read-file'), {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ path: readPath }),
      });
      const j = await res.json();
      if (!res.ok) {
        setReadContent('');
        setReadErr(j?.message || `Read failed (${res.status})`);
        return;
      }
      setReadContent(j?.content ?? '');
    } catch (e: any) {
      setReadContent('');
      setReadErr(e?.message || String(e));
    } finally {
      setLoading(null);
      refreshStatus();
    }
  };

  const runWrite = async () => {
    setLoading('write');
    setWriteErr(null);
    setWriteOk(false);
    try {
      const res = await fetch(apiUrl('/api/system/write-file'), {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ path: writePath, content: writeContent }),
      });
      const j = await res.json();
      if (!res.ok) {
        setWriteErr(j?.message || `Write failed (${res.status})`);
        return;
      }
      setWriteOk(true);
    } catch (e: any) {
      setWriteErr(e?.message || String(e));
    } finally {
      setLoading(null);
      refreshStatus();
    }
  };

  return (
    <div className="h-full bg-[#0f0b15] overflow-y-auto custom-scrollbar">
      <div className="max-w-5xl mx-auto p-8 space-y-8">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-2xl font-bold text-white">Self-Modification Console</h2>
            <p className="text-gray-400 text-sm">Direct host control endpoints via <span className="font-mono">/api/system/*</span>.</p>
          </div>
          <button
            onClick={refreshStatus}
            className="px-4 py-2 bg-white/5 hover:bg-white/10 text-gray-200 rounded-lg border border-white/10 text-sm"
          >
            {loading === 'status' ? 'Refreshing…' : 'Refresh'}
          </button>
        </div>

        <div className="glass-panel p-6 rounded-2xl border border-white/10">
          <div className="flex items-center justify-between">
            <div>
              <div className="text-xs text-gray-500 uppercase tracking-wider font-semibold">System Access</div>
              <div className="text-white font-mono mt-1">{statusText}</div>
            </div>
            <div className="text-xs text-gray-500">Local-only (binds to <span className="font-mono">127.0.0.1</span>)</div>
          </div>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <div className="glass-panel p-6 rounded-2xl border border-white/10">
            <h3 className="text-white font-bold mb-4">Execute Command</h3>
            <div className="space-y-3">
              <input
                value={cmd}
                onChange={(e) => setCmd(e.target.value)}
                className="w-full bg-void-900 border border-white/10 rounded-lg p-3 text-white outline-none focus:border-phoenix-500"
                placeholder="Command line (executed in OS shell)"
              />
              <input
                value={cwd}
                onChange={(e) => setCwd(e.target.value)}
                className="w-full bg-void-900 border border-white/10 rounded-lg p-3 text-white outline-none focus:border-phoenix-500"
                placeholder="Optional working directory (cwd)"
              />
              <button
                onClick={runExec}
                className="w-full bg-phoenix-600 hover:bg-phoenix-500 text-white rounded-lg py-2.5 font-semibold"
                disabled={loading === 'exec'}
              >
                {loading === 'exec' ? 'Running…' : 'Run'}
              </button>
              {execErr && <div className="text-red-400 text-xs font-mono">{execErr}</div>}
              {execOut && (
                <div className="bg-black/40 border border-white/10 rounded-lg p-3 font-mono text-xs text-gray-200 space-y-2">
                  <div>exit_code: <span className="text-white">{execOut.exit_code}</span></div>
                  <div className="text-gray-400">stdout:</div>
                  <pre className="whitespace-pre-wrap wrap-break-word">{execOut.stdout || '(empty)'}</pre>
                  <div className="text-gray-400">stderr:</div>
                  <pre className="whitespace-pre-wrap wrap-break-word">{execOut.stderr || '(empty)'}</pre>
                </div>
              )}
            </div>
          </div>

          <div className="glass-panel p-6 rounded-2xl border border-white/10">
            <h3 className="text-white font-bold mb-4">Read File</h3>
            <div className="space-y-3">
              <input
                value={readPath}
                onChange={(e) => setReadPath(e.target.value)}
                className="w-full bg-void-900 border border-white/10 rounded-lg p-3 text-white outline-none focus:border-phoenix-500"
                placeholder="Path"
              />
              <button
                onClick={runRead}
                className="w-full bg-white/5 hover:bg-white/10 text-white rounded-lg py-2.5 font-semibold border border-white/10"
                disabled={loading === 'read'}
              >
                {loading === 'read' ? 'Reading…' : 'Read'}
              </button>
              {readErr && <div className="text-red-400 text-xs font-mono">{readErr}</div>}
              <textarea
                value={readContent}
                readOnly
                className="w-full h-56 bg-black/40 border border-white/10 rounded-lg p-3 text-gray-200 outline-none font-mono text-xs"
                placeholder="File contents"
              />
            </div>
          </div>
        </div>

        <div className="glass-panel p-6 rounded-2xl border border-white/10">
          <h3 className="text-white font-bold mb-4">Write File</h3>
          <div className="space-y-3">
            <input
              value={writePath}
              onChange={(e) => setWritePath(e.target.value)}
              className="w-full bg-void-900 border border-white/10 rounded-lg p-3 text-white outline-none focus:border-phoenix-500"
              placeholder="Path"
            />
            <textarea
              value={writeContent}
              onChange={(e) => setWriteContent(e.target.value)}
              className="w-full h-40 bg-void-900 border border-white/10 rounded-lg p-3 text-white outline-none font-mono text-xs focus:border-phoenix-500"
              placeholder="Content"
            />
            <div className="flex items-center gap-3">
              <button
                onClick={runWrite}
                className="px-6 py-2.5 bg-green-600/20 hover:bg-green-600/30 text-green-300 border border-green-600/40 rounded-lg font-semibold"
                disabled={loading === 'write'}
              >
                {loading === 'write' ? 'Writing…' : 'Write'}
              </button>
              {writeOk && <div className="text-green-400 text-xs font-mono">OK</div>}
              {writeErr && <div className="text-red-400 text-xs font-mono">{writeErr}</div>}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};


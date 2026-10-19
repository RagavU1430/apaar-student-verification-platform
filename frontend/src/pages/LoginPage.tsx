import { useState } from "react";
import { Navigate, useNavigate } from "react-router-dom";
import { useAuth } from "../App";
import { ShieldCheck, AlertCircle, ArrowRight, Sparkles } from "lucide-react";

export default function LoginPage() {
  const { login, user } = useAuth();
  const navigate = useNavigate();
  const [loginIdentifier, setLoginIdentifier] = useState("alpha.admin");
  const [password, setPassword] = useState("DemoPass123!");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  if (user) {
    return <Navigate to="/dashboard" replace />;
  }

  async function onSubmit(event: React.FormEvent) {
    event.preventDefault();
    setError(null);
    setBusy(true);
    try {
      await login(loginIdentifier, password);
      navigate("/dashboard", { replace: true });
    } catch (err: any) {
      setError(err?.message || "Sign in failed. Please check credentials.");
    } finally {
      setBusy(false);
    }
  }

  const selectDemoAccount = (identifier: string) => {
    setLoginIdentifier(identifier);
    setPassword("DemoPass123!");
  };

  return (
    <div className="min-h-screen bg-slate-900 flex flex-col justify-center items-center px-4 py-12 relative overflow-hidden">
      {/* Background aesthetic decoration */}
      <div className="absolute top-0 -left-4 w-96 h-96 bg-blue-500/10 rounded-full blur-3xl pointer-events-none" />
      <div className="absolute bottom-0 -right-4 w-96 h-96 bg-indigo-500/10 rounded-full blur-3xl pointer-events-none" />

      <div className="w-full max-w-md bg-white rounded-2xl shadow-2xl border border-slate-100 p-8 relative z-10">
        <div className="text-center mb-6">
          <div className="w-12 h-12 bg-blue-600 rounded-xl mx-auto flex items-center justify-center text-white mb-3 shadow-md shadow-blue-500/20">
            <ShieldCheck className="w-6 h-6" />
          </div>
          <h1 className="text-2xl font-bold text-slate-900 tracking-tight">AICTE APAAR</h1>
          <p className="text-xs text-slate-500 uppercase tracking-wider font-semibold mt-1">
            Student Data &amp; Verification Platform
          </p>
        </div>

        <form onSubmit={onSubmit} className="space-y-4">
          <div>
            <label className="block text-xs font-semibold uppercase text-slate-600 mb-1">
              Login Identifier or Email
            </label>
            <div className="relative">
              <input
                id="login_identifier"
                name="login_identifier"
                autoComplete="username"
                value={loginIdentifier}
                onChange={(e) => setLoginIdentifier(e.target.value)}
                placeholder="e.g. alpha.admin or aicte.admin"
                className="w-full px-3.5 py-2.5 text-sm border border-slate-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 font-mono"
                required
              />
            </div>
          </div>

          <div>
            <label className="block text-xs font-semibold uppercase text-slate-600 mb-1">
              Password
            </label>
            <div className="relative">
              <input
                id="password"
                name="password"
                type="password"
                autoComplete="current-password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                className="w-full px-3.5 py-2.5 text-sm border border-slate-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 font-mono"
                required
              />
            </div>
          </div>

          {error && (
            <div className="p-3 bg-red-50 border border-red-200 rounded-lg text-xs text-red-700 flex items-start gap-2">
              <AlertCircle className="w-4 h-4 shrink-0 text-red-600 mt-0.5" />
              <span>{error}</span>
            </div>
          )}

          <button
            type="submit"
            disabled={busy}
            className="w-full btn-primary py-2.5 justify-center text-sm font-semibold shadow-md shadow-blue-600/20"
          >
            {busy ? "Signing in..." : (
              <>
                <span>Sign In to Platform</span>
                <ArrowRight className="w-4 h-4" />
              </>
            )}
          </button>
        </form>

        {/* Demo Accounts Quick-Select */}
        <div className="mt-8 pt-6 border-t border-slate-100">
          <div className="flex items-center gap-1.5 text-xs font-bold text-slate-700 uppercase tracking-wider mb-2.5">
            <Sparkles className="w-3.5 h-3.5 text-amber-500" />
            <span>1-Click Demo Accounts</span>
          </div>
          <div className="grid grid-cols-2 gap-2 text-xs">
            <button
              type="button"
              onClick={() => selectDemoAccount("alpha.admin")}
              className="p-2 border border-slate-200 hover:border-blue-400 rounded-lg text-left transition hover:bg-blue-50/50"
            >
              <div className="font-semibold text-slate-800">alpha.admin</div>
              <div className="text-[10px] text-slate-500">Alpha Inst Admin</div>
            </button>
            <button
              type="button"
              onClick={() => selectDemoAccount("aicte.admin")}
              className="p-2 border border-slate-200 hover:border-blue-400 rounded-lg text-left transition hover:bg-blue-50/50"
            >
              <div className="font-semibold text-slate-800">aicte.admin</div>
              <div className="text-[10px] text-slate-500">AICTE Regulator</div>
            </button>
            <button
              type="button"
              onClick={() => selectDemoAccount("alpha.operator")}
              className="p-2 border border-slate-200 hover:border-blue-400 rounded-lg text-left transition hover:bg-blue-50/50"
            >
              <div className="font-semibold text-slate-800">alpha.operator</div>
              <div className="text-[10px] text-slate-500">Data Operator</div>
            </button>
            <button
              type="button"
              onClick={() => selectDemoAccount("verifier")}
              className="p-2 border border-slate-200 hover:border-blue-400 rounded-lg text-left transition hover:bg-blue-50/50"
            >
              <div className="font-semibold text-slate-800">verifier</div>
              <div className="text-[10px] text-slate-500">Auth Verifier</div>
            </button>
          </div>
          <p className="text-[11px] text-slate-400 text-center mt-3">
            Password: <code className="bg-slate-100 px-1 py-0.5 rounded text-slate-600">DemoPass123!</code>
          </p>
        </div>
      </div>
    </div>
  );
}

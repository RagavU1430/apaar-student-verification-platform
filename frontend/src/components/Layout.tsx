import { Link, Outlet, useLocation, useNavigate } from 'react-router-dom';
import { useAuth } from '../App';
import {
  Building2,
  UploadCloud,
  Users,
  ShieldCheck,
  AlertTriangle,
  History,
  LogOut,
  Landmark,
  Layers
} from 'lucide-react';

export default function Layout() {
  const { user, logout } = useAuth();
  const navigate = useNavigate();
  const location = useLocation();

  const isAicte = user?.permissions.includes('dashboard:aicte');
  const canUpload = user?.permissions.includes('batch:upload');
  const canAudit = user?.permissions.includes('audit:read');

  const navItems = [
    {
      to: '/dashboard',
      label: 'Institution Dashboard',
      icon: Building2,
      show: true
    },
    {
      to: '/aicte',
      label: 'AICTE Regulator View',
      icon: Landmark,
      show: isAicte
    },
    {
      to: '/admissions',
      label: 'Student Registry',
      icon: Users,
      show: true
    },
    {
      to: '/upload',
      label: 'Upload Wizard',
      icon: UploadCloud,
      show: canUpload
    },
    {
      to: '/issues',
      label: 'Review Queue',
      icon: AlertTriangle,
      show: true
    },
    {
      to: '/verify',
      label: 'Verification Service',
      icon: ShieldCheck,
      show: true
    },
    {
      to: '/audit',
      label: 'Audit Trail',
      icon: History,
      show: canAudit || isAicte
    }
  ];

  return (
    <div className="app-shell bg-slate-50 min-h-screen flex flex-col">
      <header className="bg-slate-900 text-white shadow-md border-b border-slate-800">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex items-center justify-between h-16">
            {/* Brand Logo & Prototype Badge */}
            <div className="flex items-center gap-3">
              <div className="bg-blue-600 text-white p-2 rounded-lg flex items-center justify-center">
                <Layers className="w-5 h-5" />
              </div>
              <div className="flex flex-col">
                <span className="font-bold text-base tracking-tight leading-none text-white">
                  AICTE APAAR
                </span>
                <span className="text-[10px] text-blue-400 font-medium tracking-wider uppercase mt-1">
                  Data &amp; Verification Platform
                </span>
              </div>
              <span className="bg-amber-400/20 text-amber-300 text-[10px] uppercase font-bold tracking-wider px-2 py-0.5 rounded-full border border-amber-400/30 ml-1">
                Demo
              </span>
            </div>

            {/* Navigation Links */}
            <nav className="hidden lg:flex items-center gap-1">
              {navItems
                .filter((item) => item.show)
                .map((item) => {
                  const Icon = item.icon;
                  const isActive = location.pathname === item.to || location.pathname.startsWith(item.to + '/');
                  return (
                    <Link
                      key={item.to}
                      to={item.to}
                      className={`flex items-center gap-2 px-3 py-2 rounded-lg text-xs font-medium transition-colors ${
                        isActive
                          ? 'bg-blue-600 text-white font-semibold'
                          : 'text-slate-300 hover:text-white hover:bg-slate-800'
                      }`}
                    >
                      <Icon className="w-4 h-4" />
                      <span>{item.label}</span>
                    </Link>
                  );
                })}
            </nav>

            {/* User Profile & Logout */}
            <div className="flex items-center gap-4">
              <div className="text-right hidden sm:block">
                <div className="text-xs font-semibold text-white">{user?.display_name}</div>
                <div className="text-[11px] text-slate-400 font-mono">
                  {user?.role} {user?.institution_name ? `• ${user.institution_name}` : ''}
                </div>
              </div>

              <button
                type="button"
                onClick={() => {
                  logout();
                  navigate('/login');
                }}
                className="flex items-center gap-1.5 text-xs bg-slate-800 hover:bg-slate-700 text-slate-200 px-3 py-1.5 rounded-lg border border-slate-700 transition"
                title="Sign out"
              >
                <LogOut className="w-3.5 h-3.5" />
                <span>Sign out</span>
              </button>
            </div>
          </div>
        </div>

        {/* Mobile Nav */}
        <div className="lg:hidden flex overflow-x-auto px-4 py-2 border-t border-slate-800 gap-2 bg-slate-950/80">
          {navItems
            .filter((item) => item.show)
            .map((item) => {
              const Icon = item.icon;
              const isActive = location.pathname === item.to;
              return (
                <Link
                  key={item.to}
                  to={item.to}
                  className={`flex items-center gap-1.5 px-2.5 py-1.5 rounded-md text-xs whitespace-nowrap ${
                    isActive ? 'bg-blue-600 text-white' : 'text-slate-300 hover:bg-slate-800'
                  }`}
                >
                  <Icon className="w-3.5 h-3.5" />
                  <span>{item.label}</span>
                </Link>
              );
            })}
        </div>
      </header>

      {/* Main Page Content */}
      <main className="flex-1 max-w-7xl w-full mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <Outlet />
      </main>

      {/* Footer */}
      <footer className="bg-white border-t border-slate-200 py-4 mt-auto">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 text-center sm:text-left flex flex-col sm:flex-row justify-between items-center gap-2 text-xs text-slate-500">
          <p>
            Demo Build for Hackathon Delivery. Counting rule: <code>demo-v1</code>. No live government verification gateway is claimed.
          </p>
          <p className="font-mono text-[11px] text-slate-400">
            PostgreSQL + Supabase + React + TypeScript + RLS
          </p>
        </div>
      </footer>
    </div>
  );
}

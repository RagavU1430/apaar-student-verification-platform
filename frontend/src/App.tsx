import { createContext, useContext, useEffect, useMemo, useState } from "react";
import { Navigate, Route, Routes } from "react-router-dom";
import { api, getToken, setToken, type UserProfile } from "./api/client";
import Layout from "./components/Layout";
import LoginPage from "./pages/LoginPage";
import InstitutionDashboardPage from "./pages/InstitutionDashboardPage";
import UploadPage from "./pages/UploadPage";
import BatchDetailPage from "./pages/BatchDetailPage";
import AicteDashboardPage from "./pages/AicteDashboardPage";
import IssuesPage from "./pages/IssuesPage";
import AdmissionsPage from "./pages/AdmissionsPage";
import VerificationPage from "./pages/VerificationPage";
import AuditPage from "./pages/AuditPage";

interface AuthState {
  user: UserProfile | null;
  loading: boolean;
  login: (loginIdentifier: string, password: string) => Promise<void>;
  logout: () => void;
}

const AuthContext = createContext<AuthState>({
  user: null,
  loading: true,
  login: async () => {},
  logout: () => {},
});

export function useAuth(): AuthState {
  return useContext(AuthContext);
}

/** Requires a signed-in user. */
function RequireAuth({ children }: { children: JSX.Element }) {
  const { user, loading } = useAuth();
  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-slate-50">
        <p className="text-sm text-slate-500 font-medium">Checking your session...</p>
      </div>
    );
  }
  if (!user) {
    return <Navigate to="/login" replace />;
  }
  return children;
}

/** Requires a specific permission, checked on the server as well. */
function RequirePermission({
  permission,
  children,
}: {
  permission: string;
  children: JSX.Element;
}) {
  const { user } = useAuth();
  if (!user) {
    return <Navigate to="/login" replace />;
  }
  if (!user.permissions.includes(permission)) {
    return (
      <div className="panel bg-white border border-slate-200">
        <h2 className="text-lg font-semibold text-slate-800">Not available for your role</h2>
        <p className="text-sm text-slate-500 mt-2">
          Your role (<code>{user.role}</code>) does not include this view. The Supabase RLS and server policies enforce the same rule.
        </p>
      </div>
    );
  }
  return children;
}

export default function App() {
  const [user, setUser] = useState<UserProfile | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    api
      .me()
      .then((profile) => {
        if (profile) {
          setUser(profile);
        } else if (getToken()) {
          // If stored token exists but no profile, clear
          setToken(null);
        }
      })
      .catch((error: any) => {
        if (error.status === 401) {
          setToken(null);
        }
      })
      .finally(() => setLoading(false));
  }, []);

  const value = useMemo<AuthState>(
    () => ({
      user,
      loading,
      login: async (loginIdentifier: string, password: string) => {
        const profile = await api.login(loginIdentifier, password);
        setToken(profile.id);
        setUser(profile);
      },
      logout: () => {
        api.logout();
        setToken(null);
        setUser(null);
      },
    }),
    [user, loading],
  );

  return (
    <AuthContext.Provider value={value}>
      <Routes>
        <Route path="/login" element={<LoginPage />} />
        <Route
          path="/"
          element={
            <RequireAuth>
              <Layout />
            </RequireAuth>
          }
        >
          <Route index element={<Navigate to="/dashboard" replace />} />
          <Route path="dashboard" element={<InstitutionDashboardPage />} />
          <Route
            path="aicte"
            element={
              <RequirePermission permission="dashboard:aicte">
                <AicteDashboardPage />
              </RequirePermission>
            }
          />
          <Route
            path="upload"
            element={
              <RequirePermission permission="batch:upload">
                <UploadPage />
              </RequirePermission>
            }
          />
          <Route path="admissions" element={<AdmissionsPage />} />
          <Route path="batches/:id" element={<BatchDetailPage />} />
          <Route path="issues" element={<IssuesPage />} />
          <Route path="verify" element={<VerificationPage />} />
          <Route
            path="audit"
            element={
              <RequirePermission permission="audit:read">
                <AuditPage />
              </RequirePermission>
            }
          />
        </Route>
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </AuthContext.Provider>
  );
}

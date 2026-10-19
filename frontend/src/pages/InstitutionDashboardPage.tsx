import { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import { api, type InstitutionDashboardData } from '../api/client';
import StatusBadge from '../components/StatusBadge';
import { Building2, UploadCloud, AlertCircle, BarChart3, Layers } from 'lucide-react';

export default function InstitutionDashboardPage() {
  const [data, setData] = useState<InstitutionDashboardData | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    api
      .institutionDashboard()
      .then(setData)
      .catch((err: any) =>
        setError(
          err.message || 'This dashboard requires an institution-scoped account. Sign in as alpha.admin to view it.'
        )
      );
  }, []);

  if (error) {
    return (
      <div className="panel bg-white border border-red-200">
        <h2 className="text-red-700 flex items-center gap-2">
          <AlertCircle className="w-5 h-5 text-red-600" />
          Dashboard Unavailable
        </h2>
        <p className="text-sm text-red-600 mt-2">{error}</p>
      </div>
    );
  }

  if (!data) {
    return <div className="p-12 text-center text-slate-500">Loading institution dashboard...</div>;
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h1 className="text-2xl font-bold text-slate-900 flex items-center gap-2.5">
            <Building2 className="w-7 h-7 text-blue-600" />
            {data.institution_name ?? 'Institution Dashboard'}
          </h1>
          <p className="text-xs text-slate-500 mt-1">
            Data Source: <strong className="text-slate-700">{data.data_source}</strong> &middot; Rule Version: <code>{data.rule_version}</code> &middot; Updated: {new Date().toLocaleTimeString()}
          </p>
        </div>

        <Link to="/upload" className="btn-primary self-start sm:self-auto text-xs">
          <UploadCloud className="w-4 h-4" />
          Upload New Batch
        </Link>
      </div>

      {/* Metrics Grid */}
      <section className="metric-grid" aria-label="Submission summary">
        <div className="metric">
          <span className="metric-label">Batches</span>
          <span className="metric-value">{data.counts.total_batches}</span>
        </div>
        <div className="metric">
          <span className="metric-label">Rows Ingested</span>
          <span className="metric-value">{data.counts.total_rows}</span>
        </div>
        <div className="metric">
          <span className="metric-label">Accepted</span>
          <span className="metric-value text-emerald-600">{data.counts.accepted_rows}</span>
        </div>
        <div className="metric">
          <span className="metric-label">Rejected</span>
          <span className="metric-value text-red-600">{data.counts.rejected_rows}</span>
        </div>
        <div className="metric">
          <span className="metric-label">Needs Review</span>
          <span className="metric-value text-purple-600">{data.counts.rows_needing_review}</span>
        </div>
        <div className="metric">
          <span className="metric-label">Duplicates</span>
          <span className="metric-value text-amber-600">{data.counts.duplicate_candidates}</span>
        </div>
      </section>

      {/* Intake Monitoring Section */}
      <section className="panel bg-white border border-slate-200 p-0 overflow-hidden">
        <div className="p-4 border-b border-slate-100 flex items-center justify-between">
          <div>
            <h2 className="text-base font-semibold text-slate-800 flex items-center gap-2 m-0">
              <BarChart3 className="w-5 h-5 text-blue-600" />
              Approved Intake &amp; Capacity Utilisation
            </h2>
            <p className="text-xs text-slate-500 mt-0.5">
              Comparison of active admissions against configured capacity under <code>demo-v1</code>
            </p>
          </div>
          <span className="text-xs bg-slate-100 text-slate-600 px-2.5 py-1 rounded-md font-mono">
            Academic Year: 2026-27
          </span>
        </div>

        {data.intake.length === 0 ? (
          <p className="p-8 text-center text-slate-500 text-sm">No approved intake configured.</p>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-left border-collapse m-0">
              <thead>
                <tr className="bg-slate-50 text-xs font-semibold text-slate-600 uppercase">
                  <th className="py-3 px-4">Programme</th>
                  <th className="py-3 px-4">Regular Admissions</th>
                  <th className="py-3 px-4">Supernumerary</th>
                  <th className="py-3 px-4">Utilisation %</th>
                  <th className="py-3 px-4">Capacity Status</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-100 text-sm">
                {data.intake.map((intake) => {
                  const util = intake.utilisation ?? 0;
                  const isOver = intake.over_capacity;
                  return (
                    <tr key={`${intake.programme_code}-${intake.academic_year}`} className="hover:bg-slate-50/80">
                      <td className="py-3 px-4">
                        <span className="font-mono font-bold text-slate-800 text-xs bg-slate-100 px-2 py-0.5 rounded">
                          {intake.programme_code}
                        </span>
                        {intake.programme_name && (
                          <span className="text-slate-600 text-xs ml-2">{intake.programme_name}</span>
                        )}
                      </td>
                      <td className="py-3 px-4 font-medium text-slate-900">
                        {intake.regular_count} / {intake.regular_intake} seats
                      </td>
                      <td className="py-3 px-4 text-slate-600">
                        {intake.supernumerary_count} / {intake.supernumerary_capacity} pool
                      </td>
                      <td className="py-3 px-4">
                        <div className="flex items-center gap-3">
                          <div className="w-24 bg-slate-200 rounded-full h-2 overflow-hidden">
                            <div
                              className={`h-full rounded-full ${
                                isOver ? 'bg-red-500' : util > 90 ? 'bg-amber-500' : 'bg-blue-600'
                              }`}
                              style={{ width: `${Math.min(util, 100)}%` }}
                            />
                          </div>
                          <span className={`text-xs font-semibold ${isOver ? 'text-red-600' : 'text-slate-700'}`}>
                            {util}%
                          </span>
                        </div>
                      </td>
                      <td className="py-3 px-4">
                        {isOver ? (
                          <StatusBadge label="OVER CAPACITY" />
                        ) : (
                          <StatusBadge label="WITHIN LIMITS" />
                        )}
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        )}
      </section>

      {/* Recent Submission Batches */}
      <section className="panel bg-white border border-slate-200 p-0 overflow-hidden">
        <div className="p-4 border-b border-slate-100 flex items-center justify-between">
          <h2 className="text-base font-semibold text-slate-800 flex items-center gap-2 m-0">
            <Layers className="w-5 h-5 text-indigo-600" />
            Recent Submission Batches ({data.batches.length})
          </h2>
          <Link to="/upload" className="text-xs text-blue-600 hover:underline">
            Submit new &rarr;
          </Link>
        </div>

        {data.batches.length === 0 ? (
          <div className="p-8 text-center text-slate-500 text-sm">
            <p>No batches uploaded yet.</p>
            <Link to="/upload" className="btn-primary mt-3 inline-flex text-xs">
              Upload First Batch
            </Link>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-left border-collapse m-0">
              <thead>
                <tr className="bg-slate-50 text-xs font-semibold text-slate-600 uppercase">
                  <th className="py-3 px-4">File Name</th>
                  <th className="py-3 px-4">Date</th>
                  <th className="py-3 px-4">Rows</th>
                  <th className="py-3 px-4">Accepted</th>
                  <th className="py-3 px-4">Rejected</th>
                  <th className="py-3 px-4">Needs Review</th>
                  <th className="py-3 px-4">Status</th>
                  <th className="py-3 px-4 text-right">Action</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-100 text-sm">
                {data.batches.map((batch) => (
                  <tr key={batch.id} className="hover:bg-slate-50/80">
                    <td className="py-3 px-4 font-mono font-medium text-slate-900 text-xs">{batch.file_name}</td>
                    <td className="py-3 px-4 text-xs text-slate-500">{new Date(batch.submitted_at).toLocaleDateString()}</td>
                    <td className="py-3 px-4 font-semibold text-slate-800">{batch.rows}</td>
                    <td className="py-3 px-4 text-emerald-600 font-medium">{batch.accepted}</td>
                    <td className="py-3 px-4 text-red-600 font-medium">{batch.rejected}</td>
                    <td className="py-3 px-4 text-purple-600 font-medium">{batch.needs_review}</td>
                    <td className="py-3 px-4">
                      <StatusBadge label={batch.status} />
                    </td>
                    <td className="py-3 px-4 text-right">
                      <Link to={`/batches/${batch.id}`} className="text-xs font-semibold text-blue-600 hover:underline">
                        View Details &rarr;
                      </Link>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </section>
    </div>
  );
}

import { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import { api, type AicteDashboardData } from '../api/client';
import StatusBadge from '../components/StatusBadge';
import { Landmark, Download, AlertTriangle, Building2, BarChart2, CheckCircle2 } from 'lucide-react';
import { ResponsiveContainer, BarChart, Bar, XAxis, YAxis, Tooltip, Legend } from 'recharts';

export default function AicteDashboardPage() {
  const [data, setData] = useState<AicteDashboardData | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    api
      .aicteDashboard()
      .then(setData)
      .catch((err: any) => setError(err.message || 'Failed to load AICTE dashboard.'));
  }, []);

  const handleExportRegulatoryReport = async () => {
    try {
      const rows = await api.enrolmentReport('2026-27');
      const csv =
        'institution_id,institution_name,programme_code,academic_year,seat_category,count\n' +
        rows
          .map(
            (r) =>
              `${r.institution_id},"${r.institution_name}",${r.programme_code},${r.academic_year},${r.seat_category},${r.count}`
          )
          .join('\n');
      const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' });
      const url = URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = `aicte_enrolment_regulatory_report_2026-27.csv`;
      link.click();
      URL.revokeObjectURL(url);
    } catch (err: any) {
      alert('Export failed: ' + err.message);
    }
  };

  if (error) {
    return (
      <div className="panel bg-white border border-red-200">
        <h2 className="text-red-700">AICTE Dashboard Unavailable</h2>
        <p className="text-sm text-red-600 mt-2">{error}</p>
      </div>
    );
  }

  if (!data) {
    return <div className="p-12 text-center text-slate-500">Loading AICTE central monitoring dashboard...</div>;
  }

  // Prepare chart data for Recharts
  const chartData = data.intake.map((item) => ({
    name: item.programme_code,
    'Regular Admissions': item.regular_count,
    'Approved Intake': item.regular_intake,
    'Supernumerary Admissions': item.supernumerary_count,
    'Supernumerary Capacity': item.supernumerary_capacity
  }));

  return (
    <div className="space-y-6">
      {/* Top Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h1 className="text-2xl font-bold text-slate-900 flex items-center gap-2.5">
            <Landmark className="w-7 h-7 text-indigo-700" />
            AICTE Central Regulatory Overview
          </h1>
          <p className="text-xs text-slate-500 mt-1">
            Data Source: <strong className="text-slate-700">{data.data_source}</strong> &middot; Counting Rule: <code>{data.rule_version}</code> &middot; Generated: {new Date().toLocaleTimeString()}
          </p>
        </div>

        <button
          onClick={handleExportRegulatoryReport}
          className="btn-secondary self-start sm:self-auto text-xs"
        >
          <Download className="w-4 h-4 text-slate-600" />
          Export Regulatory Enrolment Report (CSV)
        </button>
      </div>

      {/* Metrics Grid */}
      <section className="metric-grid" aria-label="Cross-institution summary">
        <div className="metric">
          <span className="metric-label">Approved Institutions</span>
          <span className="metric-value">{data.institutions.length}</span>
        </div>
        <div className="metric">
          <span className="metric-label">Batches Ingested</span>
          <span className="metric-value">{data.counts.total_batches}</span>
        </div>
        <div className="metric">
          <span className="metric-label">Total Student Records</span>
          <span className="metric-value text-blue-600">{data.counts.total_rows}</span>
        </div>
        <div className="metric">
          <span className="metric-label">Accepted Admissions</span>
          <span className="metric-value text-emerald-600">{data.counts.accepted_rows}</span>
        </div>
        <div className="metric">
          <span className="metric-label">Exception Queue</span>
          <span className="metric-value text-amber-600">{data.exception_queue.length}</span>
        </div>
        <div className="metric">
          <span className="metric-label">Duplicate Candidates</span>
          <span className="metric-value text-red-600">{data.counts.duplicate_candidates}</span>
        </div>
      </section>

      {/* Recharts Visual Comparison */}
      <section className="panel bg-white border border-slate-200 p-6">
        <h2 className="text-base font-semibold text-slate-800 flex items-center gap-2 mb-4">
          <BarChart2 className="w-5 h-5 text-indigo-600" />
          Programme Intake vs. Actual Admissions (Central Comparison)
        </h2>
        <div className="h-72 w-full">
          <ResponsiveContainer width="100%" height="100%">
            <BarChart data={chartData} margin={{ top: 10, right: 20, left: 0, bottom: 20 }}>
              <XAxis dataKey="name" tick={{ fontSize: 12 }} />
              <YAxis tick={{ fontSize: 12 }} />
              <Tooltip />
              <Legend wrapperStyle={{ fontSize: '12px', paddingTop: '10px' }} />
              <Bar dataKey="Regular Admissions" fill="#3b82f6" radius={[4, 4, 0, 0]} />
              <Bar dataKey="Approved Intake" fill="#94a3b8" radius={[4, 4, 0, 0]} />
              <Bar dataKey="Supernumerary Admissions" fill="#8b5cf6" radius={[4, 4, 0, 0]} />
            </BarChart>
          </ResponsiveContainer>
        </div>
      </section>

      {/* Institutions Table */}
      <section className="panel bg-white border border-slate-200 p-0 overflow-hidden">
        <div className="p-4 border-b border-slate-100 flex items-center justify-between">
          <h2 className="text-base font-semibold text-slate-800 flex items-center gap-2 m-0">
            <Building2 className="w-5 h-5 text-blue-600" />
            Approved Participating Institutions ({data.institutions.length})
          </h2>
          <span className="text-xs text-slate-500">Cross-institution monitoring active</span>
        </div>

        <div className="overflow-x-auto">
          <table className="w-full text-left border-collapse m-0">
            <thead>
              <tr className="bg-slate-50 text-xs font-semibold text-slate-600 uppercase">
                <th className="py-3 px-4">Official Code</th>
                <th className="py-3 px-4">Institution Name</th>
                <th className="py-3 px-4">Active Enrolled Admissions</th>
                <th className="py-3 px-4 text-right">Action</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-100 text-sm">
              {data.institutions.map((institution) => (
                <tr key={institution.id} className="hover:bg-slate-50/80">
                  <td className="py-3 px-4 font-mono font-medium text-slate-800 text-xs">
                    {institution.official_code}
                  </td>
                  <td className="py-3 px-4 font-semibold text-slate-900">{institution.name}</td>
                  <td className="py-3 px-4 font-mono text-emerald-700 font-semibold">
                    {institution.active_admissions} students
                  </td>
                  <td className="py-3 px-4 text-right">
                    <Link
                      to="/admissions"
                      className="text-xs text-blue-600 hover:underline font-medium"
                    >
                      Inspect Registry &rarr;
                    </Link>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>

      {/* Exception & Review Queue */}
      <section className="panel bg-white border border-slate-200 p-0 overflow-hidden">
        <div className="p-4 border-b border-slate-100 flex items-center justify-between">
          <div>
            <h2 className="text-base font-semibold text-slate-800 flex items-center gap-2 m-0">
              <AlertTriangle className="w-5 h-5 text-amber-500" />
              Prioritised Exception &amp; Review Queue ({data.exception_queue.length})
            </h2>
            <p className="text-xs text-slate-500 mt-0.5">
              Requires human review or regulatory concurrence. Never treated as an automated fraud verdict.
            </p>
          </div>
          <Link to="/issues" className="text-xs text-blue-600 hover:underline">
            Manage all in review queue &rarr;
          </Link>
        </div>

        {data.exception_queue.length === 0 ? (
          <div className="p-8 text-center text-slate-500 text-sm">
            <CheckCircle2 className="w-8 h-8 text-emerald-500 mx-auto mb-2" />
            <p className="font-medium text-slate-700">Exception queue is clear.</p>
            <p className="text-xs text-slate-400 mt-1">No pending capacity exceptions or cross-institution conflicts.</p>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-left border-collapse m-0">
              <thead>
                <tr className="bg-slate-50 text-xs font-semibold text-slate-600 uppercase">
                  <th className="py-3 px-4">Location / Context</th>
                  <th className="py-3 px-4">Rule Code</th>
                  <th className="py-3 px-4">Severity</th>
                  <th className="py-3 px-4">Finding Explanation</th>
                  <th className="py-3 px-4">Concrete Evidence</th>
                  <th className="py-3 px-4 text-right">Action</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-100 text-sm">
                {data.exception_queue.map((item) => (
                  <tr key={item.issue_id} className="hover:bg-slate-50/80">
                    <td className="py-3 px-4 font-mono font-medium text-slate-800 text-xs">
                      {item.institution_name ? `${item.institution_name} • ` : ''}
                      {item.row_reference}
                    </td>
                    <td className="py-3 px-4 font-mono text-xs">
                      <code>{item.rule_code}</code>
                    </td>
                    <td className="py-3 px-4">
                      <StatusBadge label={item.severity} />
                    </td>
                    <td className="py-3 px-4 text-xs text-slate-800 max-w-sm">{item.message}</td>
                    <td className="py-3 px-4 font-mono text-xs text-slate-500">{item.evidence ?? '-'}</td>
                    <td className="py-3 px-4 text-right">
                      <Link to="/issues" className="text-xs font-semibold text-blue-600 hover:underline">
                        Review &rarr;
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
